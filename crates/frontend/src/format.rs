use rust_decimal::Decimal;
use time::{Date, OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};
pub fn money(value: Decimal) -> String {
    format!("৳{}", value.round_dp(2))
}
/// Money for tight spaces: ৳950, ৳12.4k, ৳3.1M.
pub fn compact_money(value: Decimal) -> String {
    let amount = decimal_as_f64(value);
    if amount >= 1_000_000.0 {
        format!("৳{:.1}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("৳{:.1}k", amount / 1_000.0)
    } else {
        format!("৳{amount:.0}")
    }
}
pub fn date(value: Date) -> String {
    value.to_string()
}
pub fn timestamp(value: OffsetDateTime) -> String {
    let bd = UtcOffset::from_hms(6, 0, 0).unwrap_or(UtcOffset::UTC);
    let local = value.to_offset(bd);
    local
        .format(&Rfc3339)
        .unwrap_or_else(|_| local.date().to_string())
}
pub fn decimal_as_f64(value: Decimal) -> f64 {
    value.to_string().parse().unwrap_or(0.0)
}
