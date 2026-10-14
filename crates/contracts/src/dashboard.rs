use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub today_sales_total: Decimal,
    pub today_sales_count: u32,
    pub medicines_in_stock: u32,
    pub low_stock_medicines: u32,
    pub near_expiry_batches: u32,
    /// Sales for the last seven Bangladesh business days, oldest first, ending today.
    pub daily_sales: Vec<DailySales>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySales {
    pub date: Date,
    pub total: Decimal,
    pub count: u32,
}
