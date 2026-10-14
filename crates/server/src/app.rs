use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use contracts::auth::PharmacyAccount;
use contracts::catalog::{MedicineCatalogItem, MedicineChange};
use rand::{RngCore, rngs::OsRng};
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use sqlx_core::{query::query, row::Row};
use sqlx_postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub licence_key: Option<std::sync::Arc<crate::licence::LicenceKey>>,
    #[cfg(debug_assertions)]
    dev_token_hash: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionKind {
    Browser,
    Desktop,
}

impl SessionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Desktop => "desktop",
        }
    }
}

// This contains bearer material and deliberately does not implement Debug.
pub struct Authentication {
    pub pharmacy: PharmacyAccount,
    pub session_token: String,
    pub csrf_token: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum LoginError {
    InvalidCredentials,
    Database,
}

#[derive(Debug, Clone, Copy)]
pub enum AuthenticateError {
    Invalid,
    Csrf,
    Database,
}

impl AppState {
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            licence_key: None,
            #[cfg(debug_assertions)]
            dev_token_hash: None,
        }
    }

    /// Debug builds only: accept `token` as a bearer for the fixed developer
    /// account, creating that account if needed. Release builds have no such method.
    #[cfg(debug_assertions)]
    pub async fn enable_dev_token(&mut self, token: &str) -> Result<(), String> {
        if token.len() < contracts::auth::dev::MIN_TOKEN_LEN {
            return Err(format!(
                "{} must be at least {} characters",
                contracts::auth::dev::TOKEN_ENV,
                contracts::auth::dev::MIN_TOKEN_LEN
            ));
        }
        // '!' is not a valid password hash, so password login can never succeed for this account.
        query("INSERT INTO pharmacies(id,username,password_hash) VALUES($1,$2,'!') ON CONFLICT (id) DO NOTHING")
            .bind(contracts::auth::dev::ACCOUNT_ID)
            .bind(contracts::auth::dev::USERNAME)
            .execute(&self.pool)
            .await
            .map_err(|error| format!("could not create the developer account: {error}"))?;
        self.dev_token_hash = Some(hash_secret(token));
        tracing::warn!("developer token enabled (debug build only)");
        Ok(())
    }

    pub async fn connect(database_url: &str) -> Result<Self, sqlx_core::error::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;
        Ok(Self::from_pool(pool))
    }

    pub async fn bootstrap_pharmacy(&self, username: &str, password: &str) -> Result<(), String> {
        let username = username.trim();
        if username.is_empty() || password.is_empty() {
            return Err("Username and password must be non-empty.".into());
        }
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| "Could not hash the supplied password.".to_string())?
            .to_string();
        query("INSERT INTO pharmacies(id,username,password_hash) VALUES($1,$2,$3)")
            .bind(Uuid::now_v7())
            .bind(username)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(|_| "Could not create the pharmacy account.".to_string())?;
        Ok(())
    }

    pub async fn login_pharmacy(
        &self,
        username: &str,
        password: &str,
        kind: SessionKind,
    ) -> Result<Authentication, LoginError> {
        let username = username.trim();
        let row = query(
            "SELECT id, username, password_hash FROM pharmacies WHERE lower(username)=lower($1) AND active=TRUE",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| LoginError::Database)?
        .ok_or(LoginError::InvalidCredentials)?;
        let id: Uuid = row.try_get("id").map_err(|_| LoginError::Database)?;
        let canonical_username: String =
            row.try_get("username").map_err(|_| LoginError::Database)?;
        let password_hash: String = row
            .try_get("password_hash")
            .map_err(|_| LoginError::Database)?;
        let parsed = PasswordHash::new(&password_hash).map_err(|_| LoginError::Database)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| LoginError::InvalidCredentials)?;

        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        let session_token = URL_SAFE_NO_PAD.encode(secret);
        let token_hash = hash_secret(&session_token);
        let csrf_token = matches!(kind, SessionKind::Browser).then(random_token);
        let csrf_hash = csrf_token
            .as_deref()
            .map(hash_secret)
            .map(|value| value.to_vec());
        let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);
        query("INSERT INTO pharmacy_sessions(id,pharmacy_id,token_hash,csrf_hash,kind,created_at,expires_at) VALUES($1,$2,$3,$4,$5,now(),$6)")
            .bind(Uuid::now_v7())
            .bind(id)
            .bind(token_hash.to_vec())
            .bind(csrf_hash)
            .bind(kind.as_str())
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(|_| LoginError::Database)?;
        Ok(Authentication {
            pharmacy: PharmacyAccount {
                id,
                username: canonical_username,
            },
            session_token,
            csrf_token,
        })
    }

    pub async fn authenticate_session(
        &self,
        token: &str,
    ) -> Result<PharmacyAccount, AuthenticateError> {
        #[cfg(debug_assertions)]
        if let Some(expected) = self.dev_token_hash
            && constant_time_eq(&hash_secret(token), &expected)
        {
            return Ok(PharmacyAccount {
                id: contracts::auth::dev::ACCOUNT_ID,
                username: contracts::auth::dev::USERNAME.into(),
            });
        }
        let row = query("SELECT p.id,p.username FROM pharmacy_sessions s JOIN pharmacies p ON p.id=s.pharmacy_id WHERE s.token_hash=$1 AND s.expires_at>now() AND p.active=TRUE")
            .bind(hash_secret(token).to_vec())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthenticateError::Database)?
            .ok_or(AuthenticateError::Invalid)?;
        Ok(PharmacyAccount {
            id: row.try_get("id").map_err(|_| AuthenticateError::Database)?,
            username: row
                .try_get("username")
                .map_err(|_| AuthenticateError::Database)?,
        })
    }

    pub async fn authenticate_browser_session(
        &self,
        token: &str,
    ) -> Result<PharmacyAccount, AuthenticateError> {
        let row = query("SELECT p.id,p.username FROM pharmacy_sessions s JOIN pharmacies p ON p.id=s.pharmacy_id WHERE s.token_hash=$1 AND s.kind='browser' AND s.expires_at>now() AND p.active=TRUE")
            .bind(hash_secret(token).to_vec())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthenticateError::Database)?
            .ok_or(AuthenticateError::Invalid)?;
        Ok(PharmacyAccount {
            id: row.try_get("id").map_err(|_| AuthenticateError::Database)?,
            username: row
                .try_get("username")
                .map_err(|_| AuthenticateError::Database)?,
        })
    }

    pub async fn authenticate_browser_mutation(
        &self,
        token: &str,
        csrf: &str,
    ) -> Result<PharmacyAccount, AuthenticateError> {
        let pharmacy = self.authenticate_browser_session(token).await?;
        let stored: Option<Vec<u8>> = query("SELECT csrf_hash FROM pharmacy_sessions WHERE token_hash=$1 AND kind='browser' AND expires_at>now()")
            .bind(hash_secret(token).to_vec())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthenticateError::Database)?
            .and_then(|row| row.try_get("csrf_hash").ok().flatten());
        let candidate = hash_secret(csrf);
        if stored
            .as_deref()
            .is_none_or(|stored| !constant_time_eq(stored, &candidate))
        {
            return Err(AuthenticateError::Csrf);
        }
        Ok(pharmacy)
    }

    pub async fn logout_session(&self, token: &str) -> Result<(), AuthenticateError> {
        let result = query("DELETE FROM pharmacy_sessions WHERE token_hash=$1")
            .bind(hash_secret(token).to_vec())
            .execute(&self.pool)
            .await
            .map_err(|_| AuthenticateError::Database)?;
        if result.rows_affected() == 0 {
            return Err(AuthenticateError::Invalid);
        }
        Ok(())
    }

    /// One keyset page of the catalog, ordered by id. `after` is the last id
    /// of the previous page (None for the first page).
    pub async fn medicine_page(
        &self,
        after: Option<Uuid>,
        limit: i64,
    ) -> Result<(u64, u64, Vec<MedicineCatalogItem>), sqlx_core::error::Error> {
        let revision = self.medicine_revision().await?;
        let count: i64 = query("SELECT COUNT(*) AS count FROM medicine_catalog")
            .fetch_one(&self.pool)
            .await?
            .try_get("count")?;
        let rows = query("SELECT id,brand_name,generic_name,strength,dosage_form,manufacturer,package_description,reference_price_paisa,is_active FROM medicine_catalog WHERE ($1::uuid IS NULL OR id > $1) ORDER BY id LIMIT $2")
            .bind(after)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        let medicines = rows
            .into_iter()
            .map(|row| {
                let paisa: Option<i64> = row.try_get("reference_price_paisa")?;
                Ok(MedicineCatalogItem {
                    id: row.try_get("id")?,
                    name: row.try_get("brand_name")?,
                    generic_name: row.try_get("generic_name")?,
                    strength: row.try_get("strength")?,
                    dosage_form: row.try_get("dosage_form")?,
                    manufacturer: row.try_get("manufacturer")?,
                    package_description: row.try_get("package_description")?,
                    reference_price: paisa.map(|value| Decimal::new(value, 2)),
                    active: row.try_get("is_active")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx_core::error::Error>>()?;
        let count = u64::try_from(count)
            .map_err(|_| sqlx_core::error::Error::Protocol("negative medicine count".into()))?;
        Ok((revision, count, medicines))
    }

    pub async fn medicine_changes(
        &self,
        after: u64,
    ) -> Result<(u64, Vec<MedicineChange>), sqlx_core::error::Error> {
        let current = self.medicine_revision().await?;
        let after = i64::try_from(after).map_err(|_| {
            sqlx_core::error::Error::Protocol("revision is outside PostgreSQL range".into())
        })?;
        let rows = query("SELECT c.revision,m.id,m.brand_name,m.generic_name,m.strength,m.dosage_form,m.manufacturer,m.package_description,m.reference_price_paisa,m.is_active FROM medicine_catalog_changes c JOIN medicine_catalog m ON m.id=c.medicine_id WHERE c.revision>$1 AND c.revision<=$2 ORDER BY c.revision")
            .bind(after)
            .bind(i64::try_from(current).map_err(|_| sqlx_core::error::Error::Protocol("revision is outside PostgreSQL range".into()))?)
            .fetch_all(&self.pool)
            .await?;
        let changes = rows
            .into_iter()
            .map(|row| {
                let paisa: Option<i64> = row.try_get("reference_price_paisa")?;
                let medicine = MedicineCatalogItem {
                    id: row.try_get("id")?,
                    name: row.try_get("brand_name")?,
                    generic_name: row.try_get("generic_name")?,
                    strength: row.try_get("strength")?,
                    dosage_form: row.try_get("dosage_form")?,
                    manufacturer: row.try_get("manufacturer")?,
                    package_description: row.try_get("package_description")?,
                    reference_price: paisa.map(|value| Decimal::new(value, 2)),
                    active: row.try_get("is_active")?,
                };
                Ok(MedicineChange {
                    revision: u64::try_from(row.try_get::<i64, _>("revision")?).map_err(|_| {
                        sqlx_core::error::Error::Protocol("negative revision".into())
                    })?,
                    medicine,
                })
            })
            .collect::<Result<Vec<_>, sqlx_core::error::Error>>()?;
        Ok((current, changes))
    }

    async fn medicine_revision(&self) -> Result<u64, sqlx_core::error::Error> {
        let revision: i64 =
            query("SELECT COALESCE(MAX(revision),0) AS revision FROM medicine_catalog_changes")
                .fetch_one(&self.pool)
                .await?
                .try_get("revision")?;
        u64::try_from(revision)
            .map_err(|_| sqlx_core::error::Error::Protocol("negative revision".into()))
    }
}

fn random_token() -> String {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    URL_SAFE_NO_PAD.encode(secret)
}

fn hash_secret(secret: &str) -> [u8; 32] {
    Sha256::digest(secret.as_bytes()).into()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}
