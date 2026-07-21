use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DateTimeRange {
    pub date_start: NaiveDateTime,
    pub date_end: NaiveDateTime,
}

impl DateTimeRange {
    pub fn from_date_range(date_start: NaiveDate, date_end: NaiveDate) -> Self {
        Self {
            date_start: date_start.and_hms_opt(0, 0, 0).unwrap(),
            date_end: date_end.and_hms_opt(23, 59, 59).unwrap(),
        }
    }

    pub fn get_date_start_v(&self) -> String {
        self.get_date_v(self.date_start)
    }

    pub fn get_date_end_v(&self) -> String {
        self.get_date_v(self.date_end)
    }

    fn get_date_v(&self, date: NaiveDateTime) -> String {
        //fuck timezones
        date.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}

pub fn get_monday(base_date: NaiveDate) -> NaiveDate {
    base_date - Duration::days(base_date.weekday().num_days_from_monday() as i64)
}
