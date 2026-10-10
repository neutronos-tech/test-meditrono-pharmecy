use rust_decimal::{Decimal, RoundingStrategy};
use std::str::FromStr;
pub fn required(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err("This field is required.".into())
    } else {
        Ok(value.to_string())
    }
}
pub fn decimal_from_ui(value: f64) -> Result<Decimal, String> {
    if !value.is_finite() || value < 0.0 {
        return Err("Enter a valid non-negative price.".into());
    }
    Decimal::from_str(&value.to_string())
        .map(|value| value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero))
        .map_err(|_| "Price is out of range.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn price_conversion_checks_finite_and_rounds() {
        assert!(decimal_from_ui(f64::NAN).is_err());
        assert!(decimal_from_ui(f64::INFINITY).is_err());
        assert!(decimal_from_ui(-1.0).is_err());
        assert_eq!(decimal_from_ui(12.345).unwrap(), Decimal::new(1235, 2));
    }
}
