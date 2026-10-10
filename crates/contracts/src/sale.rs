use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleInput {
    pub items: Vec<SaleItemInput>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItemInput {
    pub medicine_id: Uuid,
    pub quantity: u32,
    pub unit_price: Decimal,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
    pub items: Vec<SaleItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    pub id: Uuid,
    pub medicine_id: Uuid,
    pub quantity: u32,
    pub unit_price: Decimal,
}
