use chrono::{DateTime, Utc};

pub fn timestamp_to_chrono(ts: Option<pbjson_types::Timestamp>) -> Option<DateTime<Utc>> {
    let ts = ts?;
    let nanos = u32::try_from(ts.nanos).ok()?;
    DateTime::<Utc>::from_timestamp(ts.seconds, nanos)
}

pub fn chrono_to_timestamp(dt: Option<DateTime<Utc>>) -> Option<pbjson_types::Timestamp> {
    let dt = dt?;
    Some(pbjson_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}
