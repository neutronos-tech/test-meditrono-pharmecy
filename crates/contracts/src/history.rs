use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    Purchase,
    StockReceipt,
    StockReceiptReversal,
    Sale,
    SaleReversal,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub operation_id: Uuid,
    pub kind: OperationKind,
    pub actor_name: String,
    pub occurred_at: OffsetDateTime,
    pub reversal_of: Option<Uuid>,
    pub description: String,
    pub reversible: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryFilter {
    All,
    Purchases,
    Stock,
    Sales,
    Reversals,
}
