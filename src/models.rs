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
    pub panel_temp_red: f64,
    pub tau: f64,
    pub tau_down: f64,
    pub k_gain: f64,
    pub iam_factor: f64,
}

pub struct Production {
    pub power: Vec<DataItem>,
    pub incidence_east: Vec<DataItem>,
    pub incidence_west: Vec<DataItem>,
    pub ambient_temperature: Vec<DataItem>,
    pub roof_temperature_east: Vec<DataItem>,
    pub roof_temperature_west: Vec<DataItem>,
}
