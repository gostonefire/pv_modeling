use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeatherItem {
    pub x: DateTime<Local>,
    pub y: f64,
}
