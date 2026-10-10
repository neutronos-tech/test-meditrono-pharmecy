#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("duplicate invoice")]
    DuplicateInvoice,
    #[error("insufficient stock")]
    InsufficientStock,
    #[error("receipt exceeds purchased quantity")]
    ReceiptExceedsPurchase,
    #[error("stock would become negative")]
    StockWouldGoNegative,
    #[error("already reversed")]
    AlreadyReversed,
    #[error("batch metadata conflict")]
    BatchMetadataConflict,
    #[error("migration failed")]
    Migration,
    #[error("invalid stored data")]
    InvalidStoredData,
}
