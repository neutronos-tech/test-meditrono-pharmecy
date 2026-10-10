use crate::catalog::Medicine;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub medicine_id: Uuid,
    pub medicine_name: String,
    pub generic_name: Option<String>,
    pub strength: Option<String>,
    pub dosage_form: Option<String>,
    pub manufacturer: Option<String>,
    pub total_quantity: u64,
    pub batch_count: u32,
    pub nearest_expiry: Option<Date>,
    pub selling_price: Option<Decimal>,
    pub status: InventoryStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryStatus {
    InStock,
    LowStock,
    NearExpiry,
    ExpiredStock,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryBatch {
    pub id: Uuid,
    pub batch_number: String,
    pub expiry_date: Date,
    pub quantity_on_hand: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryDetail {
    pub medicine: Medicine,
    pub total_quantity: u64,
    pub batches: Vec<InventoryBatch>,
}
