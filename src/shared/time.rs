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
        // Use whole-second precision so JSON encoding stays compatible with
        // clients that only accept RFC3339 without fractional + offset.
        nanos: 0,
    })
}
