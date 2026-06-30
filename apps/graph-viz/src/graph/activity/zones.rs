use fitparser::{FitDataRecord, Value};
use fitparser::profile::MesgNum;

const HR_ZONE_NAMES: [&str; 5] = ["Warm Up", "Easy", "Aerobic", "Threshold", "Maximum"];
const POWER_ZONE_NAMES: [&str; 7] =
    ["Active Recovery", "Endurance", "Tempo", "Threshold", "VO2 Max", "Anaerobic", "Neuromuscular"];

fn hr_zone_color(index: usize) -> gpui::Hsla {
    match index {
        0 => gpui::hsla(0.0, 0.0, 0.62, 1.0),
        1 => gpui::hsla(0.58, 0.74, 0.52, 1.0),
        2 => gpui::hsla(0.33, 0.66, 0.46, 1.0),
        3 => gpui::hsla(0.08, 0.82, 0.52, 1.0),
        _ => gpui::hsla(0.0, 0.72, 0.52, 1.0),
    }
}

fn power_zone_color(index: usize) -> gpui::Hsla {
    match index {
        0 => gpui::hsla(0.0, 0.0, 0.62, 1.0),
        1 => gpui::hsla(0.58, 0.74, 0.52, 1.0),
        2 => gpui::hsla(0.33, 0.66, 0.46, 1.0),
        3 => gpui::hsla(0.12, 0.84, 0.52, 1.0),
        4 => gpui::hsla(0.08, 0.82, 0.52, 1.0),
        5 => gpui::hsla(0.0, 0.72, 0.52, 1.0),
        _ => gpui::hsla(0.78, 0.62, 0.52, 1.0),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntry {
    pub zone: u8,
    pub label: String,
    pub range_label: String,
    pub time_seconds: f32,
    /// Fraction of total time across all zones in this section (0.0–1.0).
    pub fill_fraction: f32,
    pub color: gpui::Hsla,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZoneTimeProfile {
    pub heart_rate_zones: Vec<ZoneEntry>,
    pub power_zones: Vec<ZoneEntry>,
}

pub fn extract_zone_time_profile(records: &[FitDataRecord]) -> Option<ZoneTimeProfile> {
    let zone_record = find_activity_time_in_zone(records)?;
    let hr_times = field_f64_array(zone_record, "time_in_hr_zone")?;
    let power_times = field_f64_array(zone_record, "time_in_power_zone")?;
    let hr_boundaries = field_u8_array(zone_record, "hr_zone_high_boundary");
    let ftp =
        field_u16(zone_record, "functional_threshold_power").or_else(|| field_u16(zone_record, "threshold_power"));

    let hr_zones = build_hr_zones(&hr_times, hr_boundaries.as_deref());
    let power_zones = build_power_zones(&power_times, ftp);

    if hr_zones.is_empty() && power_zones.is_empty() {
        return None;
    }

    Some(ZoneTimeProfile { heart_rate_zones: hr_zones, power_zones })
}

fn find_activity_time_in_zone(records: &[FitDataRecord]) -> Option<&FitDataRecord> {
    records
        .iter()
        .filter(|record| record.kind() == MesgNum::TimeInZone)
        .find(|record| field_string(record, "reference_mesg").as_deref() == Some("session"))
        .or_else(|| {
            records.iter().filter(|record| record.kind() == MesgNum::TimeInZone).find(|record| {
                field_string(record, "reference_mesg").as_deref() == Some("split")
                    && field_i64(record, "reference_index") == Some(0)
            })
        })
        .or_else(|| {
            records
                .iter()
                .filter(|record| record.kind() == MesgNum::TimeInZone)
                .find(|record| field_string(record, "reference_mesg").as_deref() == Some("activity"))
        })
}

fn build_hr_zones(times: &[f64], boundaries: Option<&[u8]>) -> Vec<ZoneEntry> {
    if times.len() < 6 {
        return Vec::new();
    }

    let zone_times: Vec<f32> = times.iter().skip(1).take(5).map(|value| *value as f32).collect();
    let total_time: f32 = zone_times.iter().sum();
    if total_time <= 0.0 {
        return Vec::new();
    }

    (1..=5_u8)
        .map(|zone| {
            let index = usize::from(zone - 1);
            let time_seconds = zone_times[index];
            let range_label = hr_range_label(zone, boundaries);
            ZoneEntry {
                zone,
                label: HR_ZONE_NAMES[index].to_string(),
                range_label,
                time_seconds,
                fill_fraction: (time_seconds / total_time).clamp(0.0, 1.0),
                color: hr_zone_color(index),
            }
        })
        .collect()
}

fn hr_range_label(zone: u8, boundaries: Option<&[u8]>) -> String {
    let Some(boundaries) = boundaries.filter(|values| values.len() >= 6) else {
        return format!("Zone {zone}");
    };

    let zone = usize::from(zone);
    match zone {
        1 => format!("{} - {} bpm", boundaries[0], boundaries[1].saturating_sub(1)),
        2..=4 => format!("{} - {} bpm", boundaries[zone - 1], boundaries[zone].saturating_sub(1)),
        5 => format!("> {} bpm", boundaries[4]),
        _ => format!("Zone {zone}"),
    }
}

fn build_power_zones(times: &[f64], ftp: Option<u16>) -> Vec<ZoneEntry> {
    if times.len() < 8 {
        return Vec::new();
    }

    let zone_times: Vec<f32> = times.iter().skip(1).take(7).map(|value| *value as f32).collect();
    let total_time: f32 = zone_times.iter().sum();
    if total_time <= 0.0 {
        return Vec::new();
    }

    let ftp = ftp.filter(|value| *value > 0).unwrap_or(200);
    let boundaries = power_zone_boundaries(ftp);

    (1..=7_u8)
        .map(|zone| {
            let index = usize::from(zone - 1);
            let time_seconds = zone_times[index];
            ZoneEntry {
                zone,
                label: POWER_ZONE_NAMES[index].to_string(),
                range_label: power_range_label(zone, ftp, &boundaries),
                time_seconds,
                fill_fraction: (time_seconds / total_time).clamp(0.0, 1.0),
                color: power_zone_color(index),
            }
        })
        .collect()
}

fn power_zone_boundaries(ftp: u16) -> [u16; 7] {
    let ftp = f32::from(ftp);
    [
        (ftp * 0.55).floor() as u16,
        (ftp * 0.75).floor() as u16,
        (ftp * 0.90).floor() as u16,
        (ftp * 1.05).floor() as u16,
        (ftp * 1.20).floor() as u16,
        (ftp * 1.50).floor() as u16,
        (ftp * 1.50).ceil() as u16,
    ]
}

fn power_range_label(zone: u8, _ftp: u16, boundaries: &[u16; 7]) -> String {
    let zone = usize::from(zone);
    match zone {
        1 => format!("0 - {} Watts", boundaries[0]),
        2..=6 => format!("{} - {} Watts", boundaries[zone - 2] + 1, boundaries[zone - 1].saturating_sub(1)),
        7 => format!("> {} Watts", boundaries[5]),
        _ => format!("Zone {zone}"),
    }
}

fn field_by_name<'a>(record: &'a FitDataRecord, name: &str) -> Option<&'a Value> {
    record.fields().iter().find(|field| field.name() == name).map(|field| field.value())
}

fn field_string(record: &FitDataRecord, name: &str) -> Option<String> {
    field_by_name(record, name).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Enum(value) => Some(value.to_string()),
        _ => None,
    })
}

fn field_u16(record: &FitDataRecord, name: &str) -> Option<u16> {
    field_by_name(record, name).and_then(|value| match value {
        Value::UInt16(value) | Value::UInt16z(value) => Some(*value),
        _ => value.clone().try_into().ok().map(|value: i64| value as u16),
    })
}

fn field_i64(record: &FitDataRecord, name: &str) -> Option<i64> {
    field_by_name(record, name).and_then(|value| match value {
        Value::SInt64(value) => Some(*value),
        _ => value.clone().try_into().ok(),
    })
}

fn field_f64_array(record: &FitDataRecord, name: &str) -> Option<Vec<f64>> {
    let value = field_by_name(record, name)?;
    match value {
        Value::Array(values) => Some(values.iter().filter_map(|value| value.clone().try_into().ok()).collect()),
        _ => None,
    }
}

fn field_u8_array(record: &FitDataRecord, name: &str) -> Option<Vec<u8>> {
    let value = field_by_name(record, name)?;
    match value {
        Value::Array(values) => Some(
            values
                .iter()
                .filter_map(|value| match value {
                    Value::UInt8(value) | Value::UInt8z(value) | Value::Byte(value) | Value::Enum(value) => {
                        Some(*value)
                    }
                    _ => value.clone().try_into().ok().map(|value: i64| value as u8),
                })
                .collect(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use fitparser::from_bytes;

    use super::*;
    use crate::graph::activity::SAMPLE_FIT_BYTES;

    #[test]
    fn sample_fit_extracts_zone_time_profile() {
        let records = from_bytes(SAMPLE_FIT_BYTES).expect("decode");
        let profile = extract_zone_time_profile(&records).expect("zone profile");
        assert_eq!(profile.heart_rate_zones.len(), 5);
        assert_eq!(profile.power_zones.len(), 7);

        let dominant_hr = profile
            .heart_rate_zones
            .iter()
            .max_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds))
            .expect("hr zone");
        assert_eq!(dominant_hr.zone, 4);
        assert!((dominant_hr.fill_fraction - 0.66).abs() < 0.02);

        let zone3 = profile.heart_rate_zones.iter().find(|zone| zone.zone == 3).expect("zone 3");
        assert!((zone3.fill_fraction - 0.30).abs() < 0.02);

        assert!(profile.heart_rate_zones.iter().any(|zone| zone.range_label.contains("bpm")));
        assert!(profile.power_zones.iter().any(|zone| zone.range_label.contains("Watts")));
    }

    #[test]
    fn hr_range_labels_use_boundaries() {
        let boundaries = [86, 103, 125, 141, 156, 170];
        assert_eq!(hr_range_label(1, Some(&boundaries)), "86 - 102 bpm");
        assert_eq!(hr_range_label(5, Some(&boundaries)), "> 156 bpm");
    }
}
