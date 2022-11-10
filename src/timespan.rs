use chrono::{Date, DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TimeSpan {
    pub low: DateTime<Utc>,
    pub high: DateTime<Utc>,
}

impl TimeSpan {
    pub fn today() -> Self {
        Self::new(Utc::today(), 1)
    }

    pub fn new<S: ToDateTimeUtc>(start: S, days: i64) -> Self {
        let start = start.to_datetime();
        let end = start + Duration::days(days);
        Self::for_date_range(start, end)
    }

    pub fn for_date_range<S: ToDateTimeUtc, E: ToDateTimeUtc>(start: S, end: E) -> Self {
        let start = start.to_datetime();
        let end = end.to_datetime();
        Self {
            low: std::cmp::min(start, end),
            high: std::cmp::max(start, end),
        }
    }
}

pub trait ToDateTimeUtc {
    fn to_datetime(self) -> DateTime<Utc>;
}

impl ToDateTimeUtc for NaiveDate {
    fn to_datetime(self) -> DateTime<Utc> {
        DateTime::from_utc(self.and_hms(0, 0, 0), Utc)
    }
}

impl ToDateTimeUtc for DateTime<Utc> {
    fn to_datetime(self) -> DateTime<Utc> {
        self
    }
}

impl ToDateTimeUtc for Date<Utc> {
    fn to_datetime(self) -> DateTime<Utc> {
        self.and_hms(0, 0, 0)
    }
}
