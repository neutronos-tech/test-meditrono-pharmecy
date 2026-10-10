use crate::{
    catalog::{Medicine, Supplier},
    history::OperationKind,
    purchase::{Purchase, PurchaseItem},
    sale::Sale,
};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncOperation {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSync {
    pub seq: i64,
    pub change_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: SyncOperation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub after_revision: u64,
    pub changes: Vec<ClientChange>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientChange {
    pub change_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: SyncOperation,
    pub payload: SyncPayload,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub acknowledged: Vec<Uuid>,
    pub revision: u64,
    pub changes: Vec<ServerChange>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerChange {
    pub revision: u64,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub payload: SyncPayload,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SyncPayload {
    Supplier(Supplier),
    Medicine(PharmacyMedicine),
    Purchase(PurchaseAggregate),
    StockReceipt(StockReceiptAggregate),
    Sale(SaleAggregate),
    Reversal(ReversalAggregate),
    Tombstone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharmacyMedicine {
    pub medicine: Medicine,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseAggregate {
    pub operation: OperationSnapshot,
    pub purchase: Purchase,
    pub items: Vec<PurchaseItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSnapshot {
    pub id: Uuid,
    pub kind: OperationKind,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub occurred_at: OffsetDateTime,
    pub reversal_of: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInputRecord {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub purchase_item_id: Uuid,
    pub batch_id: Uuid,
    pub quantity: u32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMovementRecord {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub batch_id: Uuid,
    pub quantity_delta: i64,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSnapshot {
    pub id: Uuid,
    pub medicine_id: Uuid,
    pub batch_number: String,
    pub expiry_date: Date,
    pub quantity_on_hand: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseItemReceived {
    pub purchase_item_id: Uuid,
    pub received_quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockReceiptAggregate {
    pub operation: OperationSnapshot,
    pub inputs: Vec<StockInputRecord>,
    pub movements: Vec<InventoryMovementRecord>,
    pub batches: Vec<BatchSnapshot>,
    pub purchase_items: Vec<PurchaseItemReceived>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleBatchAllocation {
    pub sale_item_id: Uuid,
    pub batch_id: Uuid,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleAggregate {
    pub operation: OperationSnapshot,
    pub sale: Sale,
    pub allocations: Vec<SaleBatchAllocation>,
    pub movements: Vec<InventoryMovementRecord>,
    pub batches: Vec<BatchSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversalAggregate {
    pub operation: OperationSnapshot,
    pub movements: Vec<InventoryMovementRecord>,
    pub batches: Vec<BatchSnapshot>,
    pub purchase_items: Vec<PurchaseItemReceived>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncSummary {
    pub acknowledged: usize,
    pub applied: usize,
    pub revision: u64,
}
