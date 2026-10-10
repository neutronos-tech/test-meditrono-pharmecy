use crate::{
    Backend, Error,
    database::{bd_date, bd_day_bounds, date_to_db, datetime_to_db, decimal_from_db},
};
use contracts::dashboard::Dashboard;
use rusqlite::params;
use rust_decimal::Decimal;
use time::OffsetDateTime;

impl Backend {
    pub fn dashboard(&self) -> Result<Dashboard, Error> {
        let now = OffsetDateTime::now_utc();
        let today = bd_date(now);
        let (start, end) = bd_day_bounds(today)?;
        let mut stmt=self.db.prepare("SELECT si.quantity,si.unit_price FROM sales s JOIN sale_items si ON si.sale_id=s.id WHERE s.created_at>=?1 AND s.created_at<?2 AND NOT EXISTS(SELECT 1 FROM operations r WHERE r.reversal_of=s.operation_id)")?;
        let rows = stmt.query_map(params![datetime_to_db(start), datetime_to_db(end)], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut today_sales_total = Decimal::ZERO;
        for row in rows {
            let (quantity, price) = row?;
            today_sales_total += Decimal::from(quantity) * decimal_from_db(&price)?;
        }
        let today_sales_count=self.db.query_row("SELECT COUNT(*) FROM sales s WHERE s.created_at>=?1 AND s.created_at<?2 AND NOT EXISTS(SELECT 1 FROM operations r WHERE r.reversal_of=s.operation_id)",params![datetime_to_db(start),datetime_to_db(end)],|row|row.get(0))?;
        let (medicines_in_stock,low_stock_medicines):(u32,u32)=self.db.query_row("SELECT COUNT(*),COALESCE(SUM(CASE WHEN total<=10 THEN 1 ELSE 0 END),0) FROM (SELECT m.id,SUM(b.quantity_on_hand) total FROM medicines m JOIN inventory_batches b ON b.medicine_id=m.id GROUP BY m.id HAVING total>0)",[],|row|Ok((row.get(0)?,row.get(1)?)))?;
        let near_expiry_batches=self.db.query_row("SELECT COUNT(*) FROM inventory_batches WHERE quantity_on_hand>0 AND expiry_date>=?1 AND expiry_date<=?2",params![date_to_db(today),date_to_db(today+time::Duration::days(30))],|row|row.get(0))?;
        Ok(Dashboard {
            today_sales_total,
            today_sales_count,
            medicines_in_stock,
            low_stock_medicines,
            near_expiry_batches,
        })
    }
}
