use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use crate::serialize_timestamp;

#[derive(Serialize, Deserialize)]
pub struct DataItem {
    #[serde(with = "serialize_timestamp")]
    pub x: DateTime<Local>,
    pub y: f64,
}

pub struct Parameters {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub lat: f64,
    pub long: f64,
    pub temp: [f64;1440],
    pub panel_power: f64,
    pub panel_slope: f64,
    pub panel_east_azm: f64,
    pub panel_add_temp: f64,
    pub panel_temp_red: f64,
    pub iam_factor: f64,
}