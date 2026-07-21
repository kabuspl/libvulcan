#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct TimetableResponse {
    pub data: String,
    pub godzinaOd: String,
    pub godzinaDo: String,
    pub prowadzacy: String,
    pub prowadzacyWspomagajacy1: Option<String>,
    pub prowadzacyWspomagajacy2: Option<String>,
    pub przedmiot: String,
    pub podzial: Option<String>,
    pub sala: String,
    pub pseudonim: Option<Value>,
    pub zmiany: Vec<TimetableResponseChanges>,
    pub adnotacja: u32,
    pub dodatkowe: bool,
    pub zrealizowane: bool,
}

#[derive(Deserialize)]
pub struct TimetableResponseChanges {
    pub zmiana: u8,
    pub typProwadzacego: u8,
    pub dzien: Option<String>,
    pub nrLekcji: Option<u8>,
    pub godzinaOd: Option<String>,
    pub grupa: Option<String>,
    pub zajecia: Option<String>,
    pub sala: Option<String>,
    pub prowadzacy: Option<String>,
    pub informacjeNieobecnosc: Option<String>,
}
