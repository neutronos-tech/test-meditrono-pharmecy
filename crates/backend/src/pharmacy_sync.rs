use crate::{
    Backend, Error,
    database::{
        date_from_db, date_to_db, datetime_from_db, datetime_to_db, decimal_from_db, decimal_to_db,
        uuid_from_db,
    },
};
use contracts::{
    catalog::{Medicine, Supplier},
    history::OperationKind,
    purchase::{Purchase, PurchaseItem},
    sale::{Sale, SaleItem},
    sync::{
        BatchSnapshot, ClientChange, InventoryMovementRecord, OperationSnapshot, PharmacyMedicine,
        PurchaseAggregate, PurchaseItemReceived, ReversalAggregate, SaleAggregate,
        SaleBatchAllocation, StockInputRecord, StockReceiptAggregate, SyncOperation, SyncPayload,
        SyncRequest, SyncResponse, SyncSummary,
    },
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

fn operation_kind(value: &str) -> Result<OperationKind, Error> {
    match value {
        "purchase" => Ok(OperationKind::Purchase),
        "stock_receipt" => Ok(OperationKind::StockReceipt),
        "stock_receipt_reversal" => Ok(OperationKind::StockReceiptReversal),
        "sale" => Ok(OperationKind::Sale),
        "sale_reversal" => Ok(OperationKind::SaleReversal),
        _ => Err(Error::InvalidStoredData),
    }
}

fn operation_snapshot(db: &Connection, id: Uuid) -> Result<OperationSnapshot, Error> {
    let row: Option<(String, String, String, String, Option<String>)> = db
        .query_row(
            "SELECT kind,actor_id,actor_name,occurred_at,reversal_of FROM operations WHERE id=?1",
            [id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;
    let (kind, actor_id, actor_name, occurred_at, reversal_of) = row.ok_or(Error::NotFound)?;
    Ok(OperationSnapshot {
        id,
        kind: operation_kind(&kind)?,
        actor_id: uuid_from_db(actor_id)?,
        actor_name,
        occurred_at: datetime_from_db(&occurred_at)?,
        reversal_of: reversal_of.map(uuid_from_db).transpose()?,
    })
}

fn movements(db: &Connection, operation_id: Uuid) -> Result<Vec<InventoryMovementRecord>, Error> {
    let mut stmt = db.prepare("SELECT id,batch_id,quantity_delta,created_at FROM inventory_movements WHERE operation_id=?1 ORDER BY rowid")?;
    let rows = stmt.query_map([operation_id.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    rows.map(|row| {
        let (id, batch_id, quantity_delta, created_at) = row?;
        Ok(InventoryMovementRecord {
            id: uuid_from_db(id)?,
            operation_id,
            batch_id: uuid_from_db(batch_id)?,
            quantity_delta,
            created_at: datetime_from_db(&created_at)?,
        })
    })
    .collect()
}

fn batches_for_ids(db: &Connection, ids: &[Uuid]) -> Result<Vec<BatchSnapshot>, Error> {
    let mut output = Vec::new();
    for id in ids {
        let row: Option<(String, String, String, i64)> = db
            .query_row(
                "SELECT medicine_id,batch_number,expiry_date,quantity_on_hand FROM inventory_batches WHERE id=?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let (medicine_id, batch_number, expiry, quantity_on_hand) = row.ok_or(Error::NotFound)?;
        output.push(BatchSnapshot {
            id: *id,
            medicine_id: uuid_from_db(medicine_id)?,
            batch_number,
            expiry_date: date_from_db(&expiry)?,
            quantity_on_hand: u64::try_from(quantity_on_hand)
                .map_err(|_| Error::InvalidStoredData)?,
        });
    }
    Ok(output)
}

fn purchase_items_for_receipt(
    db: &Connection,
    receipt_id: Uuid,
) -> Result<Vec<PurchaseItemReceived>, Error> {
    let mut stmt = db.prepare("SELECT DISTINCT pi.id,pi.received_quantity FROM purchase_items pi JOIN stock_inputs si ON si.purchase_item_id=pi.id WHERE si.operation_id=?1 ORDER BY pi.id")?;
    let rows = stmt.query_map([receipt_id.to_string()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;
    rows.map(|row| {
        let (id, received_quantity) = row?;
        Ok(PurchaseItemReceived {
            purchase_item_id: uuid_from_db(id)?,
            received_quantity,
        })
    })
    .collect()
}

fn stock_receipt(db: &Connection, operation_id: Uuid) -> Result<StockReceiptAggregate, Error> {
    let operation = operation_snapshot(db, operation_id)?;
    let mut stmt = db.prepare("SELECT id,purchase_item_id,batch_id,quantity,created_at FROM stock_inputs WHERE operation_id=?1 ORDER BY rowid")?;
    let rows = stmt.query_map([operation_id.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, u32>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let inputs = rows
        .map(|row| {
            let (id, item_id, batch_id, quantity, created_at) = row?;
            Ok(StockInputRecord {
                id: uuid_from_db(id)?,
                operation_id,
                purchase_item_id: uuid_from_db(item_id)?,
                batch_id: uuid_from_db(batch_id)?,
                quantity,
                created_at: datetime_from_db(&created_at)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let ids = inputs
        .iter()
        .map(|input| input.batch_id)
        .collect::<Vec<_>>();
    Ok(StockReceiptAggregate {
        operation,
        inputs,
        movements: movements(db, operation_id)?,
        batches: batches_for_ids(db, &ids)?,
        purchase_items: purchase_items_for_receipt(db, operation_id)?,
    })
}

fn reversal(db: &Connection, operation_id: Uuid) -> Result<ReversalAggregate, Error> {
    let operation = operation_snapshot(db, operation_id)?;
    let movements = movements(db, operation_id)?;
    let batch_ids = movements
        .iter()
        .map(|movement| movement.batch_id)
        .collect::<Vec<_>>();
    let purchase_items = if operation.kind == OperationKind::StockReceiptReversal {
        let original = operation.reversal_of.ok_or(Error::InvalidStoredData)?;
        purchase_items_for_receipt(db, original)?
    } else {
        Vec::new()
    };
    Ok(ReversalAggregate {
        operation,
        movements,
        batches: batches_for_ids(db, &batch_ids)?,
        purchase_items,
    })
}

fn sale(db: &Connection, sale_id: Uuid) -> Result<SaleAggregate, Error> {
    let row: (String, String, String) = db.query_row(
        "SELECT operation_id,created_by,created_at FROM sales WHERE id=?1",
        [sale_id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let operation_id = uuid_from_db(row.0)?;
    let mut stmt = db.prepare(
        "SELECT id,medicine_id,quantity,unit_price FROM sale_items WHERE sale_id=?1 ORDER BY rowid",
    )?;
    let rows = stmt.query_map([sale_id.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let items = rows
        .map(|row| {
            let (id, medicine_id, quantity, unit_price) = row?;
            Ok(SaleItem {
                id: uuid_from_db(id)?,
                medicine_id: uuid_from_db(medicine_id)?,
                quantity,
                unit_price: decimal_from_db(&unit_price)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut allocations = Vec::new();
    for item in &items {
        let mut stmt = db.prepare("SELECT batch_id,quantity FROM sale_item_batches WHERE sale_item_id=?1 ORDER BY batch_id")?;
        let rows = stmt.query_map([item.id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;
        for row in rows {
            let (batch_id, quantity) = row?;
            allocations.push(SaleBatchAllocation {
                sale_item_id: item.id,
                batch_id: uuid_from_db(batch_id)?,
                quantity,
            });
        }
    }
    let movements = movements(db, operation_id)?;
    let batch_ids = movements
        .iter()
        .map(|movement| movement.batch_id)
        .collect::<Vec<_>>();
    Ok(SaleAggregate {
        operation: operation_snapshot(db, operation_id)?,
        sale: Sale {
            id: sale_id,
            operation_id,
            created_by: uuid_from_db(row.1)?,
            created_at: datetime_from_db(&row.2)?,
            items,
        },
        allocations,
        movements,
        batches: batches_for_ids(db, &batch_ids)?,
    })
}

fn payload_for(db: &Connection, entity_type: &str, entity_id: Uuid) -> Result<SyncPayload, Error> {
    match entity_type {
        "supplier" => {
            let name: String = db.query_row(
                "SELECT name FROM suppliers WHERE id=?1",
                [entity_id.to_string()],
                |row| row.get(0),
            )?;
            Ok(SyncPayload::Supplier(Supplier {
                id: entity_id,
                name,
            }))
        }
        "medicine" => {
            let origin: String = db.query_row(
                "SELECT COALESCE(origin,'local') FROM medicines WHERE id=?1",
                [entity_id.to_string()],
                |row| row.get(0),
            )?;
            Ok(SyncPayload::Medicine(PharmacyMedicine {
                medicine: db_medicine(db, entity_id)?,
                origin,
            }))
        }
        "purchase" => {
            let purchase = load_purchase(db, entity_id)?;
            Ok(SyncPayload::Purchase(PurchaseAggregate {
                operation: operation_snapshot(db, purchase.operation_id)?,
                items: purchase.items.clone(),
                purchase,
            }))
        }
        "stock_receipt" => Ok(SyncPayload::StockReceipt(stock_receipt(db, entity_id)?)),
        "sale" => Ok(SyncPayload::Sale(sale(db, entity_id)?)),
        "stock_receipt_reversal" | "sale_reversal" => {
            Ok(SyncPayload::Reversal(reversal(db, entity_id)?))
        }
        _ => Err(Error::InvalidStoredData),
    }
}

fn db_medicine(db: &Connection, id: Uuid) -> Result<Medicine, Error> {
    let row = db.query_row("SELECT name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price,selling_price,active FROM medicines WHERE id=?1", [id.to_string()], |row| Ok((row.get::<_,String>(0)?,row.get::<_,Option<String>>(1)?,row.get::<_,Option<String>>(2)?,row.get::<_,Option<String>>(3)?,row.get::<_,Option<String>>(4)?,row.get::<_,Option<String>>(5)?,row.get::<_,Option<String>>(6)?,row.get::<_,Option<String>>(7)?,row.get::<_,bool>(8)?)))?;
    Ok(Medicine {
        id,
        name: row.0,
        generic_name: row.1,
        strength: row.2,
        dosage_form: row.3,
        manufacturer: row.4,
        package_description: row.5,
        reference_price: row.6.as_deref().map(decimal_from_db).transpose()?,
        selling_price: row.7.as_deref().map(decimal_from_db).transpose()?,
        active: row.8,
    })
}

fn load_purchase(db: &Connection, id: Uuid) -> Result<Purchase, Error> {
    let row: (String, String, String, String, String, String) = db.query_row("SELECT operation_id,supplier_id,invoice_number,created_by,created_at,id FROM purchases WHERE id=?1", [id.to_string()], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)))?;
    let mut stmt=db.prepare("SELECT id,medicine_id,quantity,unit_cost FROM purchase_items WHERE purchase_id=?1 ORDER BY rowid")?;
    let rows = stmt.query_map([id.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let items = rows
        .map(|row| {
            let (id, medicine_id, quantity, unit_cost) = row?;
            Ok(PurchaseItem {
                id: uuid_from_db(id)?,
                medicine_id: uuid_from_db(medicine_id)?,
                quantity,
                unit_cost: decimal_from_db(&unit_cost)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Purchase {
        id: uuid_from_db(row.5)?,
        operation_id: uuid_from_db(row.0)?,
        supplier_id: uuid_from_db(row.1)?,
        invoice_number: row.2,
        created_by: uuid_from_db(row.3)?,
        created_at: datetime_from_db(&row.4)?,
        items,
    })
}

impl Backend {
    pub fn prepare_sync_request(&self) -> Result<SyncRequest, Error> {
        let after_revision = self.pharmacy_revision()?;
        let mut stmt = self.db.prepare(
            "SELECT change_id,entity_type,entity_id,operation FROM sync_outbox ORDER BY seq",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut changes = Vec::new();
        for row in rows {
            let (change_id, entity_type, entity_id, operation) = row?;
            let entity_id = uuid_from_db(entity_id)?;
            let operation = match operation.as_str() {
                "upsert" => SyncOperation::Upsert,
                "delete" => SyncOperation::Delete,
                _ => return Err(Error::InvalidStoredData),
            };
            let payload = if operation == SyncOperation::Delete {
                SyncPayload::Tombstone
            } else {
                payload_for(&self.db, &entity_type, entity_id)?
            };
            changes.push(ClientChange {
                change_id: uuid_from_db(change_id)?,
                entity_type,
                entity_id,
                operation,
                payload,
            });
        }
        Ok(SyncRequest {
            after_revision,
            changes,
        })
    }

    pub fn apply_sync_response(&mut self, response: SyncResponse) -> Result<SyncSummary, Error> {
        let tx = self.db.transaction()?;
        let current_revision: i64 = tx.query_row(
            "SELECT pharmacy_revision FROM sync_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        let current_revision =
            u64::try_from(current_revision).map_err(|_| Error::InvalidStoredData)?;
        if response.revision < current_revision {
            return Err(Error::Conflict);
        }
        for change in &response.changes {
            apply_payload(&tx, &change.payload)?;
        }
        for id in &response.acknowledged {
            tx.execute(
                "DELETE FROM sync_outbox WHERE change_id=?1",
                [id.to_string()],
            )?;
        }
        let revision = i64::try_from(response.revision).map_err(|_| Error::Conflict)?;
        if response.revision > current_revision {
            tx.execute(
                "UPDATE sync_state SET pharmacy_revision=?1 WHERE singleton=1",
                [revision],
            )?;
        }
        let summary = SyncSummary {
            acknowledged: response.acknowledged.len(),
            applied: response.changes.len(),
            revision: response.revision,
        };
        tx.commit()?;
        Ok(summary)
    }
}

fn apply_operation(tx: &Transaction<'_>, operation: &OperationSnapshot) -> Result<(), Error> {
    let kind = match operation.kind {
        OperationKind::Purchase => "purchase",
        OperationKind::StockReceipt => "stock_receipt",
        OperationKind::StockReceiptReversal => "stock_receipt_reversal",
        OperationKind::Sale => "sale",
        OperationKind::SaleReversal => "sale_reversal",
    };
    tx.execute("INSERT INTO operations(id,kind,actor,occurred_at,reversal_of,actor_id,actor_name) VALUES(?1,?2,?3,?4,?5,?3,?6) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind,actor_name=excluded.actor_name,occurred_at=excluded.occurred_at,reversal_of=excluded.reversal_of,actor_id=excluded.actor_id", params![operation.id.to_string(),kind,operation.actor_id.to_string(),datetime_to_db(operation.occurred_at),operation.reversal_of.map(|id|id.to_string()),operation.actor_name])?;
    Ok(())
}

fn apply_batch(tx: &Transaction<'_>, batch: &BatchSnapshot) -> Result<(), Error> {
    tx.execute("INSERT INTO inventory_batches(id,medicine_id,batch_number,expiry_date,quantity_on_hand) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET medicine_id=excluded.medicine_id,batch_number=excluded.batch_number,expiry_date=excluded.expiry_date,quantity_on_hand=excluded.quantity_on_hand",params![batch.id.to_string(),batch.medicine_id.to_string(),batch.batch_number,date_to_db(batch.expiry_date),i64::try_from(batch.quantity_on_hand).map_err(|_|Error::InvalidStoredData)?])?;
    Ok(())
}

fn apply_movement(tx: &Transaction<'_>, movement: &InventoryMovementRecord) -> Result<(), Error> {
    tx.execute("INSERT OR IGNORE INTO inventory_movements(id,operation_id,batch_id,quantity_delta,created_at) VALUES(?1,?2,?3,?4,?5)",params![movement.id.to_string(),movement.operation_id.to_string(),movement.batch_id.to_string(),movement.quantity_delta,datetime_to_db(movement.created_at)])?;
    Ok(())
}

fn apply_payload(tx: &Transaction<'_>, payload: &SyncPayload) -> Result<(), Error> {
    match payload {
        SyncPayload::Supplier(supplier) => {
            tx.execute("INSERT INTO suppliers(id,name) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET name=excluded.name",params![supplier.id.to_string(),supplier.name])?;
        }
        SyncPayload::Medicine(pharmacy_medicine) => {
            let medicine = &pharmacy_medicine.medicine;
            tx.execute("INSERT INTO medicines(id,name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price,selling_price,active,origin) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET name=excluded.name,generic_name=excluded.generic_name,strength=excluded.strength,dosage_form=excluded.dosage_form,manufacturer=excluded.manufacturer,package_description=excluded.package_description,reference_price=excluded.reference_price,selling_price=excluded.selling_price,active=excluded.active,origin=excluded.origin",params![medicine.id.to_string(),medicine.name,medicine.generic_name,medicine.strength,medicine.dosage_form,medicine.manufacturer,medicine.package_description,medicine.reference_price.map(decimal_to_db),medicine.selling_price.map(decimal_to_db),medicine.active,pharmacy_medicine.origin])?;
        }
        SyncPayload::Purchase(aggregate) => {
            apply_operation(tx, &aggregate.operation)?;
            let p = &aggregate.purchase;
            tx.execute("INSERT INTO purchases(id,operation_id,supplier_id,invoice_number,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET operation_id=excluded.operation_id,supplier_id=excluded.supplier_id,invoice_number=excluded.invoice_number,created_by=excluded.created_by,created_at=excluded.created_at",params![p.id.to_string(),p.operation_id.to_string(),p.supplier_id.to_string(),p.invoice_number,p.created_by.to_string(),datetime_to_db(p.created_at)])?;
            for item in &aggregate.items {
                tx.execute("INSERT INTO purchase_items(id,purchase_id,medicine_id,quantity,unit_cost) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET purchase_id=excluded.purchase_id,medicine_id=excluded.medicine_id,quantity=excluded.quantity,unit_cost=excluded.unit_cost",params![item.id.to_string(),p.id.to_string(),item.medicine_id.to_string(),item.quantity,decimal_to_db(item.unit_cost)])?;
            }
        }
        SyncPayload::StockReceipt(aggregate) => {
            apply_operation(tx, &aggregate.operation)?;
            for batch in &aggregate.batches {
                apply_batch(tx, batch)?;
            }
            for input in &aggregate.inputs {
                tx.execute("INSERT OR IGNORE INTO stock_inputs(id,operation_id,purchase_item_id,batch_id,quantity,created_at) VALUES(?1,?2,?3,?4,?5,?6)",params![input.id.to_string(),input.operation_id.to_string(),input.purchase_item_id.to_string(),input.batch_id.to_string(),input.quantity,datetime_to_db(input.created_at)])?;
            }
            for item in &aggregate.purchase_items {
                tx.execute(
                    "UPDATE purchase_items SET received_quantity=?2 WHERE id=?1",
                    params![item.purchase_item_id.to_string(), item.received_quantity],
                )?;
            }
            for movement in &aggregate.movements {
                apply_movement(tx, movement)?;
            }
        }
        SyncPayload::Sale(aggregate) => {
            apply_operation(tx, &aggregate.operation)?;
            for batch in &aggregate.batches {
                apply_batch(tx, batch)?;
            }
            let sale = &aggregate.sale;
            tx.execute("INSERT INTO sales(id,operation_id,created_by,created_at) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET operation_id=excluded.operation_id,created_by=excluded.created_by,created_at=excluded.created_at",params![sale.id.to_string(),sale.operation_id.to_string(),sale.created_by.to_string(),datetime_to_db(sale.created_at)])?;
            for item in &sale.items {
                tx.execute("INSERT INTO sale_items(id,sale_id,medicine_id,quantity,unit_price) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET sale_id=excluded.sale_id,medicine_id=excluded.medicine_id,quantity=excluded.quantity,unit_price=excluded.unit_price",params![item.id.to_string(),sale.id.to_string(),item.medicine_id.to_string(),item.quantity,decimal_to_db(item.unit_price)])?;
            }
            for allocation in &aggregate.allocations {
                tx.execute("INSERT INTO sale_item_batches(sale_item_id,batch_id,quantity) VALUES(?1,?2,?3) ON CONFLICT(sale_item_id,batch_id) DO UPDATE SET quantity=excluded.quantity",params![allocation.sale_item_id.to_string(),allocation.batch_id.to_string(),allocation.quantity])?;
            }
            for movement in &aggregate.movements {
                apply_movement(tx, movement)?;
            }
        }
        SyncPayload::Reversal(aggregate) => {
            apply_operation(tx, &aggregate.operation)?;
            for batch in &aggregate.batches {
                apply_batch(tx, batch)?;
            }
            for movement in &aggregate.movements {
                apply_movement(tx, movement)?;
            }
            for item in &aggregate.purchase_items {
                tx.execute(
                    "UPDATE purchase_items SET received_quantity=?2 WHERE id=?1",
                    params![item.purchase_item_id.to_string(), item.received_quantity],
                )?;
            }
        }
        SyncPayload::Tombstone => {}
    }
    Ok(())
}
