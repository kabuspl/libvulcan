use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Exam {
    pub exam_type: ExamType,
    pub subject: String,
    pub description: String,
    pub date: NaiveDate,
    pub id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ExamType {
    Test,
    Quiz,
    Exam,
}

pub fn get_exam_type_from_id(type_id: u8) -> ExamType {
    match type_id {
        1 => ExamType::Test,
        2 => ExamType::Quiz,
        3 => ExamType::Exam,
        _ => todo!("Exam type {} is not implemented", type_id),
    }
}
