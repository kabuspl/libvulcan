#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AttendanceResponse {
    pub kategoriaFrekwencji: u8,
    pub data: String,
    pub godzinaOd: String,
    pub godzinaDo: String,
    pub idPoraLekcji: u32,
    pub idLekcjaOddzial: u32,
    pub numerLekcji: u8,
    pub opisZajec: String,
    pub nauczyciel: String,
}
