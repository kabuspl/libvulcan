#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct ExamsHomeworkResponse {
    pub typ: u8,
    pub przedmiotNazwa: String,
    pub data: String,
    pub hasAttachment: bool,
    pub id: u32,
}

#[derive(Deserialize)]
pub struct ExamDetailsResponse {
    pub typ: u8,
    pub data: String,
    pub przedmiotNazwa: String,
    pub nauczycielImieNazwisko: String,
    pub opis: String,
    pub sprawdzianModulDydaktyczny: bool,
    pub linki: Vec<Value>,
    pub id: u32,
}

#[derive(Deserialize)]
pub struct HomeworkDetailsResponse {
    pub typ: u8,
    pub data: String,
    pub terminOdpowiedzi: String,
    pub przedmiotNazwa: String,
    pub nauczycielImieNazwisko: String,
    pub opis: String,
    pub zadanieModulDydaktyczny: bool,
    pub status: u32,
    pub odpowiedzWymagana: bool,
    pub linki: Vec<Value>,
    pub zalaczniki: Vec<Value>,
    pub odpowiedz: Value,
    pub id: u32,
}
