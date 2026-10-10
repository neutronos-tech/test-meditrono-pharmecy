use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseInput {
    pub supplier_id: Uuid,
    pub invoice_number: String,
    pub items: Vec<PurchaseItemInput>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseItemInput {
    pub medicine_id: Uuid,
    pub quantity: u32,
    pub unit_cost: Decimal,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub supplier_id: Uuid,
    pub invoice_number: String,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub items: Vec<PurchaseItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub id: Uuid,
    pub medicine_id: Uuid,
    pub quantity: u32,
    pub unit_cost: Decimal,
}
