use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Debug builds only. These do not exist in release builds, so a release
/// server cannot recognise the developer token.
#[cfg(debug_assertions)]
pub mod dev {
    use uuid::Uuid;

    pub const ACCOUNT_ID: Uuid = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00de_0001);
    pub const USERNAME: &str = "developer";
    /// Environment variable holding the token. Must be at least 16 characters.
    pub const TOKEN_ENV: &str = "PHARMACY_DEV_TOKEN";
    pub const MIN_TOKEN_LEN: usize = 16;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacyAccount {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharmacyLoginInput {
    pub username: String,
    pub password: String,
}

// Intentionally does not implement Debug: the session token is a bearer secret.
#[derive(Clone, Serialize, Deserialize)]
pub struct PharmacyLoginResponse {
    pub pharmacy: PharmacyAccount,
    pub session_token: String,
}
