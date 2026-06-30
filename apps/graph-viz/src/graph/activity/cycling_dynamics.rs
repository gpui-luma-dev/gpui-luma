use std::convert::TryInto;

use fitparser::{FitDataRecord, Value};

use super::model::TelemetryPoint;
use super::stats::{ActivityStats, format_left_right_balance};

const POWER_PHASE_LABELS: [&str; 4] = ["Start", "End", "Arc 3", "Arc 4"];
const POWER_PHASE_PEAK_LABELS: [&str; 4] = ["Peak Start", "Peak End", "Peak 3", "Peak 4"];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CyclingDynamics {
    pub avg_left_pco_mm: Option<f32>,
    pub avg_right_pco_mm: Option<f32>,
    pub avg_left_power_phase_deg: Vec<f32>,
    pub avg_left_power_phase_peak_deg: Vec<f32>,
    pub avg_right_power_phase_deg: Vec<f32>,
    pub avg_right_power_phase_peak_deg: Vec<f32>,
    pub left_pco_sample_count: usize,
    pub right_pco_sample_count: usize,
    pub left_pco_min_mm: Option<f32>,
    pub left_pco_max_mm: Option<f32>,
    pub left_pco_avg_mm: Option<f32>,
    pub right_pco_min_mm: Option<f32>,
    pub right_pco_max_mm: Option<f32>,
    pub right_pco_avg_mm: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CyclingDynamicsField {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CyclingDynamicsSection {
    pub title: &'static str,
    pub fields: Vec<CyclingDynamicsField>,
}

pub fn extract_cycling_dynamics(session: Option<&FitDataRecord>, points: &[TelemetryPoint]) -> CyclingDynamics {
    let mut profile = CyclingDynamics::default();
    summarize_pco(points, &mut profile);

    let Some(session) = session else {
        return profile;
    };

    profile.avg_left_pco_mm = field_f32(session, "avg_left_pco");
    profile.avg_right_pco_mm = field_f32(session, "avg_right_pco");
    profile.avg_left_power_phase_deg = field_f32_array(session, "avg_left_power_phase");
    profile.avg_left_power_phase_peak_deg = field_f32_array(session, "avg_left_power_phase_peak");
    profile.avg_right_power_phase_deg = field_f32_array(session, "avg_right_power_phase");
    profile.avg_right_power_phase_peak_deg = field_f32_array(session, "avg_right_power_phase_peak");

    profile
}

pub fn cycling_dynamics_sections(dynamics: &CyclingDynamics, stats: &ActivityStats) -> Vec<CyclingDynamicsSection> {
    let mut sections = Vec::new();

    push_section(
        &mut sections,
        "Balance & Position",
        vec![
            stats.left_right_balance.map(|value| field("Left/Right Balance", format_left_right_balance(value))),
            stats.time_standing_seconds.map(|seconds| field("Standing Time", format_seconds(seconds))),
            stats.stand_count.map(|count| field("Stand Count", count.to_string())),
            stats.avg_seated_power.map(|watts| field("Avg Seated Power", format_watts(watts))),
            stats.avg_standing_power.map(|watts| field("Avg Standing Power", format_watts(watts))),
            stats.max_seated_power.map(|watts| field("Max Seated Power", format_watts(watts))),
            stats.max_standing_power.map(|watts| field("Max Standing Power", format_watts(watts))),
        ],
    );

    push_section(
        &mut sections,
        "Platform Center Offset (Session)",
        vec![
            dynamics.avg_left_pco_mm.map(|value| field("Avg Left PCO", format_mm(value))),
            dynamics.avg_right_pco_mm.map(|value| field("Avg Right PCO", format_mm(value))),
        ],
    );

    push_section(
        &mut sections,
        "Platform Center Offset (Records)",
        vec![
            (dynamics.left_pco_sample_count > 0)
                .then(|| field("Left PCO Samples", dynamics.left_pco_sample_count.to_string())),
            dynamics.left_pco_min_mm.map(|value| field("Left PCO Min", format_mm(value))),
            dynamics.left_pco_max_mm.map(|value| field("Left PCO Max", format_mm(value))),
            dynamics.left_pco_avg_mm.map(|value| field("Left PCO Avg", format_mm(value))),
            (dynamics.right_pco_sample_count > 0)
                .then(|| field("Right PCO Samples", dynamics.right_pco_sample_count.to_string())),
            dynamics.right_pco_min_mm.map(|value| field("Right PCO Min", format_mm(value))),
            dynamics.right_pco_max_mm.map(|value| field("Right PCO Max", format_mm(value))),
            dynamics.right_pco_avg_mm.map(|value| field("Right PCO Avg", format_mm(value))),
        ],
    );

    push_array_section(
        &mut sections,
        "Left Power Phase",
        &dynamics.avg_left_power_phase_deg,
        POWER_PHASE_LABELS,
        "Avg Left Power Phase",
    );
    push_array_section(
        &mut sections,
        "Left Power Phase Peak",
        &dynamics.avg_left_power_phase_peak_deg,
        POWER_PHASE_PEAK_LABELS,
        "Avg Left Power Phase Peak",
    );
    push_array_section(
        &mut sections,
        "Right Power Phase",
        &dynamics.avg_right_power_phase_deg,
        POWER_PHASE_LABELS,
        "Avg Right Power Phase",
    );
    push_array_section(
        &mut sections,
        "Right Power Phase Peak",
        &dynamics.avg_right_power_phase_peak_deg,
        POWER_PHASE_PEAK_LABELS,
        "Avg Right Power Phase Peak",
    );

    sections
}

fn summarize_pco(points: &[TelemetryPoint], profile: &mut CyclingDynamics) {
    let left: Vec<f32> = points.iter().filter_map(|point| point.left_pco_mm).collect();
    let right: Vec<f32> = points.iter().filter_map(|point| point.right_pco_mm).collect();

    profile.left_pco_sample_count = left.len();
    profile.right_pco_sample_count = right.len();
    profile.left_pco_min_mm = min_value(&left);
    profile.left_pco_max_mm = max_value(&left);
    profile.left_pco_avg_mm = avg_value(&left);
    profile.right_pco_min_mm = min_value(&right);
    profile.right_pco_max_mm = max_value(&right);
    profile.right_pco_avg_mm = avg_value(&right);
}

fn push_section(
    sections: &mut Vec<CyclingDynamicsSection>,
    title: &'static str,
    fields: Vec<Option<CyclingDynamicsField>>,
) {
    let fields: Vec<_> = fields.into_iter().flatten().collect();
    if fields.is_empty() {
        return;
    }
    sections.push(CyclingDynamicsSection { title, fields });
}

fn push_array_section(
    sections: &mut Vec<CyclingDynamicsSection>,
    title: &'static str,
    values: &[f32],
    labels: [&str; 4],
    combined_label: &str,
) {
    if values.is_empty() {
        return;
    }

    let mut fields = vec![field(combined_label, format_degree_list(values))];
    for (index, value) in values.iter().enumerate() {
        let label = labels.get(index).copied().unwrap_or("Value");
        fields.push(field(format!("{combined_label} · {label}"), format_degrees(*value)));
    }

    sections.push(CyclingDynamicsSection { title, fields });
}

fn field(label: impl Into<String>, value: impl Into<String>) -> CyclingDynamicsField {
    CyclingDynamicsField { label: label.into(), value: value.into() }
}

fn format_mm(value: f32) -> String {
    format!("{value:.1} mm")
}

fn format_watts(value: f32) -> String {
    format!("{value:.0} W")
}

fn format_degrees(value: f32) -> String {
    format!("{value:.1}°")
}

fn format_degree_list(values: &[f32]) -> String {
    values.iter().map(|value| format!("{value:.0}°")).collect::<Vec<_>>().join(", ")
}

fn format_seconds(seconds: f32) -> String {
    let total = seconds.round().max(0.0) as u32;
    let minutes = total / 60;
    let secs = total % 60;
    format!("{minutes}:{secs:02}")
}

fn min_value(values: &[f32]) -> Option<f32> {
    values.iter().copied().min_by(f32::total_cmp)
}

fn max_value(values: &[f32]) -> Option<f32> {
    values.iter().copied().max_by(f32::total_cmp)
}

fn avg_value(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f32>() / values.len() as f32)
}

fn field_f32(record: &FitDataRecord, name: &str) -> Option<f32> {
    field_by_name(record, name).and_then(|value| match value {
        Value::Float32(value) => Some(*value),
        Value::Float64(value) => Some(*value as f32),
        Value::SInt8(value) => Some(f32::from(*value)),
        Value::UInt8(value) | Value::UInt8z(value) | Value::Byte(value) => Some(f32::from(*value)),
        Value::SInt16(value) => Some(f32::from(*value)),
        Value::UInt16(value) | Value::UInt16z(value) => Some(f32::from(*value)),
        Value::SInt32(value) => Some(*value as f32),
        Value::UInt32(value) | Value::UInt32z(value) => Some(*value as f32),
        _ => value.clone().try_into().ok().map(|value: f64| value as f32),
    })
}

fn field_f32_array(record: &FitDataRecord, name: &str) -> Vec<f32> {
    let Some(value) = field_by_name(record, name) else {
        return Vec::new();
    };

    match value {
        Value::Array(values) => values.iter().filter_map(|value| field_f32_from_value(value)).collect(),
        _ => field_f32_from_value(value).into_iter().collect(),
    }
}

fn field_f32_from_value(value: &Value) -> Option<f32> {
    match value {
        Value::Float32(value) => Some(*value),
        Value::Float64(value) => Some(*value as f32),
        _ => value.clone().try_into().ok().map(|value: f64| value as f32),
    }
}

fn field_by_name<'a>(record: &'a FitDataRecord, name: &str) -> Option<&'a Value> {
    record.fields().iter().find(|field| field.name() == name).map(|field| field.value())
}

#[cfg(test)]
mod tests {
    use fitparser::from_bytes;
    use fitparser::profile::MesgNum;

    use super::*;
    use crate::graph::activity::{SAMPLE_FIT_BYTES, load_sample_ride};

    #[test]
    fn sample_fit_extracts_cycling_dynamics() {
        let ride = load_sample_ride().expect("sample ride");
        let dynamics = ride.cycling_dynamics.expect("cycling dynamics");
        assert!(dynamics.avg_left_power_phase_deg.len() >= 2);
        assert!(dynamics.left_pco_sample_count > 100);
        assert!(dynamics.right_pco_sample_count > 100);

        let sections = cycling_dynamics_sections(&dynamics, &ride.stats);
        assert!(sections.iter().any(|section| section.title.contains("Power Phase")));
        assert!(sections.iter().any(|section| section.fields.iter().any(|field| field.label.contains("PCO"))));
    }

    #[test]
    fn session_power_phase_fields_exist_in_sample_fit() {
        let records = from_bytes(SAMPLE_FIT_BYTES).expect("decode");
        let session = records.iter().find(|record| record.kind() == MesgNum::Session).expect("session");
        let dynamics = extract_cycling_dynamics(Some(session), &[]);
        assert_eq!(dynamics.avg_left_power_phase_deg.len(), 4);
        assert_eq!(dynamics.avg_right_power_phase_peak_deg.len(), 4);
    }
}
