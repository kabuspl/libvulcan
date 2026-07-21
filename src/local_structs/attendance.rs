use serde::{Deserialize, Serialize};

use crate::date_utils::DateTimeRange;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Attendance {
    pub category: AttendanceCategory,
    pub date_time_range: DateTimeRange,
    pub subject: String,
    pub teacher: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AttendanceCategory {
    Presence,
    Absence,
    ExcusedAbsence,
    AbsenceForSchoolReasons,
    Late,
    ExcusedLate,
    Excuse,
}

pub fn get_attendance_category_from_id(category_id: u8) -> AttendanceCategory {
    match category_id {
        1 => AttendanceCategory::Presence,
        2 => AttendanceCategory::Absence,
        3 => AttendanceCategory::ExcusedAbsence,
        4 => AttendanceCategory::Late,
        5 => AttendanceCategory::AbsenceForSchoolReasons,
        6 => AttendanceCategory::ExcusedLate,
        7 => AttendanceCategory::Excuse,
        _ => todo!("Attendance category {} is not implemented", category_id),
    }
}
