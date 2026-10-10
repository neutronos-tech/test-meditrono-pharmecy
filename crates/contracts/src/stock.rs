use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockReceiptInput {
    pub purchase_id: Uuid,
    pub lines: Vec<StockReceiptLineInput>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockReceiptLineInput {
    pub purchase_item_id: Uuid,
    pub quantity: u32,
    pub batch_number: String,
    pub expiry_date: Date,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivablePurchase {
    pub purchase_id: Uuid,
    pub invoice_number: String,
    pub supplier_name: String,
    pub created_at: OffsetDateTime,
    pub remaining_items: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivablePurchaseDetail {
    pub purchase_id: Uuid,
    pub invoice_number: String,
    pub supplier_name: String,
    pub items: Vec<ReceivablePurchaseItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivablePurchaseItem {
    pub purchase_item_id: Uuid,
    pub medicine_id: Uuid,
    pub medicine_name: String,
    pub ordered: u32,
    pub received: u32,
    pub remaining: u32,
    pub unit_cost: Decimal,
}
