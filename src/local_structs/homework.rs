use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Homework {
    pub subject: String,
    pub description: String,
    pub date: NaiveDate,
    pub id: u32,
}
