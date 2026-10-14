//! Signed device licences. The server signs a short-lived licence for one
//! device; the desktop app verifies it offline with a public key built into
//! the app and decides whether it may record new work.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// How long one issued licence lasts before the device must reconnect.
pub const LICENCE_DAYS: i64 = 30;
/// After expiry the app stays usable read-only, never locked.
pub const GRACE_DAYS: i64 = 7;
/// Warn the user this many days before expiry.
pub const WARN_DAYS: i64 = 7;
/// Clock drift tolerated before treating time as rolled back.
pub const CLOCK_TOLERANCE: Duration = Duration::hours(1);
/// Shop PC clocks are often wrong, so allow a wide gap to the server's issue
/// time; the app's own `latest_seen` still catches a clock turned back later.
pub const ISSUE_CLOCK_TOLERANCE: Duration = Duration::days(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicencePayload {
    pub key_id: u8,
    pub pharmacy_id: Uuid,
    pub device_id: i64,
    /// The device fingerprint the app sent, so a copied licence fails elsewhere.
    pub device_fingerprint: String,
    pub plan: String,
    #[serde(with = "time::serde::timestamp")]
    pub issued_at: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    pub expires_at: OffsetDateTime,
}

/// The wire and storage form: base64 JSON payload plus its Ed25519 signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedLicence {
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenceRenewInput {
    pub device_fingerprint: String,
    pub device_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenceState {
    Active {
        days_left: i64,
    },
    /// Expired but inside the grace window: view and export only.
    ReadOnly,
    /// Grace window over too; the app still opens read-only and asks to renew.
    Expired,
}

impl LicenceState {
    pub fn can_record(self) -> bool {
        matches!(self, Self::Active { .. })
    }

    pub fn should_warn(self) -> bool {
        matches!(self, Self::Active { days_left } if days_left <= WARN_DAYS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenceError {
    Malformed,
    UnknownKey,
    BadSignature,
    WrongDevice,
    ClockRolledBack,
}

/// One public key the app trusts. Several are kept during key rotation.
pub struct TrustedKey {
    pub key_id: u8,
    pub public_key: [u8; 32],
}

pub fn sign_payload(payload: &LicencePayload, key: &ed25519_dalek::SigningKey) -> SignedLicence {
    use ed25519_dalek::Signer;
    let json = serde_json::to_vec(payload).expect("licence payload serializes");
    let signature = key.sign(&json);
    SignedLicence {
        payload: URL_SAFE_NO_PAD.encode(json),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    }
}

/// Checks signature and device, and returns the payload.
pub fn verify(
    licence: &SignedLicence,
    trusted: &[TrustedKey],
    device_fingerprint: &str,
) -> Result<LicencePayload, LicenceError> {
    let json = URL_SAFE_NO_PAD
        .decode(&licence.payload)
        .map_err(|_| LicenceError::Malformed)?;
    let signature: [u8; 64] = URL_SAFE_NO_PAD
        .decode(&licence.signature)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(LicenceError::Malformed)?;
    let payload: LicencePayload =
        serde_json::from_slice(&json).map_err(|_| LicenceError::Malformed)?;
    let key = trusted
        .iter()
        .find(|key| key.key_id == payload.key_id)
        .ok_or(LicenceError::UnknownKey)?;
    let key = VerifyingKey::from_bytes(&key.public_key).map_err(|_| LicenceError::UnknownKey)?;
    key.verify(&json, &Signature::from_bytes(&signature))
        .map_err(|_| LicenceError::BadSignature)?;
    if payload.device_fingerprint != device_fingerprint {
        return Err(LicenceError::WrongDevice);
    }
    Ok(payload)
}

/// Decides what the app may do now. `latest_seen` is the latest time the app
/// has recorded; a clock earlier than that means the clock was turned back.
pub fn evaluate(
    payload: &LicencePayload,
    now: OffsetDateTime,
    latest_seen: OffsetDateTime,
) -> Result<LicenceState, LicenceError> {
    if now + CLOCK_TOLERANCE < latest_seen || now + ISSUE_CLOCK_TOLERANCE < payload.issued_at {
        return Err(LicenceError::ClockRolledBack);
    }
    if now < payload.expires_at {
        let days_left = (payload.expires_at - now).whole_days();
        return Ok(LicenceState::Active { days_left });
    }
    if now < payload.expires_at + Duration::days(GRACE_DAYS) {
        return Ok(LicenceState::ReadOnly);
    }
    Ok(LicenceState::Expired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn fixture() -> (SigningKey, Vec<TrustedKey>, LicencePayload) {
        let key = SigningKey::from_bytes(&[7; 32]);
        let trusted = vec![TrustedKey {
            key_id: 1,
            public_key: key.verifying_key().to_bytes(),
        }];
        let issued_at = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let payload = LicencePayload {
            key_id: 1,
            pharmacy_id: Uuid::nil(),
            device_id: 1,
            device_fingerprint: "device-a".into(),
            plan: "standard".into(),
            issued_at,
            expires_at: issued_at + Duration::days(LICENCE_DAYS),
        };
        (key, trusted, payload)
    }

    #[test]
    fn valid_licence_verifies_and_is_active() {
        let (key, trusted, payload) = fixture();
        let signed = sign_payload(&payload, &key);
        let verified = verify(&signed, &trusted, "device-a").unwrap();
        let state = evaluate(&verified, payload.issued_at, payload.issued_at).unwrap();
        assert_eq!(state, LicenceState::Active { days_left: 30 });
        assert!(state.can_record());
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let (key, trusted, mut payload) = fixture();
        let signed = sign_payload(&payload, &key);
        payload.expires_at += Duration::days(365);
        let forged = SignedLicence {
            payload: URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap()),
            signature: signed.signature,
        };
        assert_eq!(
            verify(&forged, &trusted, "device-a"),
            Err(LicenceError::BadSignature)
        );
    }

    #[test]
    fn copied_licence_fails_on_another_device() {
        let (key, trusted, payload) = fixture();
        let signed = sign_payload(&payload, &key);
        assert_eq!(
            verify(&signed, &trusted, "device-b"),
            Err(LicenceError::WrongDevice)
        );
    }

    #[test]
    fn expiry_moves_to_read_only_then_expired() {
        let (_, _, payload) = fixture();
        let after = payload.expires_at + Duration::days(1);
        assert_eq!(evaluate(&payload, after, after), Ok(LicenceState::ReadOnly));
        let later = payload.expires_at + Duration::days(GRACE_DAYS + 1);
        assert_eq!(evaluate(&payload, later, later), Ok(LicenceState::Expired));
        assert!(!LicenceState::ReadOnly.can_record());
    }

    #[test]
    fn slightly_wrong_pc_clock_still_accepts_a_fresh_licence() {
        let (_, _, payload) = fixture();
        let behind = payload.issued_at - Duration::hours(3);
        assert!(evaluate(&payload, behind, behind).unwrap().can_record());
    }

    #[test]
    fn clock_rollback_is_detected() {
        let (_, _, payload) = fixture();
        let seen = payload.issued_at + Duration::days(40);
        assert_eq!(
            evaluate(&payload, payload.issued_at + Duration::days(5), seen),
            Err(LicenceError::ClockRolledBack)
        );
    }
}
