#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct GradesResponse {
    pub ocenyPrzedmioty: Vec<GradesResponseSubject>,
    pub ustawienia: GradesResponseSettings,
}

#[derive(Deserialize)]
pub struct GradesResponseSubject {
    pub przedmiotNazwa: String,
    pub pozycja: isize,
    pub nauczyciele: Vec<String>,
    pub ocenyCzastkowe: Vec<GradesResponseGrade>,
    pub egzaminFormaPraktyczna: Value,
    pub egzaminFormaUstna: Value,
    pub egzaminOcenaProponowana: Value,
    pub egzaminOcenaLaczna: Value,
    pub sumaPunktow: Value,
    pub sumaPunktowWszystkieSemestry: Value,
    pub srednia: f64,
    pub sredniaWszystkieSemestry: f64,
    pub proponowanaOcenaOkresowa: Option<String>,
    pub proponowanaOcenaOkresowaPunkty: Option<String>,
    pub ocenaOkresowa: Option<String>,
    pub ocenaOkresowaPunkty: Option<String>,
    pub podsumowanieOcen: Value,
}

#[derive(Deserialize)]
pub struct GradesResponseSettings {
    pub isSredniaAndPunkty: bool,
    pub isDorosli: bool,
    pub isOcenaOpisowa: bool,
    pub isOstatniOkresKlasyfikacyjny: bool,
}

#[derive(Deserialize)]
pub struct GradesResponseGrade {
    pub wpis: String,
    pub dataOceny: String,
    pub kategoriaKolumny: Option<String>,
    pub nazwaKolumny: Option<String>,
    pub waga: f64,
    pub kolorOceny: usize,
    pub nauczyciel: String,
    pub zmienionaOdOstatniegoLogowania: bool,
}
