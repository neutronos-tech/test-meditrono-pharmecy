use crate::{
    Backend, Error,
    database::{bd_date, date_to_db, datetime_to_db, decimal_to_db},
    sync::enqueue_sync,
};
use contracts::{
    auth::PharmacyAccount,
    sale::{Sale, SaleInput, SaleItem},
    sync::SyncOperation,
};
use rusqlite::{OptionalExtension, params};
use time::OffsetDateTime;
use uuid::Uuid;

impl Backend {
    pub fn create_sale(
        &mut self,
        pharmacy: &PharmacyAccount,
        input: SaleInput,
    ) -> Result<Sale, Error> {
        let actor = pharmacy.id;
        let now = OffsetDateTime::now_utc();
        let id = Uuid::now_v7();
        let operation_id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO operations(id,kind,actor,occurred_at,actor_id,actor_name) VALUES(?1,'sale',?2,?3,?2,?4)",
            params![
                operation_id.to_string(),
                actor.to_string(),
                datetime_to_db(now),
                pharmacy.username,
            ],
        )?;
        tx.execute(
            "INSERT INTO sales(id,operation_id,created_by,created_at) VALUES(?1,?2,?3,?4)",
            params![
                id.to_string(),
                operation_id.to_string(),
                actor.to_string(),
                datetime_to_db(now)
            ],
        )?;
        let mut items = Vec::new();
        for item in input.items {
            let exists: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM medicines WHERE id=?1 AND active=1)",
                [item.medicine_id.to_string()],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(Error::NotFound);
            }
            let sale_item_id = Uuid::now_v7();
            tx.execute("INSERT INTO sale_items(id,sale_id,medicine_id,quantity,unit_price) VALUES(?1,?2,?3,?4,?5)",params![sale_item_id.to_string(),id.to_string(),item.medicine_id.to_string(),item.quantity,decimal_to_db(item.unit_price)])?;
            let batches = {
                let mut stmt=tx.prepare("SELECT id,quantity_on_hand FROM inventory_batches WHERE medicine_id=?1 AND quantity_on_hand>0 AND expiry_date>=?2 ORDER BY expiry_date ASC,id ASC")?;
                stmt.query_map(
                    params![item.medicine_id.to_string(), date_to_db(bd_date(now))],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?)),
                )?
                .collect::<Result<Vec<_>, _>>()?
            };
            let mut remaining = item.quantity;
            for (batch_id, available) in batches {
                if remaining == 0 {
                    break;
                }
                let used = remaining.min(available);
                tx.execute(
                    "UPDATE inventory_batches SET quantity_on_hand=quantity_on_hand-?2 WHERE id=?1",
                    params![batch_id, used],
                )?;
                tx.execute("INSERT INTO sale_item_batches(sale_item_id,batch_id,quantity) VALUES(?1,?2,?3)",params![sale_item_id.to_string(),batch_id,used])?;
                tx.execute("INSERT INTO inventory_movements(id,operation_id,batch_id,quantity_delta,created_at) VALUES(?1,?2,?3,?4,?5)",params![Uuid::now_v7().to_string(),operation_id.to_string(),batch_id,-i64::from(used),datetime_to_db(now)])?;
                remaining -= used;
            }
            if remaining != 0 {
                return Err(Error::InsufficientStock);
            }
            items.push(SaleItem {
                id: sale_item_id,
                medicine_id: item.medicine_id,
                quantity: item.quantity,
                unit_price: item.unit_price,
            });
        }
        enqueue_sync(&tx, "sale", id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(Sale {
            id,
            operation_id,
            created_by: actor,
            created_at: now,
            items,
        })
    }
    pub fn reverse_sale(
        &mut self,
        pharmacy: &PharmacyAccount,
        operation_id: Uuid,
    ) -> Result<Uuid, Error> {
        let actor = pharmacy.id;
        let now = OffsetDateTime::now_utc();
        let reversal_id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        let sale_id: Option<String> = tx
            .query_row(
                "SELECT s.id FROM sales s JOIN operations o ON o.id=s.operation_id WHERE s.operation_id=?1 AND o.kind='sale'",
                [operation_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        let sale_id = sale_id.ok_or(Error::NotFound)?;
        let reversed: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE reversal_of=?1)",
            [operation_id.to_string()],
            |row| row.get(0),
        )?;
        if reversed != 0 {
            return Err(Error::AlreadyReversed);
        }
        let allocations = {
            let mut stmt=tx.prepare("SELECT sib.batch_id,sib.quantity FROM sale_item_batches sib JOIN sale_items si ON si.id=sib.sale_item_id WHERE si.sale_id=?1 ORDER BY si.rowid,sib.batch_id")?;
            stmt.query_map([&sale_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?
        };
        tx.execute("INSERT INTO operations(id,kind,actor,occurred_at,reversal_of,actor_id,actor_name) VALUES(?1,'sale_reversal',?2,?3,?4,?2,?5)",params![reversal_id.to_string(),actor.to_string(),datetime_to_db(now),operation_id.to_string(),pharmacy.username])?;
        for (batch_id, quantity) in allocations {
            tx.execute(
                "UPDATE inventory_batches SET quantity_on_hand=quantity_on_hand+?2 WHERE id=?1",
                params![batch_id, quantity],
            )?;
            tx.execute("INSERT INTO inventory_movements(id,operation_id,batch_id,quantity_delta,created_at) VALUES(?1,?2,?3,?4,?5)",params![Uuid::now_v7().to_string(),reversal_id.to_string(),batch_id,quantity,datetime_to_db(now)])?;
        }
        enqueue_sync(&tx, "sale_reversal", reversal_id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(reversal_id)
    }
}
