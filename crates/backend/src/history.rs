use crate::{
    Backend, Error,
    database::{datetime_from_db, uuid_from_db},
};
use contracts::history::{HistoryEntry, HistoryFilter, OperationKind};
use rusqlite::params;

impl Backend {
    pub fn history(&self, filter: HistoryFilter, limit: u32) -> Result<Vec<HistoryEntry>, Error> {
        let filter_sql = match filter {
            HistoryFilter::All => "1=1",
            HistoryFilter::Purchases => "o.kind='purchase'",
            HistoryFilter::Stock => "o.kind IN ('stock_receipt','stock_receipt_reversal')",
            HistoryFilter::Sales => "o.kind IN ('sale','sale_reversal')",
            HistoryFilter::Reversals => "o.reversal_of IS NOT NULL",
        };
        let sql = format!(
            "SELECT o.id,o.kind,o.actor_name,o.occurred_at,o.reversal_of,NOT EXISTS(SELECT 1 FROM operations r WHERE r.reversal_of=o.id) FROM operations o WHERE {filter_sql} ORDER BY o.occurred_at DESC,o.id DESC LIMIT ?1"
        );
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, bool>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (id, kind, actor_name, occurred_at, reversal_of, not_reversed) = row?;
            let (kind, description) = match kind.as_str() {
                "purchase" => (OperationKind::Purchase, "Purchase"),
                "stock_receipt" => (OperationKind::StockReceipt, "Stock received"),
                "stock_receipt_reversal" => (
                    OperationKind::StockReceiptReversal,
                    "Stock receipt reversed",
                ),
                "sale" => (OperationKind::Sale, "Sale"),
                "sale_reversal" => (OperationKind::SaleReversal, "Sale reversed"),
                _ => return Err(Error::InvalidStoredData),
            };
            let reversible =
                not_reversed && matches!(kind, OperationKind::StockReceipt | OperationKind::Sale);
            Ok(HistoryEntry {
                operation_id: uuid_from_db(id)?,
                kind,
                actor_name,
                occurred_at: datetime_from_db(&occurred_at)?,
                reversal_of: reversal_of.map(uuid_from_db).transpose()?,
                description: description.to_string(),
                reversible,
            })
        })
        .collect()
    }
}
