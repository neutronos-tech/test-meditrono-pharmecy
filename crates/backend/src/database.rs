use crate::Error;
use rusqlite::Connection;
use rust_decimal::Decimal;
use std::str::FromStr;
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

pub(crate) fn migrate(db: &mut Connection) -> Result<(), Error> {
    let version: i64 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    match version {
        0 => {
            let tx = db.transaction()?;
            tx.execute_batch(include_str!("../../../migrations/0001_initial.sql"))?;
            tx.execute_batch(include_str!(
                "../../../migrations/0002_medicine_revision.sql"
            ))?;
            tx.execute_batch(include_str!("../../../migrations/0003_remote_identity.sql"))?;
            tx.execute_batch("PRAGMA user_version = 3")?;
            tx.commit()?;
        }
        1 => {
            let tx = db.transaction()?;
            tx.execute_batch(include_str!(
                "../../../migrations/0002_medicine_revision.sql"
            ))?;
            tx.execute_batch(include_str!("../../../migrations/0003_remote_identity.sql"))?;
            tx.execute_batch("PRAGMA user_version = 3")?;
            tx.commit()?;
        }
        2 => {
            let tx = db.transaction()?;
            tx.execute_batch(include_str!("../../../migrations/0003_remote_identity.sql"))?;
            tx.execute_batch("PRAGMA user_version = 3")?;
            tx.commit()?;
        }
        3 => {}
        _ => return Err(Error::Migration),
    }
    Ok(())
}

pub(crate) fn uuid_from_db(value: String) -> Result<Uuid, Error> {
    Uuid::parse_str(&value).map_err(|_| Error::InvalidStoredData)
}
pub(crate) fn datetime_to_db(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .expect("RFC3339 supports current time")
}
pub(crate) fn datetime_from_db(value: &str) -> Result<OffsetDateTime, Error> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| Error::InvalidStoredData)
}
pub(crate) fn date_to_db(value: Date) -> String {
    value.to_string()
}
pub(crate) fn date_from_db(value: &str) -> Result<Date, Error> {
    Date::parse(
        value,
        &time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
            .map_err(|_| Error::InvalidStoredData)?,
    )
    .map_err(|_| Error::InvalidStoredData)
}
pub(crate) fn decimal_to_db(value: Decimal) -> String {
    value.normalize().to_string()
}
pub(crate) fn decimal_from_db(value: &str) -> Result<Decimal, Error> {
    Decimal::from_str(value).map_err(|_| Error::InvalidStoredData)
}

pub(crate) fn bd_date(now: OffsetDateTime) -> Date {
    now.to_offset(time::UtcOffset::from_hms(6, 0, 0).expect("fixed offset"))
        .date()
}
pub(crate) fn bd_day_bounds(date: Date) -> Result<(OffsetDateTime, OffsetDateTime), Error> {
    let start = date
        .with_hms(0, 0, 0)
        .map_err(|_| Error::InvalidStoredData)?
        .assume_offset(time::UtcOffset::from_hms(6, 0, 0).map_err(|_| Error::InvalidStoredData)?)
        .to_offset(time::UtcOffset::UTC);
    Ok((start, start + time::Duration::days(1)))
}
