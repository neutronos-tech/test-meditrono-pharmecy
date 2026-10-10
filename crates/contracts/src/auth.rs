use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
