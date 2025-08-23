pub mod errors;
mod models;

use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, DurationRound, Local, NaiveDateTime, TimeDelta, Utc};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::cache::{read_cache_data, store_cache_data};
use crate::initialization::FoxESS;
use crate::manager_fox_cloud::errors::FoxError;
use crate::manager_fox_cloud::models::{DeviceHistoryData, DeviceHistoryResult, RequestDeviceHistoryData};
use crate::models::DataItem;

const REQUEST_DOMAIN: &str = "https://www.foxesscloud.com";
const CACHE_PREFIX: &str = "pv";

pub struct Fox {
    api_key: String,
    sn: String,
    client: Client,
}

impl Fox {
    /// Returns a new instance of the Fox struct
    ///
    /// # Arguments
    ///
    /// * 'config' - FoxESS configuration struct
    pub fn new(config: &FoxESS) -> Result<Self, FoxError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { api_key: config.api_key.to_string(), sn: config.inverter_sn.to_string(), client })
    }

    /// Obtain history data from the inverter
    ///
    /// See https://www.foxesscloud.com/public/i18n/en/OpenApiDocument.html#get20device20history20data0a3ca20id3dget20device20history20data4303e203ca3e
    ///
    /// # Arguments
    ///
    /// * 'date_time' - date to get history for
    /// * 'cache_dir' - directory to store/fetch existing date to/from
    pub async fn get_device_history_data(&self, date_time: DateTime<Local>, cache_dir: &str) -> Result<Vec<DataItem>, FoxError> {
        if let Some(result) = read_cache_data(cache_dir, CACHE_PREFIX, date_time).await? {
            return Ok(result);
        }
        
        let path = "/op/v0/device/history/query";

        let start = date_time.duration_trunc(TimeDelta::days(1)).unwrap().with_timezone(&Utc);
        let end = start.add(TimeDelta::days(1)).add(TimeDelta::seconds(-1));

        let req = RequestDeviceHistoryData {
            sn: self.sn.clone(),
            variables: ["pvPower"]
                .iter().map(|s| s.to_string())
                .collect::<Vec<String>>(),
            begin: start.timestamp_millis(),
            end: end.timestamp_millis(),
        };

        let req_json = serde_json::to_string(&req)?;

        let json = self.post_request(&path, req_json).await?;

        let fox_data: DeviceHistoryResult = serde_json::from_str(&json)?;
        let device_history = transform_history_data(fox_data.result)?;

        store_cache_data(cache_dir, CACHE_PREFIX, date_time, &device_history).await?;
        
        Ok(device_history)
    }

    /// Builds a request and sends it as a POST.
    /// The return is the json representation of the result as specified by
    /// respective FoxESS API
    ///
    /// # Arguments
    ///
    /// * path - the API path excluding the domain
    /// * body - a string containing the payload in json format
    async fn post_request(&self, path: &str, body: String) -> Result<String, FoxError> {
        let url = format!("{}{}", REQUEST_DOMAIN, path);

        //let mut req = self.client.post(url);
        let headers = self.generate_headers(&path, Some(vec!(("Content-Type", "application/json"))));

        let req = self.client.post(url)
            .headers(headers)
            .body(body)
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(FoxError::FoxCloud(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let fox_res: FoxResponse = serde_json::from_str(&json)?;
        if fox_res.errno != 0 {
            return Err(FoxError::FoxCloud(format!("errno: {}, msg: {}", fox_res.errno, fox_res.msg)));
        }

        Ok(json)
    }

    /// Generates http headers required by Fox Open API, this includes also building a
    /// md5 hashed signature.
    ///
    /// # Arguments
    ///
    /// * 'path' - the path, excluding the domain part, to the FoxESS specific API
    /// * 'extra' - any extra headers to add besides FoxCloud standards
    fn generate_headers(&self, path: &str, extra: Option<Vec<(&str, &str)>>) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let timestamp = Utc::now().timestamp() * 1000;
        let signature = format!("{}\\r\\n{}\\r\\n{}", path, self.api_key, timestamp);

        let mut hasher = Md5::new();
        hasher.update(signature.as_bytes());
        let signature_md5 = hasher.finalize().iter().map(|x| format!("{:02x}", x)).collect::<String>();

        headers.insert("token", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert("timestamp", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert("signature", HeaderValue::from_str(&signature_md5).unwrap());
        headers.insert("lang", HeaderValue::from_str("en").unwrap());

        if let Some(h) = extra {
            h.iter().for_each(|&(k, v)| {
                headers.insert(HeaderName::from_str(k).unwrap(), HeaderValue::from_str(v).unwrap());
            });
        }

        headers
    }
}

/// Transforms device history data to a format easier to save as non-json file
///
/// # Arguments
///
/// * 'input' - the data to transform
fn transform_history_data(input: Vec<DeviceHistoryData>) -> Result<Vec<DataItem>, FoxError> {
    let mut result: Vec<DataItem> = Vec::new();

    for set in &input[0].data_set {
        if set.variable == "pvPower" {
            for data in &set.data {
                let timestamp = NaiveDateTime::parse_from_str(&data.time, "%Y-%m-%d %H:%M:%S %Z")?
                    .and_local_timezone(Local).unwrap()
                    .timestamp_millis();

                result.push(DataItem{x: timestamp, y: data.value});
            }
        }
    }

    Ok(result)
}

#[derive(Serialize, Deserialize)]
struct FoxResponse {
    errno: u32,
    msg: String,
}


