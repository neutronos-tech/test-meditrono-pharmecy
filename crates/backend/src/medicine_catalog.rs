use crate::{Backend, Error, database::decimal_to_db};
use contracts::catalog::{MedicineCatalogItem, MedicineChange};
use rusqlite::{Transaction, params};

fn upsert_medicine(tx: &Transaction<'_>, medicine: &MedicineCatalogItem) -> Result<(), Error> {
    tx.execute(
        "INSERT INTO medicines (
            id, name, generic_name, strength, dosage_form, manufacturer,
            package_description, reference_price, active, origin
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'global')
        ON CONFLICT(id) DO UPDATE SET
            name=excluded.name,
            generic_name=excluded.generic_name,
            strength=excluded.strength,
            dosage_form=excluded.dosage_form,
            manufacturer=excluded.manufacturer,
            package_description=excluded.package_description,
            reference_price=excluded.reference_price,
            active=excluded.active,
            origin='global'",
        params![
            medicine.id.to_string(),
            medicine.name,
            medicine.generic_name,
            medicine.strength,
            medicine.dosage_form,
            medicine.manufacturer,
            medicine.package_description,
            medicine.reference_price.map(decimal_to_db),
            medicine.active,
        ],
    )?;
    Ok(())
}

fn update_revision(tx: &Transaction<'_>, revision: u64) -> Result<(), Error> {
    let revision = i64::try_from(revision).map_err(|_| Error::Conflict)?;
    let changed = tx.execute(
        "UPDATE sync_state SET medicine_revision=?1 WHERE singleton=1 AND medicine_revision<=?1",
        [revision],
    )?;
    if changed == 0 {
        let current: i64 = tx.query_row(
            "SELECT medicine_revision FROM sync_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        if revision < current {
            return Err(Error::Conflict);
        }
    }
    Ok(())
}

impl Backend {
    pub fn has_global_medicines(&self) -> Result<bool, Error> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM medicines WHERE origin='global')",
            [],
            |row| row.get::<_, bool>(0),
        )?)
    }

    pub fn import_medicine_catalog(
        &mut self,
        medicines: Vec<MedicineCatalogItem>,
        revision: u64,
    ) -> Result<(), Error> {
        let tx = self.db.transaction()?;
        for medicine in &medicines {
            upsert_medicine(&tx, medicine)?;
        }
        update_revision(&tx, revision)?;
        tx.commit()?;
        Ok(())
    }

    pub fn apply_medicine_changes(
        &mut self,
        changes: Vec<MedicineChange>,
        revision: u64,
    ) -> Result<(), Error> {
        let tx = self.db.transaction()?;
        for change in &changes {
            if change.revision > revision {
                return Err(Error::Conflict);
            }
            upsert_medicine(&tx, &change.medicine)?;
        }
        update_revision(&tx, revision)?;
        tx.commit()?;
        Ok(())
    }
}
