use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;
use chrono::{DateTime, DurationRound, Local, TimeDelta, TimeZone, Timelike};
use spa_sra::errors::SpaError;
use spa_sra::spa::{Function, Input, SpaData};
use crate::models::{DataItem, Parameters, Production};


/// Returns a vector of production values per minute
///
/// # Arguments
///
/// * 'params' - parameters to use in calculations
pub fn get_day_production(params: Parameters) -> Result<Production, ProdError> {
    let date_time = Local::now()
        .timezone()
        .with_ymd_and_hms(params.year, params.month, params.day, 0, 0, 0)
        .unwrap();

    Ok(day_power(params, date_time)?)
}

/// Calculates one day estimated power per minute
///
/// # Arguments
///
/// * 'params' - struct of parameters
/// * 'date_time' - date to calculate for
fn day_power(params: Parameters, date_time: DateTime<Local>) -> Result<Production, SpaError> {
    let mut power: [f64;1440] = [0.0;1440];
    let sp = solar_positions(date_time, &params)?;
    let roof_temperature_east: [f64;1440] = get_roof_temperature(&params, sp.incidence_east);
    let roof_temperature_west: [f64;1440] = get_roof_temperature(&params, sp.incidence_west);
    let mut ame: [f64;1440] = [0.0;1440];

    let (up, down) = full_sun_minute(&params, &sp);

    // Loop through the day with a one-minute incrementation
    for minute_of_day in sp.sunrise..sp.sunset {
        // Calculate factor on power production given sun incidence angles
        let inc_red_e = schlick_iam(sp.incidence_east[minute_of_day], Some(params.iam_factor));
        let inc_red_w = schlick_iam(sp.incidence_west[minute_of_day], Some(params.iam_factor));

        // Calculate power reduction due to high temperatures
        let temp_red_e = 1.0 - (roof_temperature_east[minute_of_day].max(0.0) - 25.0) * params.panel_temp_red / 100.0;
        let temp_red_w = 1.0 - (roof_temperature_west[minute_of_day].max(0.0) - 25.0) * params.panel_temp_red / 100.0;

        // Calculate power reduction due to the atmospheric effect given sun altitude relative to zenith
        let ame_red = air_mass_effect(sp.zenith[minute_of_day]);
        ame[minute_of_day] = ame_red;

        // Calculate total panel power where each side is reduced given the above power reduction factors
        let pwr = params.panel_power * 12.0 * inc_red_e * temp_red_e + params.panel_power * 15.0 * inc_red_w * temp_red_w;

        // Calculate the shadow factors for the given minute of the day
        let shadow_up = exp_increase(minute_of_day, sp.sunrise, up, 10);
        let shadow_down = exp_decrease(minute_of_day, down, sp.sunset, 4);

        // Record the estimated power at the given point in time
        power[minute_of_day] = (pwr * ame_red * shadow_up * shadow_down) / 1000.0;
    }

    Ok(Production {
        power: prepare_result(date_time, &power),
        incidence_east: prepare_result(date_time, &sp.incidence_east),
        incidence_west: prepare_result(date_time, &sp.incidence_west),
        ambient_temperature: prepare_result(date_time, &params.temp),
        roof_temperature_east: prepare_result(date_time, &roof_temperature_east),
        roof_temperature_west: prepare_result(date_time, &roof_temperature_west),
        air_mass_effect: prepare_result(date_time, &ame),
    })
}

/// Finds the points in time (minute) where the sun is free from nearby obstacles
///
/// # Arguments
///
/// * 'params' - input parameters
/// * 'solar_positions' - solar positions during the day
fn full_sun_minute(params: &Parameters, solar_positions: &SolarPositions) -> (usize, usize) {
    let mut up: usize = 0;
    let mut down: usize = 0;

    for m in solar_positions.sunrise..solar_positions.sunset {
        if up == 0 && (solar_positions.elevation[m] > params.start_elv || solar_positions.azimuth[m] > params.start_azm) {
            up = m;
        }
        if down == 0 && solar_positions.azimuth[m] > 180.0 && (solar_positions.elevation[m] < params.stop_elv || solar_positions.azimuth[m] > params.stop_azm) {
            down = m;
            break;
        }
    }

    (up,down)
}

/// Calculates and exponential increase for v between v0 and vn
/// The output is an exponential factor between 0 and 1
///
/// # Arguments
///
/// * 'v' - the input value
/// * 'v0' - the starting point for when v influences the output
/// * 'vn' - the end point for when v gives an output of 1 and no longer influences output
/// * 'exp' - exponent that determines the exponential shape
fn exp_increase(v: usize, v0: usize, vn: usize, exp: i32) -> f64 {
    let denominator = (vn - v0) as f64;
    let enumerator = (v as f64 - v0 as f64).clamp(0.0, denominator);

    (enumerator / denominator).powi(exp)
}

/// Calculates and exponential decrease for v between v0 and vn
/// The output is an exponential decrease factor between 1 and 0
///
/// # Arguments
///
/// * 'v' - the input value
/// * 'v0' - the starting point for when v influences the output
/// * 'vn' - the end point for when v gives an output of 1 and no longer influences output
/// * 'exp' - exponent that determines the exponential shape
fn exp_decrease(v: usize, v0: usize, vn: usize, exp: i32) -> f64 {
    let denominator = (vn - v0) as f64;
    let enumerator = (vn as f64 - v as f64).clamp(0.0, denominator);

    (enumerator / denominator).powi(exp)
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


/// Returns percentage of sun intensity relative to extra‑terrestrial irradiance (I0),
/// including effects of air mass geometry, air pressure (absolute air mass), and water vapor.
///
/// Arguments:
/// - zenith_angle: Sun angle relative to zenith [deg]
/// - pressure_hpa: Optional surface pressure [hPa] (defaults to 1013.25)
/// - rel_humidity: Optional relative humidity [0..1]
/// - temperature_c: Optional near-surface air temperature [°C] (used with RH to estimate water vapor)
fn air_mass_effect_with_met(
    zenith_angle: f64,
    pressure_hpa: Option<f64>,
    rel_humidity: Option<f64>,
    temperature_c: Option<f64>,
) -> f64 {
    const R: f64 = 708.0;
    // Intensity external to the earth's atmosphere
    const I_0: f64 = 1353.0;
    const P0_HPA: f64 = 1013.25;

    let zenith_cos = zenith_angle.to_radians().cos().max(0.0);
    let enumerator = 2.0 * R + 1.0;
    let denominator = ((R * zenith_cos).powf(2.0) + 2.0 * R + 1.0).sqrt() + R * zenith_cos;

    // Geometric air mass
    let am = enumerator / denominator;

    // Pressure-corrected (absolute) air mass
    let p = pressure_hpa.unwrap_or(P0_HPA).clamp(800.0, 1050.0);
    let am_abs = am * (p / P0_HPA);

    // Base broadband clear-sky attenuation (same shape you had, now using am_abs)
    let base_intensity = 1.1 * I_0 * 0.7f64.powf(am_abs.powf(0.678));

    // Optional water-vapor attenuation term (very lightweight approximation).
    // Estimate precipitable water (w, cm) from surface RH and T via vapor pressure.
    // Note: This is a crude proxy; for best results feed precipitable water directly if available.
    let tw = if let (Some(rh), Some(t_c)) = (rel_humidity, temperature_c) {
        let rh = rh.clamp(0.0, 1.0);
        // Saturation vapor pressure (Tetens, hPa)
        let es = saturation_vapor_pressure_hpa(t_c);
        // Actual vapor pressure (hPa)
        let e = rh * es;
        // Very rough mapping from surface vapor pressure to column water (cm), bounded
        // (typical PWV ~0.5–5 cm; clamp to [0, 7] to avoid pathological values)
        let w_cm = (0.1 * e).clamp(0.0, 7.0);

        // Water vapor transmittance (empirical; keep within [0.6, 1.0] for stability)
        (1.0 - 0.077 * (w_cm * am_abs).powf(0.3)).clamp(0.6, 1.0)
    } else {
        1.0
    };

    let intensity = base_intensity * tw;

    // return percentage of intensity compared to I_0
    intensity / I_0
}

/// Saturation vapor pressure over liquid water (Tetens) in hPa given temperature in °C.
fn saturation_vapor_pressure_hpa(t_c: f64) -> f64 {
    // Valid and smooth for typical surface temps
    6.112 * (17.67 * t_c / (t_c + 243.5)).exp()
}



/// Prepares a result vector of data items
///
/// # Arguments
///
/// * 'date_time' - date time truncated to day
fn prepare_result(date_time: DateTime<Local>, default: &[f64]) -> Vec<DataItem> {
    (0..1440)
        .into_iter()
        .map(|i| DataItem{x: date_time.add(TimeDelta::minutes(i)), y: default[i as usize]})
        .collect::<Vec<DataItem>>()
}

#[derive(Debug)]
pub struct ProdError(pub String);
impl fmt::Display for ProdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ProdError: {}", self.0)
    }
}
impl From<SpaError> for ProdError {
    fn from(e: SpaError) -> Self { ProdError(e.to_string()) }
}

/// Calculates roof temperature given ambient temperature and effect from direct sunlight
///
/// # Arguments
///
/// * 'params' - parameters
/// * 'inc_deg' - sun incidence on panels in degrees
fn get_roof_temperature(params: &Parameters, inc_deg: [f64;1440]) -> [f64;1440] {

    let t_roof = roof_temperature(
        &params.temp,
        &inc_deg,
        60.0,
        params.tau * 3600.0,
        params.k_gain,
        None,
        None,
        Some(params.tau_down * 3600.0));

    let mut result: [f64;1440] = [0.0; 1440];
    (0..1440)
        .into_iter()
        .for_each(|i| {
            result[i] = t_roof[i];
        });

    result
}


/// Returns sun incidence, zenith, azimuth and elevation angles per minute in degrees for the given date.
///
/// # Arguments
///
/// * 'date_time' - DateTime object carrying date of interest
/// * 'params' - various input parameters
fn solar_positions(date_time: DateTime<Local>, params: &Parameters) -> Result<SolarPositions, SpaError> {
    let mut input = Input::from_date_time(date_time);
    input.latitude = params.lat;
    input.longitude = params.long;
    input.pressure = 1013.0;
    input.temperature = 10.0;
    input.elevation = 61.0;
    input.slope = params.panel_slope;
    input.azm_rotation = 0.0;
    input.function = Function::SpaZaRts;

    let mut spa = SpaData::new(input);
    spa.spa_calculate()?;

    let sunrise = spa.get_sunrise().duration_round(TimeDelta::minutes(1)).unwrap();
    let sunset = spa.get_sunset().duration_round(TimeDelta::minutes(1)).unwrap();

    spa.input.function = Function::SpaZaInc;

    let mut time_of_interest = sunrise;

    let mut incidence_east: [f64;1440] = [90.0; 1440];
    let mut incidence_west: [f64;1440] = [90.0; 1440];
    let mut zenith: [f64;1440] = [90.0; 1440];
    let mut azimuth: [f64;1440] = [0.0; 1440];
    let mut elevation: [f64;1440] = [0.0; 1440];

    while time_of_interest < sunset {
        spa.input.date_time(time_of_interest);
        let toi = (time_of_interest.hour() * 60 + time_of_interest.minute()) as usize;

        spa.input.azm_rotation = params.panel_east_azm;
        spa.spa_calculate()?;

        incidence_east[toi] = spa.spa_za_inc.incidence.min(90.0);
        zenith[toi] = spa.spa_za.zenith.clamp(0.0, 90.0);
        azimuth[toi] = spa.spa_za.azimuth;
        elevation[toi] = spa.spa_za.e.max(0.0);

        spa.input.azm_rotation = 180.0 + params.panel_east_azm;
        spa.spa_calculate()?;
        incidence_west[toi] = spa.spa_za_inc.incidence.min(90.0);

        time_of_interest = time_of_interest.add(TimeDelta::minutes(1));
    }


    Ok(SolarPositions {
        incidence_east,
        incidence_west,
        azimuth,
        elevation,
        zenith,
        sunrise: (sunrise.hour() * 60 + sunrise.minute()) as usize,
        sunset: (sunset.hour() * 60 + sunset.minute()) as usize,
    })
}

/// Roof temperature over time using a 1st-order thermal RC model.
///
/// State update (explicit Euler):
///   T_roof[k] = T_roof[k-1] + (T_eq - T_roof[k-1]) * (dt / tau_eff)
/// where:
///   T_eq = T_air[k] + K * max(0, cos(inc_deg[k])) * clouds[k]
///   tau_eff = tau (when heating) or tau_down.unwrap_or(tau) (when cooling)
///
/// Notes:
/// - inc_deg is the sun incidence angle relative to the roof normal (0 deg = perpendicular to roof).
///   For a horizontal roof, inc_deg = 90 - altitude_deg.
/// - cos(inc_deg) gives the direct-beam projection onto the roof plane and is clamped at 0.
///
/// # Arguments
/// * `t_air`    : ambient air temperature [°C], length N
/// * `inc_deg`  : sun incidence angle to the roof normal [degrees], length N
/// * `dt`       : timestep [s], e.g. 600.0
/// * `tau`      : time constant for heating [s]
/// * `k_gain`   : °C boost at clear-sky normal incidence (proxy for A*α*G_max/U)
/// * `clouds`   : optional attenuation array in [0,1], length N (defaults to 1.0)
/// * `t0`       : optional initial roof temperature [°C] (defaults to t_air[0])
/// * `tau_down` : optional time constant for cooling [s] (defaults to `tau`)
///
/// # Returns
/// Vector of roof temperatures [°C], length N.
///
/// # Panics
/// Panics if input lengths mismatch or if `dt <= 0.0` or any tau ≤ 0.0.
pub fn roof_temperature(
    t_air: &[f64],
    inc_deg: &[f64],
    dt: f64,
    tau: f64,
    k_gain: f64,
    clouds: Option<&[f64]>,
    t0: Option<f64>,
    tau_down: Option<f64>,
) -> Vec<f64> {
    let n = t_air.len();
    if n == 0 {
        return Vec::new();
    }
    assert_eq!(inc_deg.len(), n, "inc_rad must match t_air length");
    if let Some(c) = clouds {
        assert_eq!(c.len(), n, "clouds must match t_air length");
    }
    assert!(dt > 0.0, "dt must be > 0");
    assert!(tau > 0.0, "tau must be > 0");
    if let Some(td) = tau_down {
        assert!(td > 0.0, "tau_down must be > 0");
    }

    let mut t_roof = vec![0.0; n];
    t_roof[0] = t0.unwrap_or(t_air[0]);
    let tau_cool = tau_down.unwrap_or(tau);

    for k in 1..n {
        // clouds[k] defaults to 1.0 if not provided
        let cloud_k = clouds.map_or(1.0, |c| c[k]);
        // Use projection by incidence: cos(inc_rad), clamped to [0, +inf) at 0.
        let projection = inc_deg[k].to_radians().cos().max(0.0);
        let sun_boost = k_gain * projection * cloud_k; // [°C]
        let t_eq = t_air[k] + sun_boost;

        let tau_eff = if t_eq > t_roof[k - 1] { tau } else { tau_cool };
        let alpha = dt / tau_eff; // Euler gain

        t_roof[k] = t_roof[k - 1] + (t_eq - t_roof[k - 1]) * alpha;
    }

    t_roof
}

struct SolarPositions {
    incidence_east: [f64;1440],
    incidence_west: [f64;1440],
    azimuth: [f64;1440],
    elevation: [f64;1440],
    zenith: [f64;1440],
    sunrise: usize,
    sunset: usize,
}
