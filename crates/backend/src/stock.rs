use crate::{
    Backend, Error,
    database::{
        bd_date, date_to_db, datetime_from_db, datetime_to_db, decimal_from_db, uuid_from_db,
    },
    sync::enqueue_sync,
};
use contracts::{
    auth::PharmacyAccount,
    stock::{
        ReceivablePurchase, ReceivablePurchaseDetail, ReceivablePurchaseItem, StockReceiptInput,
    },
    sync::SyncOperation,
};
use rusqlite::{OptionalExtension, params};
use time::OffsetDateTime;
use uuid::Uuid;

impl Backend {
    pub fn receivable_purchases(&self) -> Result<Vec<ReceivablePurchase>, Error> {
        let mut stmt=self.db.prepare("SELECT p.id,p.invoice_number,s.name,p.created_at,COUNT(pi.id) FROM purchases p JOIN suppliers s ON s.id=p.supplier_id JOIN purchase_items pi ON pi.purchase_id=p.id WHERE pi.received_quantity < pi.quantity GROUP BY p.id ORDER BY p.created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, u32>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (id, invoice_number, supplier_name, created_at, remaining_items) = row?;
            Ok(ReceivablePurchase {
                purchase_id: uuid_from_db(id)?,
                invoice_number,
                supplier_name,
                created_at: datetime_from_db(&created_at)?,
                remaining_items,
            })
        })
        .collect()
    }
    pub fn receivable_purchase(
        &self,
        purchase_id: Uuid,
    ) -> Result<ReceivablePurchaseDetail, Error> {
        let (invoice_number,supplier_name):(String,String)=self.db.query_row("SELECT p.invoice_number,s.name FROM purchases p JOIN suppliers s ON s.id=p.supplier_id WHERE p.id=?1",[purchase_id.to_string()],|row|Ok((row.get(0)?,row.get(1)?))).optional()?.ok_or(Error::NotFound)?;
        let mut stmt=self.db.prepare("SELECT pi.id,pi.medicine_id,m.name,pi.quantity,pi.received_quantity,pi.unit_cost FROM purchase_items pi JOIN medicines m ON m.id=pi.medicine_id WHERE pi.purchase_id=?1 AND pi.received_quantity<pi.quantity ORDER BY pi.rowid")?;
        let rows = stmt.query_map([purchase_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, u32>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        let items = rows
            .map(|row| {
                let (item_id, medicine_id, medicine_name, ordered, received, unit_cost) = row?;
                Ok(ReceivablePurchaseItem {
                    purchase_item_id: uuid_from_db(item_id)?,
                    medicine_id: uuid_from_db(medicine_id)?,
                    medicine_name,
                    ordered,
                    received,
                    remaining: ordered - received,
                    unit_cost: decimal_from_db(&unit_cost)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(ReceivablePurchaseDetail {
            purchase_id,
            invoice_number,
            supplier_name,
            items,
        })
    }
    pub fn receive_stock(
        &mut self,
        pharmacy: &PharmacyAccount,
        input: StockReceiptInput,
    ) -> Result<Uuid, Error> {
        let actor = pharmacy.id;
        let now = OffsetDateTime::now_utc();
        let operation_id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        let purchase_exists: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM purchases WHERE id=?1)",
            [input.purchase_id.to_string()],
            |row| row.get(0),
        )?;
        if purchase_exists == 0 {
            return Err(Error::NotFound);
        }
        tx.execute(
            "INSERT INTO operations(id,kind,actor,occurred_at,actor_id,actor_name) VALUES(?1,'stock_receipt',?2,?3,?2,?4)",
            params![
                operation_id.to_string(),
                actor.to_string(),
                datetime_to_db(now),
                pharmacy.username,
            ],
        )?;
        for line in input.lines {
            if line.expiry_date < bd_date(now) {
                return Err(Error::Conflict);
            }
            let item:Option<(String,u32,u32)>=tx.query_row("SELECT medicine_id,quantity,received_quantity FROM purchase_items WHERE id=?1 AND purchase_id=?2",params![line.purchase_item_id.to_string(),input.purchase_id.to_string()],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
            let (medicine_id, ordered, received) = item.ok_or(Error::NotFound)?;
            if line.quantity > ordered - received {
                return Err(Error::ReceiptExceedsPurchase);
            }
            let batch:Option<(String,String)>=tx.query_row("SELECT id,expiry_date FROM inventory_batches WHERE medicine_id=?1 AND batch_number=?2",params![medicine_id,line.batch_number],|row|Ok((row.get(0)?,row.get(1)?))).optional()?;
            let batch_id = if let Some((id, expiry)) = batch {
                if expiry != date_to_db(line.expiry_date) {
                    return Err(Error::BatchMetadataConflict);
                }
                uuid_from_db(id)?
            } else {
                let id = Uuid::now_v7();
                tx.execute("INSERT INTO inventory_batches(id,medicine_id,batch_number,expiry_date,quantity_on_hand) VALUES(?1,?2,?3,?4,0)",params![id.to_string(),medicine_id,line.batch_number,date_to_db(line.expiry_date)])?;
                id
            };
            tx.execute(
                "UPDATE inventory_batches SET quantity_on_hand=quantity_on_hand+?2 WHERE id=?1",
                params![batch_id.to_string(), line.quantity],
            )?;
            tx.execute(
                "UPDATE purchase_items SET received_quantity=received_quantity+?2 WHERE id=?1",
                params![line.purchase_item_id.to_string(), line.quantity],
            )?;
            tx.execute("INSERT INTO stock_inputs(id,operation_id,purchase_item_id,batch_id,quantity,created_at) VALUES(?1,?2,?3,?4,?5,?6)",params![Uuid::now_v7().to_string(),operation_id.to_string(),line.purchase_item_id.to_string(),batch_id.to_string(),line.quantity,datetime_to_db(now)])?;
            tx.execute("INSERT INTO inventory_movements(id,operation_id,batch_id,quantity_delta,created_at) VALUES(?1,?2,?3,?4,?5)",params![Uuid::now_v7().to_string(),operation_id.to_string(),batch_id.to_string(),line.quantity,datetime_to_db(now)])?;
        }
        enqueue_sync(&tx, "stock_receipt", operation_id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(operation_id)
    }
    pub fn reverse_stock_receipt(
        &mut self,
        pharmacy: &PharmacyAccount,
        operation_id: Uuid,
    ) -> Result<Uuid, Error> {
        let actor = pharmacy.id;
        let now = OffsetDateTime::now_utc();
        let reversal_id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        let kind: Option<String> = tx
            .query_row(
                "SELECT kind FROM operations WHERE id=?1",
                [operation_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if kind.as_deref() != Some("stock_receipt") {
            return Err(Error::NotFound);
        }
        let reversed: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE reversal_of=?1)",
            [operation_id.to_string()],
            |row| row.get(0),
        )?;
        if reversed != 0 {
            return Err(Error::AlreadyReversed);
        }
        let lines = {
            let mut stmt=tx.prepare("SELECT purchase_item_id,batch_id,quantity FROM stock_inputs WHERE operation_id=?1 ORDER BY rowid")?;
            stmt.query_map([operation_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
        };
        tx.execute("INSERT INTO operations(id,kind,actor,occurred_at,reversal_of,actor_id,actor_name) VALUES(?1,'stock_receipt_reversal',?2,?3,?4,?2,?5)",params![reversal_id.to_string(),actor.to_string(),datetime_to_db(now),operation_id.to_string(),pharmacy.username])?;
        for (item_id, batch_id, quantity) in lines {
            if tx.execute("UPDATE inventory_batches SET quantity_on_hand=quantity_on_hand-?2 WHERE id=?1 AND quantity_on_hand>=?2",params![batch_id,quantity])? == 0 {return Err(Error::StockWouldGoNegative);}
            tx.execute(
                "UPDATE purchase_items SET received_quantity=received_quantity-?2 WHERE id=?1",
                params![item_id, quantity],
            )?;
            tx.execute("INSERT INTO inventory_movements(id,operation_id,batch_id,quantity_delta,created_at) VALUES(?1,?2,?3,?4,?5)",params![Uuid::now_v7().to_string(),reversal_id.to_string(),batch_id,-i64::from(quantity),datetime_to_db(now)])?;
        }
        enqueue_sync(
            &tx,
            "stock_receipt_reversal",
            reversal_id,
            SyncOperation::Upsert,
        )?;
        tx.commit()?;
        Ok(reversal_id)
    }
}
