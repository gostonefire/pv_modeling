use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct WeatherError(pub String);
impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WeatherError: {}", self.0)
    }
}
impl From<&str> for WeatherError {
    fn from(e: &str) -> Self { WeatherError(e.to_string()) }
}
impl From<reqwest::Error> for WeatherError {
    fn from(e: reqwest::Error) -> Self { WeatherError(e.to_string()) }
}
impl From<serde_json::Error> for WeatherError {
    fn from(e: serde_json::Error) -> Self { WeatherError(e.to_string()) }
}
impl From<std::io::Error> for WeatherError {
    fn from(e: std::io::Error) -> Self { WeatherError(e.to_string()) }
}