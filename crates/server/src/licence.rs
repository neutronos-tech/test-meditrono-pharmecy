//! Issues signed 30-day licences per device. A device's own `paid_until`
//! caps the licence, so an unpaid device cannot get free days.

use crate::{app::AppState, error::AppError};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use contracts::{
    auth::PharmacyAccount,
    licence::{LICENCE_DAYS, LicencePayload, LicenceRenewInput, SignedLicence, sign_payload},
};
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};
use sqlx_core::{query::query, row::Row};
use time::{Duration, OffsetDateTime};

/// Format: `<key-id>:<base64url 32-byte seed>`. Keep it only in server.env.
pub const KEY_ENV: &str = "PHARMACY_LICENCE_KEY";

pub struct LicenceKey {
    pub key_id: u8,
    signing: SigningKey,
}

impl LicenceKey {
    pub fn parse(value: &str) -> Result<Self, String> {
        let error = || format!("{KEY_ENV} must look like 1:<base64 seed>");
        let (id, seed) = value.trim().split_once(':').ok_or_else(error)?;
        let seed: [u8; 32] = URL_SAFE_NO_PAD
            .decode(seed)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(error)?;
        Ok(Self {
            key_id: id.parse().map_err(|_| error())?,
            signing: SigningKey::from_bytes(&seed),
        })
    }

    #[cfg(test)]
    pub fn from_seed(key_id: u8, seed: [u8; 32]) -> Self {
        Self {
            key_id,
            signing: SigningKey::from_bytes(&seed),
        }
    }

    /// Prints a new env value and the public key to build into the app.
    pub fn generate_env_value(key_id: u8) -> String {
        let signing = SigningKey::generate(&mut rand::rngs::OsRng);
        format!(
            "{KEY_ENV}={key_id}:{}\npublic key (build into the app): {}",
            URL_SAFE_NO_PAD.encode(signing.to_bytes()),
            URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes())
        )
    }

    #[cfg(test)]
    pub fn public_key(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }
}

pub fn fingerprint_hash(fingerprint: &str) -> String {
    let digest = Sha256::digest(fingerprint.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl AppState {
    pub async fn renew_licence(
        &self,
        pharmacy: &PharmacyAccount,
        input: LicenceRenewInput,
    ) -> Result<SignedLicence, AppError> {
        let key = self.licence_key.as_ref().ok_or(AppError::Unavailable)?;
        let fingerprint = input.device_fingerprint.trim();
        let name = input.device_name.trim();
        if fingerprint.is_empty() || fingerprint.len() > 256 || name.is_empty() || name.len() > 100
        {
            return Err(AppError::BadRequest);
        }
        let hash = fingerprint_hash(fingerprint);
        let now = OffsetDateTime::now_utc();
        let mut tx = self.pool.begin().await?;
        // Serialise registrations per pharmacy so two new devices cannot both
        // claim the one free trial.
        query("SELECT id FROM pharmacies WHERE id=$1 FOR UPDATE")
            .bind(pharmacy.id)
            .execute(&mut *tx)
            .await?;
        let existing = query(
            "SELECT d.id,d.paid_until,d.active,p.code FROM pharmacy_devices d \
             JOIN pharmacy_plans p ON p.id=d.plan_id \
             WHERE d.pharmacy_id=$1 AND d.fingerprint_hash=$2",
        )
        .bind(pharmacy.id)
        .bind(&hash)
        .fetch_optional(&mut *tx)
        .await?;
        let (device_id, paid_until, plan) = match existing {
            Some(row) => {
                if !row.try_get::<bool, _>("active")? {
                    return Err(AppError::Forbidden);
                }
                (
                    row.try_get::<i64, _>("id")?,
                    row.try_get::<OffsetDateTime, _>("paid_until")?,
                    row.try_get::<String, _>("code")?,
                )
            }
            None => {
                // Only a pharmacy's first device gets the free trial; every
                // other device is paid for (or activated) before it works.
                let plan = query(
                    "SELECT id,code,trial_days FROM pharmacy_plans WHERE is_default AND active",
                )
                .fetch_one(&mut *tx)
                .await?;
                let has_devices: bool = query(
                    "SELECT EXISTS(SELECT 1 FROM pharmacy_devices WHERE pharmacy_id=$1) AS any",
                )
                .bind(pharmacy.id)
                .fetch_one(&mut *tx)
                .await?
                .try_get("any")?;
                let trial = if has_devices {
                    0
                } else {
                    plan.try_get::<i32, _>("trial_days")?
                };
                let paid_until = now + Duration::days(trial.into());
                let id: i64 = query(
                    "INSERT INTO pharmacy_devices(pharmacy_id,fingerprint_hash,name,plan_id,paid_until) \
                     VALUES($1,$2,$3,$4,$5) RETURNING id",
                )
                .bind(pharmacy.id)
                .bind(&hash)
                .bind(name)
                .bind(plan.try_get::<i64, _>("id")?)
                .bind(paid_until)
                .fetch_one(&mut *tx)
                .await?
                .try_get("id")?;
                audit(&mut tx, pharmacy, "device.registered", id, trial).await?;
                (id, paid_until, plan.try_get::<String, _>("code")?)
            }
        };
        query("UPDATE pharmacy_devices SET last_seen_at=now() WHERE id=$1")
            .bind(device_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        if paid_until <= now {
            return Err(AppError::PaymentRequired);
        }
        let payload = LicencePayload {
            key_id: key.key_id,
            pharmacy_id: pharmacy.id,
            device_id,
            device_fingerprint: fingerprint.to_owned(),
            plan,
            issued_at: now,
            expires_at: paid_until.min(now + Duration::days(LICENCE_DAYS)),
        };
        Ok(sign_payload(&payload, &key.signing))
    }

    /// Manual activation (until the payment gateway is live): adds `days`
    /// to a device, counting from today if it had already lapsed.
    pub async fn extend_device(
        &self,
        device_id: i64,
        days: i64,
        source: &str,
    ) -> Result<OffsetDateTime, String> {
        if !(1..=3660).contains(&days) {
            return Err("days must be between 1 and 3660".into());
        }
        let row = query(
            "UPDATE pharmacy_devices SET paid_until=GREATEST(paid_until,now())+make_interval(days=>$2) \
             WHERE id=$1 RETURNING pharmacy_id,paid_until",
        )
        .bind(device_id)
        .bind(i32::try_from(days).map_err(|_| "days out of range")?)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("Could not extend the device: {error}"))?
        .ok_or("No device has that id.")?;
        let paid_until: OffsetDateTime = row.try_get("paid_until").map_err(|e| e.to_string())?;
        query(
            "INSERT INTO audit_log(pharmacy_id,actor,action,detail,source) \
             VALUES($1,'owner','device.extended',jsonb_build_object('device_id',$2::bigint,'days',$3::bigint),$4)",
        )
        .bind(row.try_get::<uuid::Uuid, _>("pharmacy_id").map_err(|e| e.to_string())?)
        .bind(device_id)
        .bind(days)
        .bind(source)
        .execute(&self.pool)
        .await
        .map_err(|error| format!("Could not write the audit log: {error}"))?;
        Ok(paid_until)
    }
}

async fn audit(
    tx: &mut sqlx_core::transaction::Transaction<'_, sqlx_postgres::Postgres>,
    pharmacy: &PharmacyAccount,
    action: &str,
    device_id: i64,
    trial_days: i32,
) -> Result<(), AppError> {
    query(
        "INSERT INTO audit_log(pharmacy_id,actor,action,detail,source) \
         VALUES($1,$2,$3,jsonb_build_object('device_id',$4::bigint,'trial_days',$5::int),'api')",
    )
    .bind(pharmacy.id)
    .bind(&pharmacy.username)
    .bind(action)
    .bind(device_id)
    .bind(trial_days)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
