use crate::{Backend, Error, database::uuid_from_db};
use contracts::auth::PharmacyLoginResponse;
use rusqlite::OptionalExtension;

/// Locally persisted central-server credentials. `session_token` is sensitive
/// bearer credential material and must never be logged or formatted.
#[derive(Clone)]
pub struct SavedRemoteAuth {
    pub pharmacy: contracts::auth::PharmacyAccount,
    pub session_token: String,
}

impl std::fmt::Debug for SavedRemoteAuth {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SavedRemoteAuth")
            .field("pharmacy", &self.pharmacy)
            .field("session_token", &"[REDACTED]")
            .finish()
    }
}

impl Backend {
    pub fn saved_remote_auth(&self) -> Result<Option<SavedRemoteAuth>, Error> {
        let row = self
            .db
            .query_row(
                "SELECT pharmacy_id, username, session_token FROM remote_auth WHERE singleton=1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        row.map(|(id, username, session_token)| {
            Ok(SavedRemoteAuth {
                pharmacy: contracts::auth::PharmacyAccount {
                    id: uuid_from_db(id)?,
                    username,
                },
                session_token,
            })
        })
        .transpose()
    }

    pub fn save_remote_auth(&mut self, auth: &PharmacyLoginResponse) -> Result<(), Error> {
        self.db.execute(
            "INSERT INTO remote_auth(singleton, pharmacy_id, username, session_token) VALUES(1, ?1, ?2, ?3) ON CONFLICT(singleton) DO UPDATE SET pharmacy_id=excluded.pharmacy_id, username=excluded.username, session_token=excluded.session_token",
            rusqlite::params![auth.pharmacy.id.to_string(), auth.pharmacy.username, auth.session_token],
        )?;
        Ok(())
    }

    pub fn clear_remote_auth(&mut self) -> Result<(), Error> {
        self.db
            .execute("DELETE FROM remote_auth WHERE singleton=1", [])?;
        Ok(())
    }

    pub fn set_saved_pharmacy(
        &mut self,
        pharmacy: &contracts::auth::PharmacyAccount,
    ) -> Result<(), Error> {
        self.db.execute(
            "UPDATE remote_auth SET pharmacy_id=?1, username=?2 WHERE singleton=1",
            rusqlite::params![pharmacy.id.to_string(), pharmacy.username],
        )?;
        Ok(())
    }
}
