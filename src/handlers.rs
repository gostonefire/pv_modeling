use chrono::{Local, TimeZone};
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::fs::{read_to_string, write};
use crate::AppState;
use crate::initialization::Config;
use crate::manager_fox_cloud::Fox;
use crate::manager_production::get_day_production;
use crate::manager_weather::Weather;
use crate::models::{DataItem, Parameters};

#[derive(Deserialize, Serialize)]
struct Params {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub panel_power: f64,
    pub panel_slope: f64,
    pub panel_east_azm: f64,
    pub iam_factor: f64,
}

#[get("/get_data")]
pub async fn get_data(data: web::Data<AppState>, params: web::Query<Params>) -> impl Responder {
    let json = get_web_data(&data.config, &params).await;
    save_parameters(&data.config.files.cache_dir, &params).await;

    HttpResponse::Ok().body(json)
}

#[get("/get_start")]
pub async fn get_start(data: web::Data<AppState>) -> impl Responder {
    let params = load_parameters(&data.config.files.cache_dir).await;
    let json = get_web_data(&data.config, &params).await;

    HttpResponse::Ok().body(json)
}

async fn load_parameters(cache_dir: &str) -> Params {
    let path = format!("{}parameters.json", cache_dir);

    let json = read_to_string(path).await.unwrap();
    let params: Params = serde_json::from_str(&json).unwrap();

    params
}

async fn save_parameters(cache_dir: &str, params: &Params) {
    let path = format!("{}parameters.json", cache_dir);
    let json = serde_json::to_string(&params).unwrap();

    write(path, json).await.unwrap();
}

async fn get_web_data(config: &Config, params: &Params) -> String {
    let date_time = Local::now()
        .timezone()
        .with_ymd_and_hms(params.year, params.month, params.day, 0, 0, 0)
        .unwrap();

    let temp = Weather::new(&config.weather.host, &config.weather.sensor)
        .unwrap()
        .get_temp_history(date_time, &config.files.cache_dir).await.unwrap();

    let history = Fox::new(&config.fox_ess)
        .unwrap()
        .get_device_history_data(date_time, &config.files.cache_dir).await.unwrap();

    let production_params = Parameters {
        year: params.year,
        month: params.month,
        day: params.day,
        lat: config.geo_ref.lat,
        long: config.geo_ref.long,
        temp,
        panel_power: params.panel_power,
        panel_slope: params.panel_slope,
        panel_east_azm: params.panel_east_azm,
        iam_factor: params.iam_factor,
    };

    let estimated = get_day_production(production_params);

    #[derive(Serialize)]
    pub struct Series {
        pub name: String,
        #[serde(rename(serialize = "type"))]
        pub chart_type: String,
        pub data: Vec<DataItem>,
    }
    #[derive(Serialize)]
    struct WebData<'a> {
        prod_diagram: (Series, Series),
        params: &'a Params,
    }

    let web_data = WebData {
        prod_diagram: (Series {
            name: "Actual".to_string(),
            chart_type: "area".to_string(),
            data: history,
        }, Series {
            name: "Estimated".to_string(),
            chart_type: "line".to_string(),
            data: estimated,
        }),
        params,

    };

    let json = serde_json::to_string(&web_data).unwrap();

    json
}