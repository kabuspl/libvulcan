pub extern crate chrono;

use std::{
    any::Any,
    collections::HashMap,
    io::{Cursor, Write},
    path::Path,
    sync::Arc,
    time::Instant,
};

use anyhow::{Context, Result};

use base64::prelude::*;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use clap::Parser;
use date_utils::DateTimeRange;
use errors::VError;
use image::ImageReader;
pub use local_structs::exams::ExamType;
use local_structs::{
    attendance::{get_attendance_category_from_id, Attendance, AttendanceCategory},
    attendance_stats::{AttendanceStats, AttendanceStatsCategory},
    exams::{get_exam_type_from_id, Exam},
    grades::{Grade, SubjectGrades},
    homework::Homework,
    lucky_number::LuckyNumber,
    timetable::{get_change_type_from_id, TimetableChange, TimetableElement},
};
use mappings::{get_endpoint, Endpoint};
use regex::Regex;
use reqwest::{StatusCode, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use scraper::{Element, ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use server_structs::{
    attendance::AttendanceResponse,
    attendance_stats::AttendanceStatsResponse,
    exams_homework::{ExamDetailsResponse, ExamsHomeworkResponse, HomeworkDetailsResponse},
    grades::GradesResponse,
    lucky_number::LuckyNumberResponse,
    timetable::TimetableResponse,
};

pub mod date_utils;
pub mod errors;
pub mod local_structs;
mod mappings;
mod server_structs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    entry_url: Url,

    #[arg(short, long)]
    login: String,

    #[arg(short, long)]
    password: String,
}

// #[tokio::main]
// fn main() -> Result<()> {
//     let args = Args::parse();

//     println!("{:#?}", NaiveDate::parse_from_str("19.06.2024", "%d.%m.%Y"));

//     let mut vscraper = VScraper::new(args.entry_url, args.login, args.password);
//     vscraper.login()?;
//     println!("{:#?}", vscraper.get_grades());

//     Ok(())
// }

#[derive(Serialize, Deserialize)]
struct VCache {
    api_base_url: Option<String>,
    api_key: Option<String>,
    register_id: Option<u16>,
}

#[derive(PartialEq, Serialize)]
pub enum CaptchaGender {
    Male,
    Female,
}

#[derive(Serialize)]
pub struct Captchas {
    pub images_base64: Vec<String>,
    pub gender: CaptchaGender,
}

pub struct VScraper {
    entry_url: Option<Url>,
    login: Option<String>,
    password: Option<String>,

    http: reqwest::Client,
    cookie_store: Arc<CookieStoreMutex>,

    api_base_url: Option<String>,
    api_key: Option<String>,
    register_id: Option<u16>,

    request_verification_token: Option<String>,

    exams_cache: Vec<Exam>,
    homework_cache: Vec<Homework>,

    login_stage1_url: Option<Url>,
}

impl VScraper {
    pub fn new(entry_url: Url) -> Self {
        let cookie_store = {
            if let Ok(file) = std::fs::File::open("cookies.json").map(std::io::BufReader::new) {
                CookieStore::load_json(file).unwrap()
            } else {
                CookieStore::new(None)
            }
        };
        let cookie_store = CookieStoreMutex::new(cookie_store);
        let cookie_store = Arc::new(cookie_store);

        let client = reqwest::Client::builder()
            .cookie_provider(Arc::clone(&cookie_store))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:129.0) Gecko/20100101 Firefox/129.0")
            .build()
            .unwrap();

        // let mut api_base_url: Option<String> = None;
        // let mut api_key: Option<String> = None;
        // let mut register_id: Option<u16> = None;
        // if restore_session && Path::new("vcache.json").exists() {
        //     let vcache: VCache =
        //         serde_json::from_str(&std::fs::read_to_string("vcache.json").unwrap()).unwrap();
        //     api_base_url = vcache.api_base_url;
        //     api_key = vcache.api_key;
        //     register_id = vcache.register_id;
        // }

        let instance = Self {
            login: None,
            password: None,
            entry_url: Some(entry_url),
            http: client,
            cookie_store,
            api_base_url: None,
            api_key: None,
            register_id: None,
            exams_cache: vec![],
            homework_cache: vec![],
            login_stage1_url: None,
            request_verification_token: None,
        };

        // if !instance.is_logged_in() {
        //     instance.login().unwrap();
        // }

        instance
    }

    pub fn restore_session(&mut self) {
        if Path::new("vcache.json").exists() {
            let vcache: VCache =
                serde_json::from_str(&std::fs::read_to_string("vcache.json").unwrap()).unwrap();
            self.api_base_url = vcache.api_base_url;
            self.api_key = vcache.api_key;
            self.register_id = vcache.register_id;
        }
    }

    fn save_session(&self) {
        let mut writer = std::fs::File::create("cookies.json")
            .map(std::io::BufWriter::new)
            .unwrap();
        let store = self.cookie_store.lock().unwrap();
        store
            .save_incl_expired_and_nonpersistent_json(&mut writer)
            .unwrap();

        std::fs::write(
            "vcache.json",
            serde_json::to_string(&VCache {
                api_base_url: self.api_base_url.clone(),
                api_key: self.api_key.clone(),
                register_id: self.register_id,
            })
            .unwrap(),
        )
        .unwrap();
    }

    pub async fn is_logged_in(&self) -> bool {
        match self.get_current_period().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn reset_scraper(&mut self) {
        self.api_base_url = None;
        self.api_key = None;
        self.exams_cache = vec![];
        self.homework_cache = vec![];
        self.login = None;
        self.password = None;
        self.login_stage1_url = None;
        self.request_verification_token = None;
        self.register_id = None;
        self.cookie_store.lock().unwrap().clear();
    }

    pub async fn get_captchas(
        &mut self,
        login: &str,
        password: &str,
        try_number: usize,
    ) -> Result<Option<Captchas>> {
        // Stage1 = first login page
        let login_stage1_req = self
            .http
            .get(self.entry_url.as_ref().unwrap().clone())
            .send()
            .await?;
        let login_stage1_url = login_stage1_req.url().clone();
        let login_stage1_html = Html::parse_document(&login_stage1_req.text().await?);

        if login_stage1_url.to_string().contains("/Fs/Ls") && try_number < 4 {
            self.reset_scraper();
            return Box::pin(self.get_captchas(login, password, try_number + 1)).await;
        }

        self.login = Some(login.to_owned());
        self.password = Some(password.to_owned());
        self.login_stage1_url = Some(login_stage1_url.clone());

        let request_verification_token = login_stage1_html
            .select(&Selector::parse("[name='__RequestVerificationToken']").unwrap())
            .next()
            .ok_or(VError::ElementNotFound(
                "[name='__RequestVerificationToken']".to_owned(),
                login_stage1_url.as_ref().into(),
            ))?
            .attr("value")
            .ok_or(VError::ElementAttributeNotFound(
                "value".to_owned(),
                "[name='__RequestVerificationToken']".to_owned(),
                login_stage1_url.as_ref().into(),
            ))?;

        self.request_verification_token = Some(request_verification_token.to_owned());

        let regex = Regex::new(r"nonce:\s*'(\d+)',\s*signature:\s*'([A-Z0-9]+)'").unwrap();

        if let Some(captures) = regex.captures(&login_stage1_html.html()) {
            let nonce = &captures[1];
            let signature = &captures[2];

            let query_user_info_text = self
                .http
                .post("https://dziennik-logowanie.vulcan.net.pl/kielce/Account/QueryUserInfo")
                .form(&[
                    ("login", login),
                    ("__RequestVerificationToken", request_verification_token),
                    ("nonce", nonce),
                    ("nonceSignature", signature),
                ])
                .send()
                .await?
                .text()
                .await?;

            let query_user_info_resp: Value = serde_json::from_str(&query_user_info_text)?;

            let show_captcha = query_user_info_resp["data"]
                .as_object()
                .expect("should always work")
                .get("ShowCaptcha")
                .expect("what in the fuck")
                .as_bool()
                .unwrap_or(true);

            if !show_captcha {
                return Ok(None);
            }
        }

        let captcha_gender = login_stage1_html
            .select(&Selector::parse("label[for='captchaUser']").unwrap())
            .next()
            .ok_or(VError::ElementNotFound(
                "label[for='captchaUser']".to_owned(),
                login_stage1_url.as_ref().into(),
            ))?
            .text()
            .next()
            .ok_or(VError::ElementAttributeNotFound(
                "[text]".to_owned(),
                "label[for='captchaUser']".to_owned(),
                login_stage1_url.as_ref().into(),
            ))?
            .contains("męskie");

        let captcha_selector = Selector::parse(".v-captcha-image").unwrap();
        let captcha_images: Vec<String> = login_stage1_html
            .select(&captcha_selector)
            .map(|el| {
                el.attr("src")
                    .ok_or(VError::ElementAttributeNotFound(
                        "[text]".to_owned(),
                        "label[for='captchaUser']".to_owned(),
                        login_stage1_url.as_ref().into(),
                    ))
                    .unwrap()
                    .replace("data:image/png;base64,", "")
            })
            .collect();

        if captcha_images.len() > 0 {
            Ok(Some(Captchas {
                gender: if captcha_gender {
                    CaptchaGender::Male
                } else {
                    CaptchaGender::Female
                },
                images_base64: captcha_images,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn login(&mut self, captcha_answer: Option<&str>) -> Result<()> {
        if self.request_verification_token.is_none()
            || self.login_stage1_url.is_none()
            || self.entry_url.is_none()
        {
            return Err(VError::NotLoggedIn.into());
        }
        let mut stage2_form_data = HashMap::new();
        stage2_form_data.insert("Login", self.login.as_ref().unwrap().to_owned());
        stage2_form_data.insert("Haslo", self.password.as_ref().unwrap().to_owned());
        stage2_form_data.insert(
            "__RequestVerificationToken",
            self.request_verification_token.as_ref().unwrap().to_owned(),
        );
        if let Some(captcha_answer) = captcha_answer {
            stage2_form_data.insert("captchaUser", captcha_answer.to_owned());
        }

        // Stage2 = Fs/Ls - hidden form auto submit - title: Working...
        let login_stage2_req = self
            .http
            .post(self.login_stage1_url.as_ref().unwrap().as_ref())
            .form(&stage2_form_data)
            .send()
            .await?;
        let login_stage2_html = Html::parse_document(&login_stage2_req.text().await?);

        let wa = login_stage2_html
            .select(&Selector::parse("[name='wa']").unwrap())
            .next()
            .ok_or(VError::ElementNotFound(
                "[name='wa']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?
            .attr("value")
            .ok_or(VError::ElementAttributeNotFound(
                "value".to_owned(),
                "[name='wa']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?;
        let wresult = login_stage2_html
            .select(&Selector::parse("[name='wresult']").unwrap())
            .next()
            .ok_or(VError::ElementNotFound(
                "[name='wresult']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?
            .attr("value")
            .ok_or(VError::ElementAttributeNotFound(
                "value".to_owned(),
                "[name='wresult']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?;
        let wctx = login_stage2_html
            .select(&Selector::parse("[name='wctx']").unwrap())
            .next()
            .ok_or(VError::ElementNotFound(
                "[name='wctx']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?
            .attr("value")
            .ok_or(VError::ElementAttributeNotFound(
                "value".to_owned(),
                "[name='wctx']".to_owned(),
                self.login_stage1_url.as_ref().unwrap().as_ref().into(),
            ))?;

        let mut stage3_form_data = HashMap::new();
        stage3_form_data.insert("wa", wa);
        stage3_form_data.insert("wresult", wresult);
        stage3_form_data.insert("wctx", wctx);

        let login_stage3_req = self
            .http
            .post(self.entry_url.as_ref().unwrap().clone())
            .form(&stage3_form_data)
            .send()
            .await?;
        let student_dashboard_url = login_stage3_req.url().clone();
        // println!("{}", login_stage3_req.text().await?);

        //https://dziennik-uczen.vulcan.net.pl/kielce/api/99f04f67-d7f6-4f30-9d02-61cdd656612f

        let students_req = self
            .http
            .get(
                student_dashboard_url
                    .as_str()
                    .replace("App", &format!("api/{}", &get_endpoint(Endpoint::Context))),
            )
            .send()
            .await?;

        // println!("{}", );
        let context_data: Value = serde_json::from_str(&students_req.text().await?)?;
        self.api_key = Some(
            context_data["uczniowie"][0]["key"]
                .as_str()
                .ok_or(VError::ApiFormatChanged(
                    "students".to_owned(),
                    "uczniowie.0.key".to_owned(),
                ))?
                .to_owned(),
        );
        self.register_id = Some(context_data["uczniowie"][0]["idDziennik"].as_u64().ok_or(
            VError::ApiFormatChanged("students".to_owned(), "uczniowie.0.idDziennik".to_owned()),
        )? as u16);
        self.api_base_url = Some(student_dashboard_url.as_str().replace("App", "api/"));

        Ok(())
    }

    async fn get(&self, endpoint: Endpoint, query: &[(&str, &str)]) -> Result<Value> {
        if self.api_key.is_none() {
            return Err(VError::NotLoggedIn.into());
        }

        let now = Instant::now();

        let query_with_key = &[&[("key", self.api_key.as_ref().unwrap().as_str())], query].concat();

        let url = format!(
            "{}{}",
            self.api_base_url.as_ref().unwrap(),
            get_endpoint(endpoint)
        );

        let req = self.http.get(&url).query(query_with_key).send().await?;

        self.save_session();

        let text = req.text().await?;

        let data = serde_json::from_str(&text)?;

        let elapsed = now.elapsed();

        println!(
            "[{}] [GET: {:?}] {} [{}ms]",
            Local::now().format("%H:%M:%S").to_string(),
            endpoint,
            url,
            elapsed.as_millis()
        );

        Ok(data)
    }

    pub async fn get_grades(&self) -> Result<Vec<SubjectGrades>> {
        if self.api_key.is_none() {
            return Err(VError::NotLoggedIn.into());
        }

        let resp = self
            .get(
                Endpoint::Grades,
                &[(
                    "idOkresKlasyfikacyjny",
                    &self.get_current_period().await?.to_string(),
                )],
            )
            .await;

        let data: GradesResponse = serde_json::from_value(resp?)?;

        let grades_mapped = data
            .ocenyPrzedmioty
            .iter()
            .map(|subject| SubjectGrades {
                subject_name: subject.przedmiotNazwa.to_owned(),
                teacher: if subject.nauczyciele.len() == 0 {
                    "".to_owned()
                } else {
                    subject.nauczyciele[0].to_owned()
                },
                average: subject.srednia,
                grades: subject
                    .ocenyCzastkowe
                    .iter()
                    .map(|grade| {
                        let left_parenthesis_pos = grade.wpis.find(|c| c == '(');
                        let right_parenthesis_pos = grade.wpis.rfind(|c| c == ')');
                        let mut comment: Option<String> = None;
                        if let (Some(left_parenthesis_pos), Some(right_parenthesis_pos)) =
                            (left_parenthesis_pos, right_parenthesis_pos)
                        {
                            comment = Some(
                                grade.wpis.as_str()
                                    [(left_parenthesis_pos + 1)..right_parenthesis_pos]
                                    .to_owned(),
                            )
                        }
                        Grade {
                            grade: grade
                                .wpis
                                .to_owned()
                                .split('(')
                                .next()
                                .unwrap()
                                .trim()
                                .to_owned(),
                            date: NaiveDate::parse_from_str(&grade.dataOceny, "%d.%m.%Y")
                                .expect("Cannot format grade date"),
                            comment,
                            category: grade.kategoriaKolumny.to_owned(),
                            description: grade.nazwaKolumny.to_owned(),
                            weight: grade.waga as usize,
                            teacher: grade.nauczyciel.to_owned(),
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(grades_mapped)
    }

    // TODO: use self.get()
    async fn get_current_period(&self) -> Result<u16> {
        if self.api_key.is_none() {
            return Err(VError::NotLoggedIn.into());
        }

        let req = self
            .http
            .get(format!(
                "{}/{}",
                self.api_base_url.as_ref().unwrap(),
                get_endpoint(Endpoint::OkresyKlasyfikacyjne)
            ))
            .query(&[
                ("key", self.api_key.as_ref().unwrap()),
                (
                    "idDziennik",
                    &self.register_id.as_ref().unwrap().to_string(),
                ),
            ])
            .send()
            .await?;

        let text = &req.text().await?;

        let data = serde_json::from_str::<Value>(text)?;

        if !data.get("message").is_none() {
            return Err(VError::NotLoggedIn.into());
        }

        let periods = data.as_array().ok_or(VError::ApiFormatChanged(
            "OkresyKlasyfikacyjne".to_owned(),
            "root array".to_owned(),
        ))?;

        let mut current_period = 0u16;
        for period in periods {
            let parsed_start_date = DateTime::parse_from_rfc3339(period["dataOd"].as_str().ok_or(
                VError::ApiFormatChanged("OkresyKlasyfikacyjne".to_owned(), "x.dataOd".to_owned()),
            )?)
            .unwrap();
            if parsed_start_date < Local::now() {
                current_period = period["id"].as_u64().ok_or(VError::ApiFormatChanged(
                    "OkresyKlasyfikacyjne".to_owned(),
                    "x.id".to_owned(),
                ))? as u16;
            }
        }

        Ok(current_period)
    }

    pub async fn get_attendance_stats(&self) -> Result<AttendanceStats> {
        let resp = self.get(Endpoint::AttendanceStats, &[]);
        let data: AttendanceStatsResponse = serde_json::from_value(resp.await?)?;

        let new_stats = AttendanceStats {
            overall_attendance: data.podsumowanie,
            categories: data
                .statystyki
                .iter()
                .map(|category| {
                    let mut months_fixed: [Option<usize>; 12] = [None; 12];

                    for month in &category.miesiace {
                        months_fixed[month.miesiac as usize - 1] = Some(month.wartosc)
                    }

                    AttendanceStatsCategory {
                        category: get_attendance_category_from_id(category.kategoriaFrekwencji),
                        per_month: months_fixed,
                        per_semester: [category.okresy[0], category.okresy[1]],
                        total: category.razem,
                    }
                })
                .collect(),
        };

        Ok(new_stats)
    }

    pub async fn get_attendance(&self, date_range: DateTimeRange) -> Result<Vec<Attendance>> {
        let resp = self
            .get(
                Endpoint::Attendance,
                &[
                    ("dataOd", &date_range.get_date_start_v()),
                    ("dataDo", &date_range.get_date_end_v()),
                ],
            )
            .await;
        let data: Vec<AttendanceResponse> = serde_json::from_value(resp?)?;

        let new_data = data
            .iter()
            .map(|attendance| Attendance {
                category: get_attendance_category_from_id(attendance.kategoriaFrekwencji),
                date_time_range: DateTimeRange {
                    date_start: NaiveDateTime::parse_from_str(
                        &attendance.godzinaOd,
                        "%Y-%m-%dT%H:%M:%S%:z",
                    )
                    .unwrap(),
                    date_end: NaiveDateTime::parse_from_str(
                        &attendance.godzinaDo,
                        "%Y-%m-%dT%H:%M:%S%:z",
                    )
                    .unwrap(),
                },
                subject: attendance.opisZajec.to_owned(),
                teacher: attendance.nauczyciel.to_owned(),
            })
            .collect();

        Ok(new_data)
    }

    pub async fn get_timetable(&self, date_range: DateTimeRange) -> Result<Vec<TimetableElement>> {
        let resp = self
            .get(
                Endpoint::Timetable,
                &[
                    ("dataOd", &date_range.get_date_start_v()),
                    ("dataDo", &date_range.get_date_end_v()),
                    ("zakresDanych", "2"),
                ],
            )
            .await;

        let data: Vec<TimetableResponse> = serde_json::from_value(resp?)?;

        let new_data = data
            .iter()
            .map(|lesson| TimetableElement {
                date_time_range: DateTimeRange {
                    date_start: NaiveDateTime::parse_from_str(
                        &lesson.godzinaOd,
                        "%Y-%m-%dT%H:%M:%S%:z",
                    )
                    .unwrap(),
                    date_end: NaiveDateTime::parse_from_str(
                        &lesson.godzinaDo,
                        "%Y-%m-%dT%H:%M:%S%:z",
                    )
                    .unwrap(),
                },
                teacher: lesson.prowadzacy.to_owned(),
                subject: lesson.przedmiot.to_owned(),
                room: lesson.sala.to_owned(),
                changes: lesson
                    .zmiany
                    .iter()
                    .map(|change| {
                        let mut new_change = TimetableChange {
                            change_type: get_change_type_from_id(change.zmiana),
                            date: None,
                            room: change.sala.clone(),
                            subject: change.zajecia.clone(),
                            teacher: change.prowadzacy.clone(),
                            description: change.informacjeNieobecnosc.clone(),
                        };

                        if let Some(dzien) = &change.dzien {
                            new_change.date = Some(
                                NaiveDateTime::parse_from_str(dzien, "%Y-%m-%dT%H:%M:%S%:z")
                                    .unwrap()
                                    .date(),
                            );
                        }

                        new_change
                    })
                    .collect(),
                completed: lesson.zrealizowane,
                group: lesson.podzial.clone(),
            })
            .collect();

        Ok(new_data)
    }

    // FIXME: do not add exam to cache if it exists in it already
    async fn refresh_exams_and_homework_cache(
        &mut self,
        date_range: &DateTimeRange,
        invalidate_cache: bool,
    ) -> Result<()> {
        let resp = self
            .get(
                Endpoint::ExamsHomework,
                &[
                    ("dataOd", &date_range.get_date_start_v()),
                    ("dataDo", &date_range.get_date_end_v()),
                ],
            )
            .await;

        let data: Vec<ExamsHomeworkResponse> = serde_json::from_value(resp?)?;

        let exams_cache_content: Vec<Exam>;
        if Path::new("exams_cache.json").exists() {
            exams_cache_content =
                serde_json::from_str(&std::fs::read_to_string("exams_cache.json")?)?;
        } else {
            exams_cache_content = vec![];
        }

        let homework_cache_content: Vec<Homework>;
        if Path::new("homework_cache.json").exists() {
            homework_cache_content =
                serde_json::from_str(&std::fs::read_to_string("homework_cache.json")?)?;
        } else {
            homework_cache_content = vec![];
        }

        // let mut new_exams_list: Vec<Exam> = vec![];
        // let mut new_homework_list: Vec<Homework> = vec![];

        for exam_homework in data {
            // typ == 4 is homework
            if exam_homework.typ == 4 {
                let description = if let Some(cached_homework) = homework_cache_content
                    .iter()
                    .find(|hw| hw.id == exam_homework.id)
                {
                    cached_homework.description.to_owned()
                } else {
                    self.get_homework_details(exam_homework.id)
                        .await?
                        .description
                };
                if self
                    .homework_cache
                    .iter()
                    .find(|homework| homework.id == exam_homework.id)
                    .is_none()
                {
                    self.homework_cache.push(Homework {
                        date: NaiveDateTime::parse_from_str(
                            &exam_homework.data,
                            "%Y-%m-%dT%H:%M:%S%:z",
                        )?
                        .date(),
                        description,
                        subject: exam_homework.przedmiotNazwa,
                        id: exam_homework.id,
                    });
                }
            } else {
                let exam_type = get_exam_type_from_id(exam_homework.typ);
                let description = if let Some(cached_exam) = exams_cache_content
                    .iter()
                    .find(|ex| ex.id == exam_homework.id)
                {
                    cached_exam.description.to_owned()
                } else {
                    self.get_exam_details(exam_homework.id).await?.description
                };
                if self
                    .exams_cache
                    .iter()
                    .find(|exam| exam.id == exam_homework.id)
                    .is_none()
                {
                    self.exams_cache.push(Exam {
                        exam_type,
                        date: NaiveDateTime::parse_from_str(
                            &exam_homework.data,
                            "%Y-%m-%dT%H:%M:%S%:z",
                        )?
                        .date(),
                        description,
                        subject: exam_homework.przedmiotNazwa,
                        id: exam_homework.id,
                    });
                }
            }
        }

        std::fs::write(
            "homework_cache.json",
            serde_json::to_string(&self.homework_cache)?,
        )?;
        std::fs::write(
            "exams_cache.json",
            serde_json::to_string(&self.exams_cache)?,
        )?;

        Ok(())
    }

    async fn get_homework_details(&self, id: u32) -> Result<Homework> {
        let resp = self
            .get(Endpoint::HomeworkDetails, &[("id", &id.to_string())])
            .await;

        let data: HomeworkDetailsResponse = serde_json::from_value(resp?)?;

        Ok(Homework {
            subject: data.przedmiotNazwa,
            description: data.opis,
            date: NaiveDateTime::parse_from_str(&data.data, "%Y-%m-%dT%H:%M:%S%:z")?.date(),
            id: data.id,
        })
    }

    async fn get_exam_details(&self, id: u32) -> Result<Exam> {
        let resp = self
            .get(Endpoint::ExamDetails, &[("id", &id.to_string())])
            .await;

        let data: ExamDetailsResponse = serde_json::from_value(resp?)?;

        Ok(Exam {
            exam_type: get_exam_type_from_id(data.typ),
            subject: data.przedmiotNazwa,
            description: data.opis,
            date: NaiveDateTime::parse_from_str(&data.data, "%Y-%m-%dT%H:%M:%S%:z")?.date(),
            id: data.id,
        })
    }

    pub async fn get_exams(&mut self, date_range: DateTimeRange) -> Result<Vec<Exam>> {
        self.refresh_exams_and_homework_cache(&date_range, false)
            .await?;

        Ok(self
            .exams_cache
            .iter()
            .filter_map(|exam| {
                if exam.date.and_hms_opt(12, 0, 0)? >= date_range.date_start
                    && exam.date.and_hms_opt(12, 0, 0)? <= date_range.date_end
                {
                    return Some(exam.clone());
                } else {
                    return None;
                }
            })
            .collect())
    }

    pub async fn get_homework(&mut self, date_range: DateTimeRange) -> Result<Vec<Homework>> {
        self.refresh_exams_and_homework_cache(&date_range, false)
            .await?;

        Ok(self
            .homework_cache
            .iter()
            .filter_map(|homework| {
                if homework.date.and_hms_opt(12, 0, 0)? >= date_range.date_start
                    && homework.date.and_hms_opt(12, 0, 0)? <= date_range.date_end
                {
                    return Some(homework.clone());
                } else {
                    return None;
                }
            })
            .collect())
    }

    pub async fn get_lucky_number(&self) -> Result<Option<LuckyNumber>> {
        let resp = self.get(Endpoint::LuckyNumber, &[]).await;

        let data: Option<LuckyNumberResponse> = serde_json::from_value(resp?)?;

        Ok(if let Some(data) = data {
            Some(data.numer)
        } else {
            None
        })
    }

    pub async fn refresh_session(&self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(VError::NotLoggedIn.into());
        }

        let url = format!(
            "{}{}",
            self.api_base_url.as_ref().unwrap(),
            get_endpoint(Endpoint::RefreshSession)
        );

        let req = self.http.get(&url).send().await?;

        if req.status() != StatusCode::NO_CONTENT {
            return Err(VError::ApiFormatChanged(
                "RefreshSession".to_owned(),
                "[wrong status code]".to_owned(),
            )
            .into());
        }

        Ok(())
    }
}
