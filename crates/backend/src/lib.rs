mod auth;
mod catalog;
mod dashboard;
mod database;
mod error;
mod history;
mod inventory;
mod medicine_catalog;
mod pharmacy_sync;
mod purchase;
mod sale;
mod server_client;
mod stock;
mod sync;
mod sync_once;

pub use auth::SavedRemoteAuth;
pub use error::Error;
use rusqlite::Connection;
pub use server_client::{FullMedicineDownload, MedicineChangesDownload, RemoteError, ServerClient};
use std::path::Path;
pub use sync_once::{SyncError, sync_with_server};

pub struct Backend {
    db: Connection,
}
impl Backend {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut db = Connection::open(path)?;
        db.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
        )?;
        database::migrate(&mut db)?;
        Ok(Self { db })
    }
}

#[cfg(test)]
mod tests;
