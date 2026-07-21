use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubjectGrades {
    pub subject_name: String,
    pub teacher: String,
    pub average: f64,
    pub grades: Vec<Grade>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Grade {
    pub grade: String,
    pub comment: Option<String>,
    pub date: NaiveDate,
    pub category: Option<String>,
    pub description: Option<String>,
    pub weight: usize,
    pub teacher: String,
}
