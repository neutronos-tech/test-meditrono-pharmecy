use crate::{
    Backend, Error,
    database::{datetime_from_db, datetime_to_db, decimal_from_db, decimal_to_db, uuid_from_db},
    sync::enqueue_sync,
};
use contracts::{
    auth::PharmacyAccount,
    purchase::{Purchase, PurchaseInput, PurchaseItem},
    sync::SyncOperation,
};
use rusqlite::{OptionalExtension, params};
use time::OffsetDateTime;
use uuid::Uuid;

impl Backend {
    pub fn create_purchase(
        &mut self,
        pharmacy: &PharmacyAccount,
        input: PurchaseInput,
    ) -> Result<Purchase, Error> {
        let actor = pharmacy.id;
        let id = Uuid::now_v7();
        let operation_id = Uuid::now_v7();
        let now = OffsetDateTime::now_utc();
        let tx = self.db.transaction()?;
        let supplier_exists: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM suppliers WHERE id=?1)",
            [input.supplier_id.to_string()],
            |row| row.get(0),
        )?;
        if supplier_exists == 0 {
            return Err(Error::NotFound);
        }
        let duplicate: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM purchases WHERE supplier_id=?1 AND invoice_number=?2)",
            params![input.supplier_id.to_string(), input.invoice_number],
            |row| row.get(0),
        )?;
        if duplicate != 0 {
            return Err(Error::DuplicateInvoice);
        }
        for item in &input.items {
            let exists: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM medicines WHERE id=?1 AND active=1)",
                [item.medicine_id.to_string()],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(Error::NotFound);
            }
        }
        tx.execute(
            "INSERT INTO operations(id,kind,actor,occurred_at,actor_id,actor_name) VALUES(?1,'purchase',?2,?3,?2,?4)",
            params![
                operation_id.to_string(),
                actor.to_string(),
                datetime_to_db(now),
                pharmacy.username,
            ],
        )?;
        tx.execute("INSERT INTO purchases(id,operation_id,supplier_id,invoice_number,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6)",params![id.to_string(),operation_id.to_string(),input.supplier_id.to_string(),input.invoice_number,actor.to_string(),datetime_to_db(now)])?;
        let mut items = Vec::new();
        for item in input.items {
            let item_id = Uuid::now_v7();
            tx.execute("INSERT INTO purchase_items(id,purchase_id,medicine_id,quantity,unit_cost) VALUES(?1,?2,?3,?4,?5)",params![item_id.to_string(),id.to_string(),item.medicine_id.to_string(),item.quantity,decimal_to_db(item.unit_cost)])?;
            items.push(PurchaseItem {
                id: item_id,
                medicine_id: item.medicine_id,
                quantity: item.quantity,
                unit_cost: item.unit_cost,
            });
        }
        enqueue_sync(&tx, "purchase", id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(Purchase {
            id,
            operation_id,
            supplier_id: input.supplier_id,
            invoice_number: input.invoice_number,
            created_by: actor,
            created_at: now,
            items,
        })
    }
    pub fn purchase(&self, purchase_id: Uuid) -> Result<Purchase, Error> {
        let row=self.db.query_row("SELECT operation_id,supplier_id,invoice_number,created_by,created_at FROM purchases WHERE id=?1",[purchase_id.to_string()],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?))).optional()?.ok_or(Error::NotFound)?;
        let mut stmt=self.db.prepare("SELECT id,medicine_id,quantity,unit_cost FROM purchase_items WHERE purchase_id=?1 ORDER BY rowid")?;
        let rows = stmt.query_map([purchase_id.to_string()], |row| {
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
            id: purchase_id,
            operation_id: uuid_from_db(row.0)?,
            supplier_id: uuid_from_db(row.1)?,
            invoice_number: row.2,
            created_by: uuid_from_db(row.3)?,
            created_at: datetime_from_db(&row.4)?,
            items,
        })
    }
}
