use crate::{
    Backend, Error,
    database::{bd_date, date_from_db, date_to_db, decimal_from_db, uuid_from_db},
};
use contracts::inventory::{InventoryBatch, InventoryDetail, InventoryItem, InventoryStatus};
use rusqlite::params;
use time::OffsetDateTime;
use uuid::Uuid;

impl Backend {
    pub fn inventory(&self, query: &str, limit: u32) -> Result<Vec<InventoryItem>, Error> {
        self.inventory_rows(query, limit, false)
    }
    pub fn sellable_inventory(&self, query: &str, limit: u32) -> Result<Vec<InventoryItem>, Error> {
        self.inventory_rows(query, limit, true)
    }
    fn inventory_rows(
        &self,
        query: &str,
        limit: u32,
        sellable: bool,
    ) -> Result<Vec<InventoryItem>, Error> {
        let today = bd_date(OffsetDateTime::now_utc());
        let sql = if sellable {
            "SELECT m.id,m.name,m.generic_name,m.strength,m.dosage_form,m.manufacturer,m.selling_price,COALESCE(SUM(b.quantity_on_hand),0),COUNT(b.id),MIN(b.expiry_date) FROM medicines m LEFT JOIN inventory_batches b ON b.medicine_id=m.id AND b.quantity_on_hand>0 AND b.expiry_date>=?1 WHERE m.active=1 AND (m.name LIKE ?2 OR m.generic_name LIKE ?2) GROUP BY m.id HAVING COALESCE(SUM(b.quantity_on_hand),0)>0 ORDER BY m.name LIMIT ?3"
        } else {
            "SELECT m.id,m.name,m.generic_name,m.strength,m.dosage_form,m.manufacturer,m.selling_price,COALESCE(SUM(b.quantity_on_hand),0),COUNT(b.id),MIN(b.expiry_date) FROM medicines m LEFT JOIN inventory_batches b ON b.medicine_id=m.id AND b.quantity_on_hand>0 WHERE (m.name LIKE ?2 OR m.generic_name LIKE ?2) GROUP BY m.id ORDER BY m.name LIMIT ?3"
        };
        let mut stmt = self.db.prepare(sql)?;
        let rows = stmt.query_map(
            params![date_to_db(today), format!("%{query}%"), limit],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, u64>(7)?,
                    row.get::<_, u32>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (
                id,
                name,
                generic_name,
                strength,
                dosage_form,
                manufacturer,
                price,
                total_quantity,
                batch_count,
                expiry,
            ) = row?;
            let nearest_expiry = expiry.as_deref().map(date_from_db).transpose()?;
            let status = match nearest_expiry {
                Some(d) if d < today => InventoryStatus::ExpiredStock,
                _ if total_quantity <= 10 => InventoryStatus::LowStock,
                Some(d) if d <= today + time::Duration::days(30) => InventoryStatus::NearExpiry,
                _ => InventoryStatus::InStock,
            };
            Ok(InventoryItem {
                medicine_id: uuid_from_db(id)?,
                medicine_name: name,
                generic_name,
                strength,
                dosage_form,
                manufacturer,
                total_quantity,
                batch_count,
                nearest_expiry,
                selling_price: price.as_deref().map(decimal_from_db).transpose()?,
                status,
            })
        })
        .collect()
    }
    pub fn inventory_detail(&self, medicine_id: Uuid) -> Result<InventoryDetail, Error> {
        let medicine = self.medicine_by_id(medicine_id)?;
        let mut stmt=self.db.prepare("SELECT id,batch_number,expiry_date,quantity_on_hand FROM inventory_batches WHERE medicine_id=?1 ORDER BY expiry_date,id")?;
        let rows = stmt.query_map([medicine_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
            ))
        })?;
        let batches = rows
            .map(|row| {
                let (id, batch_number, expiry_date, quantity_on_hand) = row?;
                Ok(InventoryBatch {
                    id: uuid_from_db(id)?,
                    batch_number,
                    expiry_date: date_from_db(&expiry_date)?,
                    quantity_on_hand,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let total_quantity = batches.iter().map(|b| b.quantity_on_hand).sum();
        Ok(InventoryDetail {
            medicine,
            total_quantity,
            batches,
        })
    }
}
