use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DataItem {
    pub x: i64,
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
    pub iam_factor: f64,
}