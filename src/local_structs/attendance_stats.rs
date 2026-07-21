use serde::Serialize;

use super::attendance::AttendanceCategory;

#[derive(Debug, Serialize)]
pub struct AttendanceStats {
    pub overall_attendance: f64,
    pub categories: Vec<AttendanceStatsCategory>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceStatsCategory {
    pub category: AttendanceCategory,
    pub per_month: [Option<usize>; 12],
    pub per_semester: [usize; 2],
    pub total: usize,
}
