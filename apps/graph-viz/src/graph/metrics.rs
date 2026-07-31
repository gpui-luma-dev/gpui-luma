use super::activity::TelemetryPoint;
use super::units::{MetricDisplay, PowerUnit, SpeedUnit};

const RIGHT_PCO_AXIS_MIN_MM: f32 = -30.0;
const RIGHT_PCO_AXIS_MAX_MM: f32 = 30.0;
const RIGHT_PCO_AXIS_STEP_MM: f32 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TelemetryMetric {
    Elevation,
    HeartRate,
    RespirationRate,
    RightPlatformCenterOffset,
    Power,
    Speed,
}

impl TelemetryMetric {
    pub const ALL: [Self; 6] = [
        Self::Elevation,
        Self::HeartRate,
        Self::RespirationRate,
        Self::RightPlatformCenterOffset,
        Self::Power,
        Self::Speed,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Elevation => "elevation",
            Self::HeartRate => "heart_rate",
            Self::RespirationRate => "respiration_rate",
            Self::RightPlatformCenterOffset => "right_pco",
            Self::Power => "power",
            Self::Speed => "speed",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Elevation => "Elevation",
            Self::HeartRate => "Heart Rate",
            Self::RespirationRate => "Respiration Rate",
            Self::RightPlatformCenterOffset => "Right Platform Center Offset",
            Self::Power => "Power",
            Self::Speed => "Speed",
        }
    }

    pub fn invert_y_axis(self) -> bool {
        false
    }

    pub fn emphasize_zero_baseline(self) -> bool {
        matches!(self, Self::RightPlatformCenterOffset)
    }

    pub fn fixed_y_display_range(self) -> Option<std::ops::RangeInclusive<f32>> {
        match self {
            Self::RightPlatformCenterOffset => Some(RIGHT_PCO_AXIS_MIN_MM..=RIGHT_PCO_AXIS_MAX_MM),
            _ => None,
        }
    }

    pub fn is_available(self, points: &[TelemetryPoint]) -> bool {
        self.samples(points).len() >= 2
    }

    pub fn available(points: &[TelemetryPoint]) -> Vec<Self> {
        Self::ALL.into_iter().filter(|metric| metric.is_available(points)).collect()
    }

    pub fn format_display_tick(self, display_value: f32, step: f32) -> String {
        let decimals = if step >= 1.0 - f32::EPSILON {
            0
        } else if step >= 0.1 - f32::EPSILON {
            1
        } else {
            2
        };
        format!("{display_value:.decimals$}")
    }

    pub fn to_display(self, value: f32, units: MetricDisplay) -> f32 {
        match self {
            Self::Speed => units.speed_unit.mps_to_display(value),
            Self::Power => power_to_display(value, units),
            _ => value,
        }
    }

    pub fn display_to_internal(self, display: f32, units: MetricDisplay) -> f32 {
        match self {
            Self::Speed => units.speed_unit.to_mps(display),
            Self::Power => power_from_display(display, units),
            _ => display,
        }
    }

    pub fn include_zero_on_axis(self) -> bool {
        matches!(self, Self::Speed | Self::Power)
    }

    pub fn preferred_y_display_step(self, units: MetricDisplay) -> Option<f32> {
        match self {
            Self::Speed if units.speed_unit == SpeedUnit::Mph => Some(5.0),
            Self::Power if units.power_unit == PowerUnit::Watts => Some(50.0),
            Self::Power if units.power_unit == PowerUnit::WattsPerKg => Some(0.5),
            Self::RightPlatformCenterOffset => Some(RIGHT_PCO_AXIS_STEP_MM),
            _ => None,
        }
    }

    pub fn samples(self, points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
        match self {
            Self::Elevation => points
                .iter()
                .filter_map(|point| point.altitude_meters.map(|value| (point.timestamp_seconds, value)))
                .collect(),
            Self::HeartRate => points
                .iter()
                .filter_map(|point| point.heart_rate_bpm.map(|value| (point.timestamp_seconds, f32::from(value))))
                .collect(),
            Self::RespirationRate => points
                .iter()
                .filter_map(|point| point.respiration_rate_brpm.map(|value| (point.timestamp_seconds, value)))
                .collect(),
            Self::RightPlatformCenterOffset => points
                .iter()
                .filter_map(|point| point.right_pco_mm.map(|value| (point.timestamp_seconds, value)))
                .collect(),
            Self::Power => points
                .iter()
                .filter_map(|point| point.power_watts.map(|value| (point.timestamp_seconds, f32::from(value))))
                .collect(),
            Self::Speed => points
                .iter()
                .filter_map(|point| point.speed_mps.map(|value| (point.timestamp_seconds, value)))
                .collect(),
        }
    }
}

pub fn left_pco_samples(points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    points
        .iter()
        .filter_map(|point| point.left_pco_mm.map(|value| (point.timestamp_seconds, value)))
        .collect()
}

/// Left PCO plotted above the zero line (positive Y).
pub fn pco_left_plot_samples(points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    points
        .iter()
        .filter_map(|point| point.left_pco_mm.map(|value| (point.timestamp_seconds, value.abs())))
        .collect()
}

/// Right PCO plotted below the zero line (negative Y).
pub fn pco_right_plot_samples(points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    points
        .iter()
        .filter_map(|point| point.right_pco_mm.map(|value| (point.timestamp_seconds, -value.abs())))
        .collect()
}

fn power_to_display(watts: f32, units: MetricDisplay) -> f32 {
    match units.power_unit {
        PowerUnit::Watts => watts,
        PowerUnit::WattsPerKg => units.rider_weight_kg.map(|kg| watts / kg).unwrap_or(watts),
    }
}

fn power_from_display(display: f32, units: MetricDisplay) -> f32 {
    match units.power_unit {
        PowerUnit::Watts => display,
        PowerUnit::WattsPerKg => units.rider_weight_kg.map(|kg| display * kg).unwrap_or(display),
    }
}

pub fn value_at_time(samples: &[(f32, f32)], time: f32) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }
    if time <= samples[0].0 {
        return Some(samples[0].1);
    }
    if time >= samples[samples.len() - 1].0 {
        return Some(samples[samples.len() - 1].1);
    }

    for window in samples.windows(2) {
        let (left_time, left_value) = window[0];
        let (right_time, right_value) = window[1];
        if time >= left_time && time <= right_time {
            let span = (right_time - left_time).max(f32::EPSILON);
            let t = (time - left_time) / span;
            return Some(left_value + (right_value - left_value) * t);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::units::XAxisMode;

    fn sample_point(timestamp_seconds: f32, altitude: Option<f32>, heart_rate: Option<u8>) -> TelemetryPoint {
        TelemetryPoint {
            timestamp_seconds,
            location: None,
            altitude_meters: altitude,
            speed_mps: None,
            heart_rate_bpm: heart_rate,
            cadence_rpm: None,
            power_watts: None,
            respiration_rate_brpm: None,
            right_pco_mm: None,
            left_pco_mm: None,
        }
    }

    #[test]
    fn sample_ride_left_pco_pairs_with_right_metric() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        assert!(TelemetryMetric::RightPlatformCenterOffset.is_available(&ride.points));
        assert!(left_pco_samples(&ride.points).len() >= 2);
    }

    #[test]
    fn pco_plot_samples_split_above_and_below_zero() {
        let ride = crate::graph::activity::load_sample_ride().expect("sample ride");
        let left = pco_left_plot_samples(&ride.points);
        let right = pco_right_plot_samples(&ride.points);
        assert!(left.iter().all(|(_, y)| *y >= 0.0));
        assert!(right.iter().all(|(_, y)| *y <= 0.0));
    }

    #[test]
    fn right_pco_uses_fixed_axis_without_invert() {
        let metric = TelemetryMetric::RightPlatformCenterOffset;
        assert!(!metric.invert_y_axis());
        assert_eq!(metric.fixed_y_display_range(), Some(-30.0..=30.0));
    }

    #[test]
    fn available_metrics_require_at_least_two_samples() {
        let sparse = vec![sample_point(0.0, Some(100.0), None), sample_point(10.0, Some(110.0), None)];
        assert_eq!(TelemetryMetric::available(&sparse), vec![TelemetryMetric::Elevation]);

        let dense = vec![sample_point(0.0, Some(100.0), Some(120)), sample_point(10.0, Some(110.0), Some(125))];
        assert!(TelemetryMetric::available(&dense).contains(&TelemetryMetric::Elevation));
        assert!(TelemetryMetric::available(&dense).contains(&TelemetryMetric::HeartRate));
    }

    #[test]
    fn power_display_converts_watts_to_w_per_kg() {
        let units = MetricDisplay {
            speed_unit: SpeedUnit::Mph,
            x_axis: XAxisMode::Time,
            power_unit: PowerUnit::WattsPerKg,
            rider_weight_kg: Some(80.0),
        };
        assert!((TelemetryMetric::Power.to_display(160.0, units) - 2.0).abs() < f32::EPSILON);
        assert!((TelemetryMetric::Power.display_to_internal(2.0, units) - 160.0).abs() < f32::EPSILON);
    }

    #[test]
    fn value_at_time_interpolates_between_samples() {
        let samples = vec![(0.0, 10.0), (10.0, 30.0)];
        assert_eq!(value_at_time(&samples, 5.0), Some(20.0));
    }
}
