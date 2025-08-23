use chrono::{DateTime, Local};
use tokio::fs::{read_to_string, write};
use crate::models::DataItem;

/// Writes history data to file
///
/// # Arguments
///
/// * 'cache_dir' - directory to store data in
/// * 'prefix' - prefix to identify source
/// * 'date_time' - date to use as name for the file to create
/// * 'data' - data to store
pub async fn store_cache_data(cache_dir: &str, prefix: &str, date_time: DateTime<Local>, data: &Vec<DataItem>) -> Result<(), std::io::Error> {
    let name = date_time.format("%Y-%m-%d").to_string();
    let path = format!("{}{}-{}.json", cache_dir, prefix, name);

    let json = serde_json::to_string(data)?;
    write(path, json).await?;

    Ok(())
}


/// Tries to read history data from file
///
/// # Arguments
///
/// * 'cache_dir' - directory to read data from
/// * 'prefix' - prefix to identify source
/// * 'date_time' - date to use as name for the file to read
pub async fn read_cache_data(cache_dir: &str, prefix: &str, date_time: DateTime<Local>) -> Result<Option<Vec<DataItem>>, std::io::Error> {
    let name = date_time.format("%Y-%m-%d").to_string();
    let path = format!("{}{}-{}.json", cache_dir, prefix, name);

    if let Ok(json) = read_to_string(path).await {
        let result: Vec<DataItem> = serde_json::from_str(&json)?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}