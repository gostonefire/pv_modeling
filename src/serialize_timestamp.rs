use chrono::{DateTime, Local};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

const ERROR: &str = "unable to construct DateTime<Local> from i64";

/// Serializer for serde with to serialize a chrono `DateTime<Local>` into a millisecond timestamp (Utc)
/// This function is not used directly but rather from struct fields with a serde with attribute 
/// pointing to this module
///
/// # Arguments
///
/// * 'date_time' - the date time object
/// * 'serializer' - serializer given from serde
pub fn serialize<S>(
    date: &DateTime<Local>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    date.naive_local().and_utc().timestamp_millis().serialize(serializer)
}


pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    // FIXME it is not possible to call a Deserializer function without a serde::de::Visitor
    let milli_seconds = i64::deserialize(d)?;

    Ok(DateTime::from_timestamp_millis(milli_seconds)
        .ok_or_else(|| D::Error::custom(ERROR))?
        .naive_utc()
        .and_local_timezone(Local)
        .unwrap())

    //NaiveDateTime::from_timestamp_opt(seconds, 0).ok_or_else(|| {
    //    D::Error::custom("unable to construct DateTime<Local> from i64")
    //})
}