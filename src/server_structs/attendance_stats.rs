#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AttendanceStatsResponse {
    pub podsumowanie: f64,
    pub statystyki: Vec<AttendanceStatsResponseCategory>,
}

#[derive(Deserialize)]
pub struct AttendanceStatsResponseCategory {
    pub kategoriaFrekwencji: u8,
    pub miesiace: Vec<AttendanceStatsResponseCategoryMonth>,
    pub okresy: Vec<usize>,
    pub razem: usize,
}

#[derive(Deserialize)]
pub struct AttendanceStatsResponseCategoryMonth {
    pub miesiac: u8,
    pub wartosc: usize,
}
