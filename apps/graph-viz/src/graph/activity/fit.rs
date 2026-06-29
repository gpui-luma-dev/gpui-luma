use std::convert::TryInto;

use anyhow::{Context, Result, bail};
use fitparser::profile::MesgNum;
use fitparser::{FitDataRecord, Value, from_bytes};

use super::lap::LapSummary;
use super::model::{RideActivity, RideSummary, TelemetryPoint};
use super::stats::ActivityStats;

const SEMICIRCLE_TO_DEGREES: f64 = 180.0 / 2_147_483_648.0;

pub fn parse_fit_bytes(bytes: &[u8]) -> Result<RideActivity> {
    let records = from_bytes(bytes).context("fitparser failed to decode FIT bytes")?;
    let points = extract_record_points(&records)?;
    if points.is_empty() {
        bail!("FIT file contained no record telemetry points");
    }

    let stats = extract_activity_stats(&records, &points)?;
    let summary = RideSummary::from_stats(&stats);
    let laps = extract_laps(&records);
    let rider_weight_kg = extract_rider_weight_kg(&records);
    Ok(RideActivity { stats, summary, laps, points, rider_weight_kg })
}

fn extract_record_points(records: &[FitDataRecord]) -> Result<Vec<TelemetryPoint>> {
    let record_messages: Vec<_> = records.iter().filter(|record| record.kind() == MesgNum::Record).collect();
    if record_messages.is_empty() {
        return Ok(Vec::new());
    }

    let start_timestamp = record_messages
        .iter()
        .find_map(|record| field_timestamp(record, "timestamp"))
        .context("record messages missing timestamp field")?;

    let mut points = Vec::with_capacity(record_messages.len());
    for record in record_messages {
        let timestamp = field_timestamp(record, "timestamp").context("record message missing timestamp")?;
        let timestamp_seconds = (timestamp - start_timestamp).max(0.0) as f32;

        points.push(TelemetryPoint {
            timestamp_seconds,
            location: record_location(record),
            altitude_meters: field_f32_any(record, &["altitude", "enhanced_altitude"]),
            speed_mps: field_f32_any(record, &["speed", "enhanced_speed"]),
            heart_rate_bpm: field_u8(record, "heart_rate"),
            cadence_rpm: field_u8(record, "cadence"),
            power_watts: field_u16(record, "power"),
            respiration_rate_brpm: field_f32_any(record, &["enhanced_respiration_rate", "respiration_rate"]),
            right_pco_mm: field_f32(record, "right_pco"),
            left_pco_mm: field_f32(record, "left_pco"),
        });
    }

    Ok(points)
}

fn extract_activity_stats(records: &[FitDataRecord], points: &[TelemetryPoint]) -> Result<ActivityStats> {
    let (min_elevation, max_elevation) = compute_elevation_range(points);
    let max_speed_from_points = compute_max_speed(points);

    let mut stats = if let Some(session) = records.iter().find(|record| record.kind() == MesgNum::Session) {
        let elapsed_time_seconds = field_f32_any(session, &["total_elapsed_time", "total_timer_time"])
            .unwrap_or_else(|| points.last().map(|point| point.timestamp_seconds).unwrap_or(0.0));
        let timer_time_seconds = field_f32(session, "total_timer_time").unwrap_or(elapsed_time_seconds);
        let total_distance_meters = field_f32(session, "total_distance").unwrap_or_else(|| compute_distance(points));
        let avg_speed_mps = field_f32_any(session, &["enhanced_avg_speed", "avg_speed"])
            .filter(|speed| *speed > 0.0)
            .or_else(|| {
                if elapsed_time_seconds > 0.0 {
                    Some(total_distance_meters / elapsed_time_seconds)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| compute_avg_speed(points));

        ActivityStats {
            sport: field_string(session, "sport"),
            sub_sport: field_string(session, "sub_sport"),
            elapsed_time_seconds,
            timer_time_seconds,
            total_distance_meters,
            avg_speed_mps,
            max_speed_mps: field_f32_any(session, &["enhanced_max_speed", "max_speed"]).or(max_speed_from_points),
            total_ascent_meters: field_f32(session, "total_ascent").unwrap_or_else(|| compute_elevation_gain(points)),
            total_descent_meters: field_f32(session, "total_descent"),
            min_elevation_meters: min_elevation,
            max_elevation_meters: max_elevation,
            avg_heart_rate: field_f32(session, "avg_heart_rate").or_else(|| compute_avg_heart_rate(points)),
            max_heart_rate: field_f32(session, "max_heart_rate"),
            avg_power: field_f32(session, "avg_power").or_else(|| compute_avg_power(points)),
            max_power: field_f32(session, "max_power"),
            normalized_power: field_f32(session, "normalized_power"),
            intensity_factor: field_f32(session, "intensity_factor"),
            training_stress_score: field_f32(session, "training_stress_score"),
            threshold_power: field_f32(session, "threshold_power"),
            total_work_joules: field_f32(session, "total_work"),
            left_right_balance: field_u16(session, "left_right_balance"),
            avg_cadence: field_f32(session, "avg_cadence"),
            max_cadence: field_f32(session, "max_cadence"),
            total_strokes: field_u32(session, "total_strokes"),
            total_calories: field_f32(session, "total_calories"),
            avg_temperature_c: field_f32(session, "avg_temperature"),
            min_temperature_c: field_f32(session, "min_temperature"),
            max_temperature_c: field_f32(session, "max_temperature"),
            avg_respiration: field_f32(session, "enhanced_avg_respiration_rate"),
            min_respiration: field_f32(session, "enhanced_min_respiration_rate"),
            max_respiration: field_f32(session, "enhanced_max_respiration_rate"),
            aerobic_training_effect: field_f32(session, "total_training_effect"),
            anaerobic_training_effect: field_f32(session, "total_anaerobic_training_effect"),
            training_load_peak: field_f32(session, "training_load_peak"),
            time_standing_seconds: field_f32(session, "time_standing"),
            stand_count: field_u32(session, "stand_count"),
            avg_seated_power: field_u16_pair(session, "avg_power_position").map(|(seated, _)| seated),
            avg_standing_power: field_u16_pair(session, "avg_power_position").map(|(_, standing)| standing),
            max_seated_power: field_u16_pair(session, "max_power_position").map(|(seated, _)| seated),
            max_standing_power: field_u16_pair(session, "max_power_position").map(|(_, standing)| standing),
            num_laps: field_u16(session, "num_laps"),
        }
    } else {
        ActivityStats {
            elapsed_time_seconds: points.last().map(|point| point.timestamp_seconds).unwrap_or(0.0),
            timer_time_seconds: points.last().map(|point| point.timestamp_seconds).unwrap_or(0.0),
            total_distance_meters: compute_distance(points),
            avg_speed_mps: compute_avg_speed(points),
            max_speed_mps: max_speed_from_points,
            total_ascent_meters: compute_elevation_gain(points),
            min_elevation_meters: min_elevation,
            max_elevation_meters: max_elevation,
            avg_heart_rate: compute_avg_heart_rate(points),
            avg_power: compute_avg_power(points),
            ..ActivityStats::default()
        }
    };

    if stats.max_speed_mps.is_none() {
        stats.max_speed_mps = max_speed_from_points;
    }
    if stats.min_elevation_meters.is_none() {
        stats.min_elevation_meters = min_elevation;
    }
    if stats.max_elevation_meters.is_none() {
        stats.max_elevation_meters = max_elevation;
    }

    Ok(stats)
}

fn extract_rider_weight_kg(records: &[FitDataRecord]) -> Option<f32> {
    records
        .iter()
        .find(|record| record.kind() == MesgNum::UserProfile)
        .and_then(|profile| field_f32(profile, "weight"))
        .filter(|weight| *weight > 0.0)
}

fn extract_laps(records: &[FitDataRecord]) -> Vec<LapSummary> {
    records
        .iter()
        .filter(|record| record.kind() == MesgNum::Lap)
        .enumerate()
        .map(|(index, lap)| LapSummary {
            index: index as u16,
            elapsed_time_seconds: field_f32_any(lap, &["total_elapsed_time", "total_timer_time"]).unwrap_or(0.0),
            timer_time_seconds: field_f32(lap, "total_timer_time").unwrap_or(0.0),
            total_distance_meters: field_f32(lap, "total_distance").unwrap_or(0.0),
            avg_speed_mps: field_f32_any(lap, &["enhanced_avg_speed", "avg_speed"]).unwrap_or(0.0),
            max_speed_mps: field_f32_any(lap, &["enhanced_max_speed", "max_speed"]),
            avg_heart_rate: field_f32(lap, "avg_heart_rate"),
            max_heart_rate: field_f32(lap, "max_heart_rate"),
            avg_power: field_f32(lap, "avg_power"),
            max_power: field_f32(lap, "max_power"),
            normalized_power: field_f32(lap, "normalized_power"),
            total_ascent_meters: field_f32(lap, "total_ascent").unwrap_or(0.0),
            total_descent_meters: field_f32(lap, "total_descent"),
            total_calories: field_f32(lap, "total_calories"),
            total_work_joules: field_f32(lap, "total_work"),
            avg_cadence: field_f32(lap, "avg_cadence"),
            lap_trigger: field_string(lap, "lap_trigger"),
        })
        .collect()
}

fn record_location(record: &FitDataRecord) -> Option<gpui::Point<f32>> {
    let latitude = field_f64(record, "position_lat").map(|value| (value * SEMICIRCLE_TO_DEGREES) as f32);
    let longitude = field_f64(record, "position_long").map(|value| (value * SEMICIRCLE_TO_DEGREES) as f32);
    match (latitude, longitude) {
        (Some(x), Some(y)) => Some(gpui::point(x, y)),
        _ => None,
    }
}

fn field_by_name<'a>(record: &'a FitDataRecord, name: &str) -> Option<&'a Value> {
    record.fields().iter().find(|field| field.name() == name).map(|field| field.value())
}

fn field_f32(record: &FitDataRecord, name: &str) -> Option<f32> {
    field_by_name(record, name).and_then(value_to_f32)
}

fn field_f32_any(record: &FitDataRecord, names: &[&str]) -> Option<f32> {
    names.iter().find_map(|name| field_f32(record, name))
}

fn field_f64(record: &FitDataRecord, name: &str) -> Option<f64> {
    field_by_name(record, name).and_then(|value| value.clone().try_into().ok())
}

fn field_u8(record: &FitDataRecord, name: &str) -> Option<u8> {
    field_by_name(record, name).and_then(value_to_u8)
}

fn field_u16(record: &FitDataRecord, name: &str) -> Option<u16> {
    field_by_name(record, name).and_then(value_to_u16)
}

fn field_u32(record: &FitDataRecord, name: &str) -> Option<u32> {
    field_by_name(record, name).and_then(value_to_u32)
}

fn field_string(record: &FitDataRecord, name: &str) -> Option<String> {
    field_by_name(record, name).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Enum(value) => Some(value.to_string()),
        _ => None,
    })
}

fn field_u16_pair(record: &FitDataRecord, name: &str) -> Option<(f32, f32)> {
    let value = field_by_name(record, name)?;
    match value {
        Value::Array(values) if values.len() >= 2 => {
            let first = value_to_f32(&values[0])?;
            let second = value_to_f32(&values[1])?;
            Some((first, second))
        }
        _ => None,
    }
}

fn field_timestamp(record: &FitDataRecord, name: &str) -> Option<f64> {
    field_by_name(record, name).and_then(|value| match value {
        Value::Timestamp(timestamp) => Some(timestamp.timestamp() as f64),
        _ => value.clone().try_into().ok(),
    })
}

fn value_to_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Float32(value) => Some(*value),
        Value::Float64(value) => Some(*value as f32),
        _ => value.clone().try_into().ok().map(|value: f64| value as f32),
    }
}

fn value_to_u8(value: &Value) -> Option<u8> {
    match value {
        Value::UInt8(value) | Value::UInt8z(value) | Value::Byte(value) | Value::Enum(value) => Some(*value),
        _ => value.clone().try_into().ok().map(|value: i64| value as u8),
    }
}

fn value_to_u16(value: &Value) -> Option<u16> {
    match value {
        Value::UInt16(value) | Value::UInt16z(value) => Some(*value),
        Value::SInt64(value) => u16::try_from(*value).ok(),
        _ => value.clone().try_into().ok().map(|value: i64| value as u16),
    }
}

fn value_to_u32(value: &Value) -> Option<u32> {
    match value {
        Value::UInt32(value) | Value::UInt32z(value) => Some(*value),
        Value::UInt16(value) => Some(u32::from(*value)),
        _ => value.clone().try_into().ok().map(|value: i64| value as u32),
    }
}

fn compute_distance(points: &[TelemetryPoint]) -> f32 {
    let mut total = 0.0_f32;
    for window in points.windows(2) {
        let (previous, current) = (&window[0], &window[1]);
        if let (Some(from), Some(to)) = (previous.location, current.location) {
            total += haversine_meters(from, to);
        }
    }
    total
}

fn compute_elevation_gain(points: &[TelemetryPoint]) -> f32 {
    let mut gain = 0.0_f32;
    let mut previous: Option<f32> = None;
    for point in points {
        if let Some(altitude) = point.altitude_meters {
            if let Some(last) = previous
                && altitude > last
            {
                gain += altitude - last;
            }
            previous = Some(altitude);
        }
    }
    gain
}

fn compute_elevation_range(points: &[TelemetryPoint]) -> (Option<f32>, Option<f32>) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for point in points {
        if let Some(altitude) = point.altitude_meters {
            min = min.min(altitude);
            max = max.max(altitude);
        }
    }
    if min.is_finite() && max.is_finite() {
        (Some(min), Some(max))
    } else {
        (None, None)
    }
}

fn compute_max_speed(points: &[TelemetryPoint]) -> Option<f32> {
    points.iter().filter_map(|point| point.speed_mps).max_by(f32::total_cmp)
}

fn compute_avg_speed(points: &[TelemetryPoint]) -> f32 {
    let speeds: Vec<f32> = points.iter().filter_map(|point| point.speed_mps).collect();
    if speeds.is_empty() {
        return 0.0;
    }
    speeds.iter().sum::<f32>() / speeds.len() as f32
}

fn compute_avg_heart_rate(points: &[TelemetryPoint]) -> Option<f32> {
    let samples: Vec<f32> = points.iter().filter_map(|point| point.heart_rate_bpm.map(f32::from)).collect();
    if samples.is_empty() {
        return None;
    }
    Some(samples.iter().sum::<f32>() / samples.len() as f32)
}

fn compute_avg_power(points: &[TelemetryPoint]) -> Option<f32> {
    let samples: Vec<f32> = points.iter().filter_map(|point| point.power_watts.map(f32::from)).collect();
    if samples.is_empty() {
        return None;
    }
    Some(samples.iter().sum::<f32>() / samples.len() as f32)
}

pub(crate) fn haversine_meters(from: gpui::Point<f32>, to: gpui::Point<f32>) -> f32 {
    const EARTH_RADIUS_METERS: f64 = 6_371_000.0;
    let lat1 = f64::from(from.x).to_radians();
    let lat2 = f64::from(to.x).to_radians();
    let delta_lat = f64::from(to.x - from.x).to_radians();
    let delta_lon = f64::from(to.y - from.y).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    (EARTH_RADIUS_METERS * c) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_short_distance_is_sane() {
        let from = gpui::point(37.7749, -122.4194);
        let to = gpui::point(37.7750, -122.4194);
        let distance = haversine_meters(from, to);
        assert!(distance > 0.0);
        assert!(distance < 20.0);
    }

    #[test]
    fn sample_fit_parses_left_and_right_pco() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        let with_right = ride.points.iter().filter(|point| point.right_pco_mm.is_some()).count();
        let with_left = ride.points.iter().filter(|point| point.left_pco_mm.is_some()).count();
        assert!(with_right > 100, "expected per-record right PCO samples");
        assert!(with_left > 100, "expected per-record left PCO samples");
    }

    #[test]
    fn sample_fit_parses_right_pco() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        let with_pco = ride.points.iter().filter(|point| point.right_pco_mm.is_some()).count();
        assert!(with_pco > 100, "expected per-record right PCO samples");
    }

    #[test]
    fn sample_fit_parses_respiration_and_rider_weight() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        assert!(ride.rider_weight_kg.is_some());
        let with_respiration = ride.points.iter().filter(|point| point.respiration_rate_brpm.is_some()).count();
        assert!(with_respiration > 100, "expected per-record respiration samples");
    }

    #[test]
    fn sample_fit_record_fields_for_new_metrics() {
        use fitparser::from_bytes;
        let records = from_bytes(crate::graph::activity::SAMPLE_FIT_BYTES).expect("decode");
        let mut respiration_samples = 0usize;
        let mut weight_kg = None;
        for record in &records {
            if record.kind() == fitparser::profile::MesgNum::Record {
                if super::field_f32_any(record, &["enhanced_respiration_rate", "respiration_rate"]).is_some() {
                    respiration_samples += 1;
                }
            }
            if record.kind() == fitparser::profile::MesgNum::UserProfile {
                weight_kg = super::field_f32(record, "weight");
            }
        }
        eprintln!("respiration_samples={respiration_samples} weight_kg={weight_kg:?}");
        assert!(respiration_samples > 0, "expected per-record respiration in sample FIT");
        assert!(weight_kg.is_some(), "expected rider weight in sample FIT user profile");
    }

    #[test]
    fn sample_fit_session_stats_populate_activity_stats() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        assert!(ride.stats.total_distance_meters > 1000.0);
        assert!(ride.stats.avg_power.is_some());
        assert!(ride.stats.training_stress_score.is_some());
        assert!(ride.stats.left_right_balance.is_some());
        assert_eq!(ride.summary.total_distance_meters, ride.stats.total_distance_meters);
    }
}
