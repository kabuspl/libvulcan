#![allow(non_snake_case)]
#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LuckyNumberResponse {
    pub numer: u8,
    pub id: u32,
}
