use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::date_utils::DateTimeRange;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TimetableElement {
    pub date_time_range: DateTimeRange,
    pub teacher: String,
    pub subject: String,
    pub room: String,
    pub changes: Vec<TimetableChange>,
    pub completed: bool,
    pub group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TimetableChange {
    pub change_type: TimetableChangeType,
    pub date: Option<NaiveDate>,
    pub room: Option<String>,
    pub subject: Option<String>,
    pub teacher: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TimetableChangeType {
    Canceled,
    MovedFromHere,
    MovedToHere,
    SubstituteTeacher,
}

pub fn get_change_type_from_id(change_id: u8) -> TimetableChangeType {
    match change_id {
        1 | 4 => TimetableChangeType::Canceled,
        5 => TimetableChangeType::MovedFromHere,
        6 => TimetableChangeType::MovedToHere,
        7 => TimetableChangeType::SubstituteTeacher,
        _ => todo!("Timetable change category {} is not implemented", change_id),
    }
}
