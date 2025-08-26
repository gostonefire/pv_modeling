use std::ops::Add;
use chrono::{DateTime, DurationRound, Local, TimeDelta, TimeZone, Timelike};
use spa_sra::errors::SpaError;
use spa_sra::spa::{Function, Input, SpaData};
use crate::models::{DataItem, Parameters};


/// Returns a vector of production values per minute
///
/// # Arguments
///
/// * 'params' - parameters to use in calculations
pub fn get_day_production(params: Parameters) -> Vec<DataItem> {
    let date_time = Local::now()
        .timezone()
        .with_ymd_and_hms(params.year, params.month, params.day, 0, 0, 0)
        .unwrap();

    let start = date_time;

    let mut result = (0..1440)
        .into_iter()
        .map(|i| DataItem{x: start.add(TimeDelta::minutes(i)), y: 0.0})
        .collect::<Vec<DataItem>>();

    if let Ok(minute_power) = day_power(params, date_time) {
        minute_power.into_iter().enumerate().for_each(|(i,y)| {result[i].y = y / 1000.0});
    }

    result
}

/// Calculates one day estimated power per minute
///
/// # Arguments
///
/// * 'params' - struct of parameters
/// * 'date_time' - date to calculate for
fn day_power(params: Parameters, date_time: DateTime<Local>) -> Result<[f64;1440], SpaError> {
    let mut result: [f64;1440] = [0.0;1440];

    // Create an Input instance with relevant parameters, and we are happy with the
    // defaults for atmospheric_refraction, delta_ut1 and delta_t.
    // We only need sunrise and sunset first, so we chose a specific function for that
    let mut input = Input::from_date_time(date_time);
    input.latitude = params.lat;
    input.longitude = params.long;
    input.pressure = 1013.0;
    input.temperature = 10.0;
    input.elevation = 61.0;
    input.slope = params.panel_slope;
    input.azm_rotation = params.panel_east_azm;
    input.function = Function::SpaZaRts;

    // Create and calculate a SpaData struct
    let mut spa = SpaData::new(input);
    spa.spa_calculate()?;

    // Now we got sunrise and sunset for the chosen day, round to whole minutes
    let sunrise = spa.get_sunrise().duration_round(TimeDelta::minutes(1)).unwrap();
    let sunset = spa.get_sunset().duration_round(TimeDelta::minutes(1)).unwrap();
    let mut time_of_interest = sunrise;

    // We are only interested in the incidence from now on so we set that function
    spa.input.function = Function::SpaZaInc;

    // Loop through the day with a one minute incrementation and save the incidence to result
    while time_of_interest < sunset {
        let minute_of_day = (time_of_interest.hour() * 60 + time_of_interest.minute()) as usize;

        spa.input.date_time(time_of_interest);

        // Calculate factor on power production given sun incidence angle for eastward panels
        spa.input.azm_rotation = params.panel_east_azm;
        spa.spa_calculate()?;
        let idx_e = schlick_iam(spa.spa_za_inc.incidence, Some(params.iam_factor));

        // Calculate factor on power production given sun incidence angle for westward panels
        spa.input.azm_rotation = 180.0 + params.panel_east_azm;
        spa.spa_calculate()?;
        let idx_w = schlick_iam(spa.spa_za_inc.incidence, Some(params.iam_factor));

        // Calculate total panel power where each side is reduced given incidence angles
        let pwr = params.panel_power * 12.0 * idx_e + params.panel_power * 15.0 * idx_w;

        // Calculate the atmospheric effect given sun altitude
        let ame = air_mass_effect(spa.spa_za.zenith);

        // Calculate air temperature reduction factor on power generation
        // We assume max panel temperature is air temp plus 66 (given from general approximation of
        // high panel temp in direct sunlight). Rule of thumb is 0.5 percentage reduction per
        // degree Celsius above 27 degree Celsius. Since sun intensity is directly related to
        // the panel material temperature we use the reduction of sun intensity (ame) to also reduce
        // the estimated panel temperature given time of day.
        let t_red = 1.0 - ((params.panel_add_temp + params.temp[minute_of_day] - 25.0).max(0.0) * params.panel_temp_red * ame) / 100.0;

        // Record the estimated power at the given point in time
        result[minute_of_day] = pwr * ame * t_red;

        time_of_interest = time_of_interest.add(TimeDelta::minutes(1));
    }

    Ok(result)
}

/// Returns max sun elevation in degrees for the given date
///
/// # Arguments
///
/// * 'date_time' - DateTime object carrying date of interest
fn max_elevation(date_time: DateTime<Local>) -> Result<f64, SpaError> {
    let mut input = Input::from_date_time(date_time);
    input.latitude = 56.223306;
    input.longitude = 15.658389;
    input.pressure = 1013.0;
    input.temperature = 10.0;
    input.elevation = 61.0;
    input.function = Function::SpaZaRts;

    let mut spa = SpaData::new(input);
    spa.spa_calculate()?;

    let sunrise = spa.get_sunrise();
    let sunset = spa.get_sunset();
    let mut time_of_interest = sunrise;

    spa.input.function = Function::SpaZa;
    let mut max_elevation: f64 = 0.0;

    while time_of_interest < sunset {
        spa.input.date_time(time_of_interest);
        spa.spa_calculate()?;
        max_elevation = max_elevation.max(spa.spa_za.e);

        time_of_interest = time_of_interest.add(TimeDelta::minutes(1));
    }

    Ok(max_elevation)
}

/// The Schlick Incidence Angle Modifier algorithm
///
/// # Arguments
///
/// * 'theta_deg' - Sun-panel incidence angle
/// * 'factor' - level of flatness, 1 gives cosine flatness, higher values gives more flatness
pub fn schlick_iam(theta_deg: f64, factor: Option<f64>) -> f64 {
    // Handle NaN/inf robustly.
    if !theta_deg.is_finite() {
        return 0.0;
    }

    // Model is symmetric in angle; anything beyond 90° contributes zero.
    let theta = theta_deg.abs();
    if theta >= 90.0 {
        return 0.0;
    }

    let factor = factor.unwrap_or(5.0);

    // The Schlick IAM formula
    1.0 - (1.0 - theta.to_radians().cos()).powf(factor)
}

/// Returns percentage of sun intensity in relation to intensity external to the earth's atmosphere.
///
/// # Arguments
///
/// * 'zenith_angle' - sun angle in relation to sun zenith
fn air_mass_effect(zenith_angle: f64) -> f64 {
    const R: f64 = 708.0;

    // Intensity external to the earths atmosphere
    const I_0: f64 = 1353.0;

    let zenith_cos = zenith_angle.to_radians().cos();
    let enumerator = 2.0 * R + 1.0;
    let denominator = ((R * zenith_cos).powf(2.0) + 2.0 * R + 1.0).sqrt() + R * zenith_cos;

    let am = enumerator / denominator;
    let intensity = 1.1 * I_0 * 0.7f64.powf(am.powf(0.678));

    // return percentage of intensity compared to I_0
    intensity / I_0
}