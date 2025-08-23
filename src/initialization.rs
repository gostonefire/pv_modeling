use std::{env, fs};
use log::LevelFilter;
use serde::Deserialize;
use crate::errors::ConfigError;
use crate::logging::setup_logger;

#[derive(Deserialize, Clone)]
pub struct GeoRef {
    pub lat: f64,
    pub long: f64,
}

#[derive(Deserialize, Clone)]
pub struct FoxESS {
    pub api_key: String,
    pub inverter_sn: String,
}

#[derive(Deserialize, Clone)]
pub struct Weather {
    pub host: String,
    pub sensor: String,
}

#[derive(Deserialize, Clone)]
pub struct Files {
    pub cache_dir: String,
}

#[derive(Deserialize, Clone)]
pub struct WebServerParameters {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct General {
    pub log_path: String,
    pub log_level: LevelFilter,
    pub log_to_stdout: bool,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub geo_ref: GeoRef,
    pub fox_ess: FoxESS,
    pub weather: Weather,
    pub files: Files,
    pub web_server: WebServerParameters,
    pub general: General,
}

/// Returns a configuration struct for the application and starts logging
///
pub fn config() -> Result<Config, ConfigError> {
    let args: Vec<String> = env::args().collect();
    let config_path = args.iter()
        .find(|p| p.starts_with("--config="))
        .ok_or(ConfigError::from("missing --config=<config_path>"))?;
    let config_path = config_path
        .split_once('=')
        .ok_or(ConfigError::from("invalid --config=<config_path>"))?
        .1;

    let config = load_config(&config_path)?;

    setup_logger(&config.general.log_path, config.general.log_level, config.general.log_to_stdout)?;

    Ok(config)
}

/// Loads the configuration file and returns a struct with all configuration items
///
/// # Arguments
///
/// * 'config_path' - path to the config file
fn load_config(config_path: &str) -> Result<Config, ConfigError> {

    let toml = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&toml)?;

    Ok(config)
}
