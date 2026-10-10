use crate::{
    Backend, Error,
    database::{datetime_to_db, uuid_from_db},
};
use contracts::sync::{PendingSync, SyncOperation};
use rusqlite::{Transaction, params};
use time::OffsetDateTime;
use uuid::Uuid;

pub(crate) fn enqueue_sync(
    tx: &Transaction<'_>,
    entity_type: &str,
    entity_id: Uuid,
    operation: SyncOperation,
) -> Result<(), Error> {
    tx.execute("INSERT INTO sync_outbox(change_id,entity_type,entity_id,operation,created_at) VALUES(?1,?2,?3,?4,?5)",
        params![Uuid::now_v7().to_string(),entity_type,entity_id.to_string(),match operation { SyncOperation::Upsert => "upsert", SyncOperation::Delete => "delete" },datetime_to_db(OffsetDateTime::now_utc())])?;
    Ok(())
}
impl Backend {
    pub fn pending_sync(&self, limit: u32) -> Result<Vec<PendingSync>, Error> {
        let mut stmt=self.db.prepare("SELECT seq,change_id,entity_type,entity_id,operation FROM sync_outbox ORDER BY seq LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (seq, change_id, entity_type, entity_id, operation) = row?;
            Ok(PendingSync {
                seq,
                change_id: uuid_from_db(change_id)?,
                entity_type,
                entity_id: uuid_from_db(entity_id)?,
                operation: match operation.as_str() {
                    "upsert" => SyncOperation::Upsert,
                    "delete" => SyncOperation::Delete,
                    _ => return Err(Error::InvalidStoredData),
                },
            })
        })
        .collect()
    }
    pub fn acknowledge_sync(&mut self, change_ids: &[Uuid]) -> Result<(), Error> {
        let tx = self.db.transaction()?;
        for id in change_ids {
            tx.execute(
                "DELETE FROM sync_outbox WHERE change_id=?1",
                [id.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn medicine_revision(&self) -> Result<u64, Error> {
        let value: i64 = self.db.query_row(
            "SELECT medicine_revision FROM sync_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        u64::try_from(value).map_err(|_| Error::InvalidStoredData)
    }
    pub fn set_medicine_revision(&mut self, revision: u64) -> Result<(), Error> {
        let value = i64::try_from(revision).map_err(|_| Error::Conflict)?;
        let changed = self.db.execute(
            "UPDATE sync_state SET medicine_revision=?1 WHERE singleton=1 AND medicine_revision<=?1",
            [value],
        )?;
        if changed == 0 {
            let current: i64 = self.db.query_row(
                "SELECT medicine_revision FROM sync_state WHERE singleton=1",
                [],
                |row| row.get(0),
            )?;
            if value < current {
                return Err(Error::Conflict);
            }
        }
        Ok(())
    }

    pub fn pharmacy_revision(&self) -> Result<u64, Error> {
        let revision: i64 = self.db.query_row(
            "SELECT pharmacy_revision FROM sync_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        u64::try_from(revision).map_err(|_| Error::InvalidStoredData)
    }

    pub fn set_pharmacy_revision(&mut self, revision: u64) -> Result<(), Error> {
        let value = i64::try_from(revision).map_err(|_| Error::Conflict)?;
        let changed = self.db.execute(
            "UPDATE sync_state SET pharmacy_revision=?1 WHERE singleton=1 AND pharmacy_revision<=?1",
            [value],
        )?;
        if changed == 0 {
            let current: i64 = self.db.query_row(
                "SELECT pharmacy_revision FROM sync_state WHERE singleton=1",
                [],
                |row| row.get(0),
            )?;
            if value < current {
                return Err(Error::Conflict);
            }
        }
        Ok(())
    }
}
