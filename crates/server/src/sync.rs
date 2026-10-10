use crate::{app::AppState, error::AppError};
use contracts::{
    auth::PharmacyAccount,
    catalog::{Medicine, Supplier},
    history::OperationKind,
    purchase::{Purchase, PurchaseItem},
    sale::{Sale, SaleItem},
    sync::{
        BatchSnapshot, ClientChange, InventoryMovementRecord, OperationSnapshot, PharmacyMedicine,
        PurchaseAggregate, PurchaseItemReceived, ReversalAggregate, SaleAggregate,
        SaleBatchAllocation, ServerChange, StockInputRecord, StockReceiptAggregate, SyncOperation,
        SyncPayload, SyncRequest, SyncResponse,
    },
};
use sqlx_core::{query::query, row::Row, transaction::Transaction};
use sqlx_postgres::Postgres;
use uuid::Uuid;

fn operation_kind(value: &str) -> Result<OperationKind, AppError> {
    match value {
        "purchase" => Ok(OperationKind::Purchase),
        "stock_receipt" => Ok(OperationKind::StockReceipt),
        "stock_receipt_reversal" => Ok(OperationKind::StockReceiptReversal),
        "sale" => Ok(OperationKind::Sale),
        "sale_reversal" => Ok(OperationKind::SaleReversal),
        _ => Err(AppError::Internal),
    }
}

fn operation_kind_name(kind: OperationKind) -> &'static str {
    match kind {
        OperationKind::Purchase => "purchase",
        OperationKind::StockReceipt => "stock_receipt",
        OperationKind::StockReceiptReversal => "stock_receipt_reversal",
        OperationKind::Sale => "sale",
        OperationKind::SaleReversal => "sale_reversal",
    }
}

async fn owns(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    pharmacy: Uuid,
    id: Uuid,
) -> Result<bool, AppError> {
    let sql =
        format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE pharmacy_id=$1 AND id=$2) AS found");
    Ok(query(&sql)
        .bind(pharmacy)
        .bind(id)
        .fetch_one(&mut **tx)
        .await?
        .try_get("found")?)
}

async fn require_owned(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
    pharmacy: Uuid,
    id: Uuid,
) -> Result<(), AppError> {
    if owns(tx, table, pharmacy, id).await? {
        Ok(())
    } else {
        Err(AppError::BadRequest)
    }
}

async fn save_operation(
    tx: &mut Transaction<'_, Postgres>,
    pharmacy: &PharmacyAccount,
    operation: &OperationSnapshot,
) -> Result<(), AppError> {
    if operation.actor_id != pharmacy.id {
        return Err(AppError::BadRequest);
    }
    if let Some(original) = operation.reversal_of {
        require_owned(tx, "pharmacy_operations", pharmacy.id, original).await?;
    }
    query("INSERT INTO pharmacy_operations(id,pharmacy_id,kind,actor_id,actor_name,occurred_at,reversal_of) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind,actor_id=excluded.actor_id,actor_name=excluded.actor_name,occurred_at=excluded.occurred_at,reversal_of=excluded.reversal_of WHERE pharmacy_operations.pharmacy_id=excluded.pharmacy_id")
        .bind(operation.id).bind(pharmacy.id).bind(operation_kind_name(operation.kind)).bind(operation.actor_id).bind(&operation.actor_name)
        .bind(operation.occurred_at).bind(operation.reversal_of)
        .execute(&mut **tx).await?;
    require_owned(tx, "pharmacy_operations", pharmacy.id, operation.id).await?;
    Ok(())
}

async fn apply_client_change(
    tx: &mut Transaction<'_, Postgres>,
    pharmacy: &PharmacyAccount,
    change: &ClientChange,
) -> Result<(), AppError> {
    if change.operation == SyncOperation::Delete {
        return Err(AppError::BadRequest);
    }
    let owner = pharmacy.id;
    match &change.payload {
        SyncPayload::Supplier(supplier)
            if change.entity_type == "supplier" && supplier.id == change.entity_id =>
        {
            query("INSERT INTO pharmacy_suppliers(id,pharmacy_id,name) VALUES($1,$2,$3) ON CONFLICT(id) DO UPDATE SET name=excluded.name WHERE pharmacy_suppliers.pharmacy_id=excluded.pharmacy_id")
                .bind(supplier.id).bind(owner).bind(&supplier.name).execute(&mut **tx).await?;
            require_owned(tx, "pharmacy_suppliers", owner, supplier.id).await?;
        }
        SyncPayload::Medicine(pharmacy_medicine)
            if change.entity_type == "medicine"
                && pharmacy_medicine.medicine.id == change.entity_id =>
        {
            let medicine = &pharmacy_medicine.medicine;
            query("INSERT INTO pharmacy_medicines(id,pharmacy_id,name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price,selling_price,origin,active) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'local',$11) ON CONFLICT(id) DO UPDATE SET name=excluded.name,generic_name=excluded.generic_name,strength=excluded.strength,dosage_form=excluded.dosage_form,manufacturer=excluded.manufacturer,package_description=excluded.package_description,reference_price=excluded.reference_price,selling_price=excluded.selling_price,active=excluded.active WHERE pharmacy_medicines.pharmacy_id=excluded.pharmacy_id")
                .bind(medicine.id).bind(owner).bind(&medicine.name).bind(&medicine.generic_name).bind(&medicine.strength).bind(&medicine.dosage_form).bind(&medicine.manufacturer).bind(&medicine.package_description).bind(medicine.reference_price).bind(medicine.selling_price).bind(medicine.active).execute(&mut **tx).await?;
            query("UPDATE pharmacy_medicines SET origin=$3 WHERE pharmacy_id=$1 AND id=$2")
                .bind(owner)
                .bind(medicine.id)
                .bind(&pharmacy_medicine.origin)
                .execute(&mut **tx)
                .await?;
            require_owned(tx, "pharmacy_medicines", owner, medicine.id).await?;
        }
        SyncPayload::Purchase(aggregate)
            if change.entity_type == "purchase"
                && aggregate.purchase.id == change.entity_id
                && aggregate.purchase.operation_id == aggregate.operation.id
                && aggregate.operation.kind == OperationKind::Purchase =>
        {
            require_owned(
                tx,
                "pharmacy_suppliers",
                owner,
                aggregate.purchase.supplier_id,
            )
            .await?;
            save_operation(tx, pharmacy, &aggregate.operation).await?;
            let purchase = &aggregate.purchase;
            query("INSERT INTO pharmacy_purchases(id,pharmacy_id,operation_id,supplier_id,invoice_number,created_by,created_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO UPDATE SET operation_id=excluded.operation_id,supplier_id=excluded.supplier_id,invoice_number=excluded.invoice_number,created_by=excluded.created_by,created_at=excluded.created_at WHERE pharmacy_purchases.pharmacy_id=excluded.pharmacy_id")
                .bind(purchase.id).bind(owner).bind(purchase.operation_id).bind(purchase.supplier_id).bind(&purchase.invoice_number).bind(owner).bind(purchase.created_at).execute(&mut **tx).await?;
            require_owned(tx, "pharmacy_purchases", owner, purchase.id).await?;
            for item in &aggregate.items {
                require_owned(tx, "pharmacy_medicines", owner, item.medicine_id).await?;
                query("INSERT INTO pharmacy_purchase_items(id,pharmacy_id,purchase_id,medicine_id,quantity,unit_cost) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(id) DO UPDATE SET purchase_id=excluded.purchase_id,medicine_id=excluded.medicine_id,quantity=excluded.quantity,unit_cost=excluded.unit_cost WHERE pharmacy_purchase_items.pharmacy_id=excluded.pharmacy_id")
                    .bind(item.id).bind(owner).bind(purchase.id).bind(item.medicine_id).bind(i32::try_from(item.quantity).map_err(|_|AppError::BadRequest)?).bind(item.unit_cost).execute(&mut **tx).await?;
                require_owned(tx, "pharmacy_purchase_items", owner, item.id).await?;
            }
        }
        SyncPayload::StockReceipt(aggregate)
            if change.entity_type == "stock_receipt"
                && aggregate.operation.id == change.entity_id =>
        {
            if aggregate.operation.kind != OperationKind::StockReceipt {
                return Err(AppError::BadRequest);
            }
            save_operation(tx, pharmacy, &aggregate.operation).await?;
            save_batches(tx, owner, &aggregate.batches).await?;
            for input in &aggregate.inputs {
                if input.operation_id != aggregate.operation.id {
                    return Err(AppError::BadRequest);
                }
                require_owned(tx, "pharmacy_purchase_items", owner, input.purchase_item_id).await?;
                require_owned(tx, "pharmacy_inventory_batches", owner, input.batch_id).await?;
                query("INSERT INTO pharmacy_stock_inputs(id,pharmacy_id,operation_id,purchase_item_id,batch_id,quantity,created_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO NOTHING")
                    .bind(input.id).bind(owner).bind(input.operation_id).bind(input.purchase_item_id).bind(input.batch_id).bind(i32::try_from(input.quantity).map_err(|_|AppError::BadRequest)?).bind(input.created_at).execute(&mut **tx).await?;
                require_owned(tx, "pharmacy_stock_inputs", owner, input.id).await?;
            }
            if aggregate
                .movements
                .iter()
                .any(|movement| movement.operation_id != aggregate.operation.id)
            {
                return Err(AppError::BadRequest);
            }
            save_movements(tx, owner, &aggregate.movements).await?;
            for item in &aggregate.purchase_items {
                query("UPDATE pharmacy_purchase_items SET received_quantity=$3 WHERE pharmacy_id=$1 AND id=$2")
                    .bind(owner).bind(item.purchase_item_id).bind(i32::try_from(item.received_quantity).map_err(|_|AppError::BadRequest)?).execute(&mut **tx).await?;
            }
        }
        SyncPayload::Sale(aggregate)
            if change.entity_type == "sale"
                && aggregate.sale.id == change.entity_id
                && aggregate.sale.operation_id == aggregate.operation.id
                && aggregate.operation.kind == OperationKind::Sale =>
        {
            save_operation(tx, pharmacy, &aggregate.operation).await?;
            save_batches(tx, owner, &aggregate.batches).await?;
            let sale = &aggregate.sale;
            query("INSERT INTO pharmacy_sales(id,pharmacy_id,operation_id,created_by,created_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(id) DO UPDATE SET operation_id=excluded.operation_id,created_by=excluded.created_by,created_at=excluded.created_at WHERE pharmacy_sales.pharmacy_id=excluded.pharmacy_id")
                .bind(sale.id).bind(owner).bind(sale.operation_id).bind(owner).bind(sale.created_at).execute(&mut **tx).await?;
            require_owned(tx, "pharmacy_sales", owner, sale.id).await?;
            for item in &sale.items {
                require_owned(tx, "pharmacy_medicines", owner, item.medicine_id).await?;
                query("INSERT INTO pharmacy_sale_items(id,pharmacy_id,sale_id,medicine_id,quantity,unit_price) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(id) DO UPDATE SET sale_id=excluded.sale_id,medicine_id=excluded.medicine_id,quantity=excluded.quantity,unit_price=excluded.unit_price WHERE pharmacy_sale_items.pharmacy_id=excluded.pharmacy_id")
                    .bind(item.id).bind(owner).bind(sale.id).bind(item.medicine_id).bind(i32::try_from(item.quantity).map_err(|_|AppError::BadRequest)?).bind(item.unit_price).execute(&mut **tx).await?;
                require_owned(tx, "pharmacy_sale_items", owner, item.id).await?;
            }
            for allocation in &aggregate.allocations {
                require_owned(tx, "pharmacy_sale_items", owner, allocation.sale_item_id).await?;
                require_owned(tx, "pharmacy_inventory_batches", owner, allocation.batch_id).await?;
                query("INSERT INTO pharmacy_sale_item_batches(pharmacy_id,sale_item_id,batch_id,quantity) VALUES($1,$2,$3,$4) ON CONFLICT(sale_item_id,batch_id) DO UPDATE SET quantity=excluded.quantity WHERE pharmacy_sale_item_batches.pharmacy_id=excluded.pharmacy_id")
                    .bind(owner).bind(allocation.sale_item_id).bind(allocation.batch_id).bind(i32::try_from(allocation.quantity).map_err(|_|AppError::BadRequest)?).execute(&mut **tx).await?;
            }
            if aggregate
                .movements
                .iter()
                .any(|movement| movement.operation_id != aggregate.operation.id)
            {
                return Err(AppError::BadRequest);
            }
            save_movements(tx, owner, &aggregate.movements).await?;
        }
        SyncPayload::Reversal(aggregate)
            if ((change.entity_type == "stock_receipt_reversal"
                && aggregate.operation.kind == OperationKind::StockReceiptReversal)
                || (change.entity_type == "sale_reversal"
                    && aggregate.operation.kind == OperationKind::SaleReversal))
                && aggregate.operation.id == change.entity_id
                && aggregate.operation.reversal_of.is_some() =>
        {
            save_operation(tx, pharmacy, &aggregate.operation).await?;
            save_batches(tx, owner, &aggregate.batches).await?;
            if aggregate
                .movements
                .iter()
                .any(|movement| movement.operation_id != aggregate.operation.id)
            {
                return Err(AppError::BadRequest);
            }
            save_movements(tx, owner, &aggregate.movements).await?;
            for item in &aggregate.purchase_items {
                require_owned(tx, "pharmacy_purchase_items", owner, item.purchase_item_id).await?;
                query("UPDATE pharmacy_purchase_items SET received_quantity=$3 WHERE pharmacy_id=$1 AND id=$2")
                    .bind(owner).bind(item.purchase_item_id).bind(i32::try_from(item.received_quantity).map_err(|_|AppError::BadRequest)?).execute(&mut **tx).await?;
            }
        }
        _ => return Err(AppError::BadRequest),
    }
    Ok(())
}

async fn save_movements(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    movements: &[InventoryMovementRecord],
) -> Result<(), AppError> {
    for movement in movements {
        require_owned(tx, "pharmacy_operations", owner, movement.operation_id).await?;
        require_owned(tx, "pharmacy_inventory_batches", owner, movement.batch_id).await?;
        query("INSERT INTO pharmacy_inventory_movements(id,pharmacy_id,operation_id,batch_id,quantity_delta,created_at) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(id) DO NOTHING")
            .bind(movement.id).bind(owner).bind(movement.operation_id).bind(movement.batch_id).bind(movement.quantity_delta).bind(movement.created_at).execute(&mut **tx).await?;
        require_owned(tx, "pharmacy_inventory_movements", owner, movement.id).await?;
    }
    Ok(())
}

async fn save_batches(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    batches: &[BatchSnapshot],
) -> Result<(), AppError> {
    for batch in batches {
        require_owned(tx, "pharmacy_medicines", owner, batch.medicine_id).await?;
        query("INSERT INTO pharmacy_inventory_batches(id,pharmacy_id,medicine_id,batch_number,expiry_date,quantity_on_hand) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(id) DO UPDATE SET medicine_id=excluded.medicine_id,batch_number=excluded.batch_number,expiry_date=excluded.expiry_date,quantity_on_hand=excluded.quantity_on_hand WHERE pharmacy_inventory_batches.pharmacy_id=excluded.pharmacy_id")
            .bind(batch.id).bind(owner).bind(batch.medicine_id).bind(&batch.batch_number).bind(batch.expiry_date).bind(i64::try_from(batch.quantity_on_hand).map_err(|_|AppError::BadRequest)?).execute(&mut **tx).await?;
        require_owned(tx, "pharmacy_inventory_batches", owner, batch.id).await?;
    }
    Ok(())
}

impl AppState {
    pub async fn synchronize(
        &self,
        pharmacy: &PharmacyAccount,
        request: SyncRequest,
    ) -> Result<SyncResponse, AppError> {
        let after = i64::try_from(request.after_revision).map_err(|_| AppError::BadRequest)?;
        let mut tx = self.pool.begin().await?;
        let mut acknowledged = Vec::with_capacity(request.changes.len());
        for change in &request.changes {
            let inserted = query("INSERT INTO pharmacy_processed_changes(pharmacy_id,change_id) VALUES($1,$2) ON CONFLICT(pharmacy_id,change_id) DO NOTHING")
                .bind(pharmacy.id).bind(change.change_id).execute(&mut *tx).await?.rows_affected();
            if inserted > 0 {
                apply_client_change(&mut tx, pharmacy, change).await?;
                query("INSERT INTO pharmacy_changes(pharmacy_id,entity_type,entity_id) VALUES($1,$2,$3)")
                    .bind(pharmacy.id).bind(&change.entity_type).bind(change.entity_id).execute(&mut *tx).await?;
            }
            acknowledged.push(change.change_id);
        }
        let revision:i64=query("SELECT COALESCE(MAX(revision),0) AS revision FROM pharmacy_changes WHERE pharmacy_id=$1")
            .bind(pharmacy.id).fetch_one(&mut *tx).await?.try_get("revision")?;
        let rows=query("SELECT revision,entity_type,entity_id FROM pharmacy_changes WHERE pharmacy_id=$1 AND revision>$2 AND revision<=$3 ORDER BY revision")
            .bind(pharmacy.id).bind(after).bind(revision).fetch_all(&mut *tx).await?;
        let mut changes = Vec::with_capacity(rows.len());
        for row in rows {
            let marker_revision: i64 = row.try_get("revision")?;
            let entity_type: String = row.try_get("entity_type")?;
            let entity_id: Uuid = row.try_get("entity_id")?;
            let payload = load_payload(&mut tx, pharmacy.id, &entity_type, entity_id).await?;
            changes.push(ServerChange {
                revision: u64::try_from(marker_revision).map_err(|_| AppError::Internal)?,
                entity_type,
                entity_id,
                payload,
            });
        }
        let revision = u64::try_from(revision).map_err(|_| AppError::Internal)?;
        tx.commit().await?;
        Ok(SyncResponse {
            acknowledged,
            revision,
            changes,
        })
    }
}

async fn load_operation(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    id: Uuid,
) -> Result<OperationSnapshot, AppError> {
    let row=query("SELECT kind,actor_id,actor_name,occurred_at,reversal_of FROM pharmacy_operations WHERE pharmacy_id=$1 AND id=$2")
        .bind(owner).bind(id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
    let kind: String = row.try_get("kind")?;
    Ok(OperationSnapshot {
        id,
        kind: operation_kind(&kind)?,
        actor_id: row.try_get("actor_id")?,
        actor_name: row.try_get("actor_name")?,
        occurred_at: row.try_get("occurred_at")?,
        reversal_of: row.try_get("reversal_of")?,
    })
}

async fn load_medicine(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    id: Uuid,
) -> Result<Medicine, AppError> {
    let row=query("SELECT name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price::TEXT AS reference_price,selling_price::TEXT AS selling_price,active FROM pharmacy_medicines WHERE pharmacy_id=$1 AND id=$2")
        .bind(owner).bind(id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
    let reference: Option<String> = row.try_get("reference_price")?;
    let selling: Option<String> = row.try_get("selling_price")?;
    Ok(Medicine {
        id,
        name: row.try_get("name")?,
        generic_name: row.try_get("generic_name")?,
        strength: row.try_get("strength")?,
        dosage_form: row.try_get("dosage_form")?,
        manufacturer: row.try_get("manufacturer")?,
        package_description: row.try_get("package_description")?,
        reference_price: reference
            .map(|v| v.parse().map_err(|_| AppError::Internal))
            .transpose()?,
        selling_price: selling
            .map(|v| v.parse().map_err(|_| AppError::Internal))
            .transpose()?,
        active: row.try_get("active")?,
    })
}

async fn load_batches(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    ids: &[Uuid],
) -> Result<Vec<BatchSnapshot>, AppError> {
    let mut output = Vec::new();
    for id in ids {
        let row=query("SELECT medicine_id,batch_number,expiry_date,quantity_on_hand FROM pharmacy_inventory_batches WHERE pharmacy_id=$1 AND id=$2")
            .bind(owner).bind(id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
        let quantity: i64 = row.try_get("quantity_on_hand")?;
        output.push(BatchSnapshot {
            id: *id,
            medicine_id: row.try_get("medicine_id")?,
            batch_number: row.try_get("batch_number")?,
            expiry_date: row.try_get("expiry_date")?,
            quantity_on_hand: u64::try_from(quantity).map_err(|_| AppError::Internal)?,
        });
    }
    Ok(output)
}

async fn load_movements(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    operation_id: Uuid,
) -> Result<Vec<InventoryMovementRecord>, AppError> {
    let rows=query("SELECT id,batch_id,quantity_delta,created_at FROM pharmacy_inventory_movements WHERE pharmacy_id=$1 AND operation_id=$2 ORDER BY id")
        .bind(owner).bind(operation_id).fetch_all(&mut **tx).await?;
    rows.into_iter()
        .map(|row| {
            Ok(InventoryMovementRecord {
                id: row.try_get("id")?,
                operation_id,
                batch_id: row.try_get("batch_id")?,
                quantity_delta: row.try_get("quantity_delta")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn load_payload(
    tx: &mut Transaction<'_, Postgres>,
    owner: Uuid,
    kind: &str,
    id: Uuid,
) -> Result<SyncPayload, AppError> {
    match kind {
        "supplier" => {
            let row = query("SELECT name FROM pharmacy_suppliers WHERE pharmacy_id=$1 AND id=$2")
                .bind(owner)
                .bind(id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or(AppError::NotFound)?;
            Ok(SyncPayload::Supplier(Supplier {
                id,
                name: row.try_get("name")?,
            }))
        }
        "medicine" => {
            let row = query("SELECT origin FROM pharmacy_medicines WHERE pharmacy_id=$1 AND id=$2")
                .bind(owner)
                .bind(id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or(AppError::NotFound)?;
            Ok(SyncPayload::Medicine(PharmacyMedicine {
                medicine: load_medicine(tx, owner, id).await?,
                origin: row.try_get("origin")?,
            }))
        }
        "purchase" => {
            let row=query("SELECT operation_id,supplier_id,invoice_number,created_by,created_at FROM pharmacy_purchases WHERE pharmacy_id=$1 AND id=$2").bind(owner).bind(id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
            let operation_id: Uuid = row.try_get("operation_id")?;
            let rows=query("SELECT id,medicine_id,quantity,unit_cost::TEXT AS unit_cost FROM pharmacy_purchase_items WHERE pharmacy_id=$1 AND purchase_id=$2 ORDER BY id").bind(owner).bind(id).fetch_all(&mut **tx).await?;
            let items = rows
                .into_iter()
                .map(|row| {
                    Ok(PurchaseItem {
                        id: row.try_get("id")?,
                        medicine_id: row.try_get("medicine_id")?,
                        quantity: u32::try_from(row.try_get::<i32, _>("quantity")?)
                            .map_err(|_| AppError::Internal)?,
                        unit_cost: row
                            .try_get::<String, _>("unit_cost")?
                            .parse()
                            .map_err(|_| AppError::Internal)?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            let purchase = Purchase {
                id,
                operation_id,
                supplier_id: row.try_get("supplier_id")?,
                invoice_number: row.try_get("invoice_number")?,
                created_by: row.try_get("created_by")?,
                created_at: row.try_get("created_at")?,
                items: items.clone(),
            };
            Ok(SyncPayload::Purchase(PurchaseAggregate {
                operation: load_operation(tx, owner, operation_id).await?,
                purchase,
                items,
            }))
        }
        "stock_receipt" => {
            let operation = load_operation(tx, owner, id).await?;
            let rows=query("SELECT id,purchase_item_id,batch_id,quantity,created_at FROM pharmacy_stock_inputs WHERE pharmacy_id=$1 AND operation_id=$2 ORDER BY id").bind(owner).bind(id).fetch_all(&mut **tx).await?;
            let inputs = rows
                .into_iter()
                .map(|row| {
                    Ok(StockInputRecord {
                        id: row.try_get("id")?,
                        operation_id: id,
                        purchase_item_id: row.try_get("purchase_item_id")?,
                        batch_id: row.try_get("batch_id")?,
                        quantity: u32::try_from(row.try_get::<i32, _>("quantity")?)
                            .map_err(|_| AppError::Internal)?,
                        created_at: row.try_get("created_at")?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            let ids = inputs.iter().map(|item| item.batch_id).collect::<Vec<_>>();
            let movements = load_movements(tx, owner, id).await?;
            let mut purchase_items = Vec::new();
            for item_id in inputs
                .iter()
                .map(|item| item.purchase_item_id)
                .collect::<std::collections::HashSet<_>>()
            {
                let row=query("SELECT received_quantity FROM pharmacy_purchase_items WHERE pharmacy_id=$1 AND id=$2").bind(owner).bind(item_id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
                purchase_items.push(PurchaseItemReceived {
                    purchase_item_id: item_id,
                    received_quantity: u32::try_from(row.try_get::<i32, _>("received_quantity")?)
                        .map_err(|_| AppError::Internal)?,
                });
            }
            Ok(SyncPayload::StockReceipt(StockReceiptAggregate {
                operation,
                inputs,
                movements,
                batches: load_batches(tx, owner, &ids).await?,
                purchase_items,
            }))
        }
        "sale" => {
            let row=query("SELECT operation_id,created_by,created_at FROM pharmacy_sales WHERE pharmacy_id=$1 AND id=$2").bind(owner).bind(id).fetch_optional(&mut **tx).await?.ok_or(AppError::NotFound)?;
            let operation_id: Uuid = row.try_get("operation_id")?;
            let rows=query("SELECT id,medicine_id,quantity,unit_price::TEXT AS unit_price FROM pharmacy_sale_items WHERE pharmacy_id=$1 AND sale_id=$2 ORDER BY id").bind(owner).bind(id).fetch_all(&mut **tx).await?;
            let items = rows
                .into_iter()
                .map(|row| {
                    Ok(SaleItem {
                        id: row.try_get("id")?,
                        medicine_id: row.try_get("medicine_id")?,
                        quantity: u32::try_from(row.try_get::<i32, _>("quantity")?)
                            .map_err(|_| AppError::Internal)?,
                        unit_price: row
                            .try_get::<String, _>("unit_price")?
                            .parse()
                            .map_err(|_| AppError::Internal)?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            let rows=query("SELECT sale_item_id,batch_id,quantity FROM pharmacy_sale_item_batches WHERE pharmacy_id=$1 AND sale_item_id=ANY(SELECT id FROM pharmacy_sale_items WHERE pharmacy_id=$1 AND sale_id=$2) ORDER BY sale_item_id,batch_id").bind(owner).bind(id).fetch_all(&mut **tx).await?;
            let allocations = rows
                .into_iter()
                .map(|row| {
                    Ok(SaleBatchAllocation {
                        sale_item_id: row.try_get("sale_item_id")?,
                        batch_id: row.try_get("batch_id")?,
                        quantity: u32::try_from(row.try_get::<i32, _>("quantity")?)
                            .map_err(|_| AppError::Internal)?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            let movements = load_movements(tx, owner, operation_id).await?;
            let batch_ids = movements
                .iter()
                .map(|movement| movement.batch_id)
                .collect::<Vec<_>>();
            let sale = Sale {
                id,
                operation_id,
                created_by: row.try_get("created_by")?,
                created_at: row.try_get("created_at")?,
                items,
            };
            Ok(SyncPayload::Sale(SaleAggregate {
                operation: load_operation(tx, owner, operation_id).await?,
                sale,
                allocations,
                movements,
                batches: load_batches(tx, owner, &batch_ids).await?,
            }))
        }
        "stock_receipt_reversal" | "sale_reversal" => {
            let operation = load_operation(tx, owner, id).await?;
            let movements = load_movements(tx, owner, id).await?;
            let batch_ids = movements
                .iter()
                .map(|movement| movement.batch_id)
                .collect::<Vec<_>>();
            let mut purchase_items = Vec::new();
            if operation.kind == OperationKind::StockReceiptReversal {
                let original = operation.reversal_of.ok_or(AppError::Internal)?;
                let rows=query("SELECT DISTINCT pi.id,pi.received_quantity FROM pharmacy_stock_inputs si JOIN pharmacy_purchase_items pi ON pi.id=si.purchase_item_id AND pi.pharmacy_id=si.pharmacy_id WHERE si.pharmacy_id=$1 AND si.operation_id=$2 ORDER BY pi.id").bind(owner).bind(original).fetch_all(&mut **tx).await?;
                purchase_items = rows
                    .into_iter()
                    .map(|row| {
                        Ok(PurchaseItemReceived {
                            purchase_item_id: row.try_get("id")?,
                            received_quantity: u32::try_from(
                                row.try_get::<i32, _>("received_quantity")?,
                            )
                            .map_err(|_| AppError::Internal)?,
                        })
                    })
                    .collect::<Result<Vec<_>, AppError>>()?;
            }
            Ok(SyncPayload::Reversal(ReversalAggregate {
                operation,
                movements,
                batches: load_batches(tx, owner, &batch_ids).await?,
                purchase_items,
            }))
        }
        _ => Err(AppError::NotFound),
    }
}
