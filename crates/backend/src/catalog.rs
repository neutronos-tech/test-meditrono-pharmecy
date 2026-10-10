use crate::{
    Backend, Error,
    database::{decimal_from_db, decimal_to_db, uuid_from_db},
    sync::enqueue_sync,
};
use contracts::sync::SyncOperation;
use contracts::{
    auth::PharmacyAccount,
    catalog::{CreateMedicineInput, CreateSupplierInput, Medicine, SetSellingPriceInput, Supplier},
};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

type MedicineRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

fn medicine_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MedicineRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
    ))
}
fn medicine_from_tuple(row: MedicineRow) -> Result<Medicine, Error> {
    Ok(Medicine {
        id: uuid_from_db(row.0)?,
        name: row.1,
        generic_name: row.2,
        strength: row.3,
        dosage_form: row.4,
        manufacturer: row.5,
        package_description: row.6,
        reference_price: row.7.as_deref().map(decimal_from_db).transpose()?,
        selling_price: row.8.as_deref().map(decimal_from_db).transpose()?,
        active: row.9 != 0,
    })
}
impl Backend {
    pub fn create_supplier(
        &mut self,
        _pharmacy: &PharmacyAccount,
        input: CreateSupplierInput,
    ) -> Result<Supplier, Error> {
        let id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO suppliers(id,name) VALUES(?1,?2)",
            params![id.to_string(), input.name],
        )?;
        enqueue_sync(&tx, "supplier", id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(Supplier {
            id,
            name: input.name,
        })
    }
    pub fn create_medicine(
        &mut self,
        _pharmacy: &PharmacyAccount,
        input: CreateMedicineInput,
    ) -> Result<Medicine, Error> {
        let id = Uuid::now_v7();
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO medicines(id,name) VALUES(?1,?2)",
            params![id.to_string(), input.name],
        )?;
        enqueue_sync(&tx, "medicine", id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(Medicine {
            id,
            name: input.name,
            generic_name: None,
            strength: None,
            dosage_form: None,
            manufacturer: None,
            package_description: None,
            reference_price: None,
            selling_price: None,
            active: true,
        })
    }
    pub fn search_suppliers(&self, query: &str, limit: u32) -> Result<Vec<Supplier>, Error> {
        let mut stmt = self
            .db
            .prepare("SELECT id,name FROM suppliers WHERE name LIKE ?1 ORDER BY name LIMIT ?2")?;
        let rows = stmt.query_map(params![format!("%{query}%"), limit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.map(|row| {
            let (id, name) = row?;
            Ok(Supplier {
                id: uuid_from_db(id)?,
                name,
            })
        })
        .collect()
    }
    pub fn search_medicines(&self, query: &str, limit: u32) -> Result<Vec<Medicine>, Error> {
        let mut stmt=self.db.prepare("SELECT id,name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price,selling_price,active FROM medicines WHERE active=1 AND (name LIKE ?1 OR generic_name LIKE ?1) ORDER BY name LIMIT ?2")?;
        let rows = stmt.query_map(params![format!("%{query}%"), limit], medicine_from_row)?;
        rows.map(|row| medicine_from_tuple(row?)).collect()
    }
    pub fn set_selling_price(
        &mut self,
        _pharmacy: &PharmacyAccount,
        input: SetSellingPriceInput,
    ) -> Result<(), Error> {
        let price = input.selling_price.map(decimal_to_db);
        let tx = self.db.transaction()?;
        if tx.execute(
            "UPDATE medicines SET selling_price=?2 WHERE id=?1",
            params![input.medicine_id.to_string(), price],
        )? == 0
        {
            return Err(Error::NotFound);
        }
        enqueue_sync(&tx, "medicine", input.medicine_id, SyncOperation::Upsert)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn medicine_by_id(&self, id: Uuid) -> Result<Medicine, Error> {
        let row=self.db.query_row("SELECT id,name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price,selling_price,active FROM medicines WHERE id=?1", [id.to_string()], medicine_from_row).optional()?.ok_or(Error::NotFound)?;
        medicine_from_tuple(row)
    }
}
