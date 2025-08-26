pub mod errors;
mod models;

use std::ops::Add;
use std::time::Duration;
use chrono::{DateTime, DurationRound, Local, TimeDelta};
use reqwest::Client;
use crate::cache::{read_cache_data, store_cache_data};
use crate::manager_weather::errors::WeatherError;
use crate::manager_weather::models::WeatherItem;
use crate::models::DataItem;

const CACHE_PREFIX: &str = "temp";

/// Weather manager
/// 
pub struct Weather {
    client: Client,
    host: String,
    sensor: String,
}

impl Weather {

    /// Returns a new instance of Weather
    /// 
    /// # Arguments
    /// 
    /// * 'host' - host running the weather logger service
    /// * 'sensor' - name of sensor to get weather data for
    pub fn new(host: &str, sensor: &str) -> Result<Self, WeatherError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, host: host.to_string(), sensor: sensor.to_string() })
    }
    
    /// Returns the temperature history from the given date
    /// 
    /// # Arguments
    /// 
    /// * 'date_time' - date to get history for
    /// * 'cache_dir' - directory to store/fetch existing date to/from
    pub async fn get_temp_history(&self, date_time: DateTime<Local>, cache_dir: &str) -> Result<[f64;1440], WeatherError> {
        let result = if let Some(result) = read_cache_data(cache_dir, CACHE_PREFIX, date_time).await? {
            result
        } else {
            let url = format!("http://{}/temperature", self.host);

            let from = date_time.duration_trunc(TimeDelta::days(1)).unwrap();
            let to = from.add(TimeDelta::days(1)).add(TimeDelta::minutes(-1));

            let req = self.client.get(&url)
                .query(&[("id", &self.sensor), ("from", &from.to_rfc3339()), ("to", &to.to_rfc3339())])
                .send().await?;

            let status = req.status();
            if !status.is_success() {
                return Err(WeatherError(format!("{:?}", status)));
            }

            let json = req.text().await?;
            let weather_res: Vec<WeatherItem> = serde_json::from_str(&json)?;

            let result = transform_history(weather_res, from, to);
            store_cache_data(cache_dir, CACHE_PREFIX, date_time, &result).await?;

            result
        };

        Ok(fill_minutes(result))
    }
}

/// Returns a copy of the given data but with very minute filled with data
///
/// # Arguments
///
/// * 'data' - a vector to fill in the blanks for
fn fill_minutes(data: Vec<DataItem>) -> [f64;1440] {
    let mut result: [f64;1440] = [0.0;1440];
    if data.len() < 2 {
        return result;
    }

    let mut pit= data[0].x;
    let mut idx: usize = 0;
    let mut pit_data: f64 = 0.0;

    for di in data.into_iter() {
        while di.x > pit {
            result[idx] = pit_data;
            pit = pit.add(TimeDelta::minutes(1));
            idx += 1;
        }
        pit_data = di.y;
    }
    result[idx] = pit_data;

    result
}

/// Transforms the history from the weather database to a per minute vector
///
/// While doing so the transformation also ensures that the 'to' date has a data item, and
/// possibly also the 'from' date
/// 
/// # Arguments
/// 
/// * 'history' - the history data to transform
/// * 'from' - from date to include with a data item
/// * 'to' - to date to include with a data item
fn transform_history(history: Vec<WeatherItem>, from: DateTime<Local>, to: DateTime<Local>) -> Vec<DataItem> {
    let mut result: Vec<DataItem> = Vec::new();
    
    if history.len() == 0 {
        result
    } else {
        history.into_iter().for_each(|w| {
            result.push(DataItem{
                x: w.x.duration_trunc(TimeDelta::minutes(1)).unwrap()
                , y: w.y
            });
        });
        
        if result[0].x != from {
            result.insert(0, DataItem{x: from, y: result[0].y});
        }

        let last = result.len() - 1;
        if result[last].x != to {
            result.push(DataItem{x: to, y: result[last].y});
        }
        
        result
    }
}