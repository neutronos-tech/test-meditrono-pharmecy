//! Public SaaS endpoints shared by the server and the web app.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub code: String,
    pub name: String,
    pub monthly_price_paisa: i64,
    pub yearly_price_paisa: i64,
    pub trial_days: i32,
    pub ai_scan_cap: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub paid_until: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_seen_at: OffsetDateTime,
}
