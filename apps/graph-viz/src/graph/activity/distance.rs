use super::fit::haversine_meters;
use super::model::TelemetryPoint;
use crate::graph::metrics::value_at_time;

/// Cumulative distance in meters at each telemetry timestamp.
pub fn cumulative_distance_profile(points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut total = 0.0_f32;
    let mut profile = Vec::with_capacity(points.len());
    profile.push((points[0].timestamp_seconds, 0.0));

    for window in points.windows(2) {
        let (previous, current) = (&window[0], &window[1]);
        let dt = (current.timestamp_seconds - previous.timestamp_seconds).max(0.0);
        if let (Some(from), Some(to)) = (previous.location, current.location) {
            total += haversine_meters(from, to);
        } else if let Some(speed) = previous.speed_mps.or(current.speed_mps) {
            total += speed * dt;
        }
        profile.push((current.timestamp_seconds, total));
    }

    profile
}

pub fn distance_axis_available(profile: &[(f32, f32)]) -> bool {
    profile.last().is_some_and(|(_, distance_meters)| *distance_meters > f32::EPSILON)
}

pub fn total_distance_meters(profile: &[(f32, f32)]) -> f32 {
    profile.last().map(|(_, distance)| *distance).unwrap_or(0.0)
}

pub fn remap_time_samples_to_x(
    samples: &[(f32, f32)],
    distance_profile: &[(f32, f32)],
    x_axis: crate::graph::units::XAxisMode,
    speed_unit: crate::graph::units::SpeedUnit,
) -> Vec<(f32, f32)> {
    match x_axis {
        crate::graph::units::XAxisMode::Time => samples.to_vec(),
        crate::graph::units::XAxisMode::Distance => samples
            .iter()
            .filter_map(|(time, value)| {
                value_at_time(distance_profile, *time)
                    .map(|meters| (speed_unit.distance_axis_from_meters(meters), *value))
            })
            .collect(),
    }
}

pub fn ride_distance_display_extent(
    profile: &[(f32, f32)],
    duration_seconds: f32,
    speed_unit: crate::graph::units::SpeedUnit,
) -> f32 {
    let meters = value_at_time(profile, duration_seconds).unwrap_or_else(|| total_distance_meters(profile));
    speed_unit.distance_axis_from_meters(meters).max(f32::EPSILON)
}

pub fn scrub_x_fraction(
    time_scrub_fraction: f32,
    duration_seconds: f32,
    distance_profile: &[(f32, f32)],
    x_min: f32,
    x_max: f32,
    x_axis: crate::graph::units::XAxisMode,
    speed_unit: crate::graph::units::SpeedUnit,
) -> f32 {
    let span = (x_max - x_min).max(f32::EPSILON);
    let x_value = match x_axis {
        crate::graph::units::XAxisMode::Time => x_min + time_scrub_fraction.clamp(0.0, 1.0) * span,
        crate::graph::units::XAxisMode::Distance => {
            let scrub_time = duration_seconds * time_scrub_fraction.clamp(0.0, 1.0);
            let meters = value_at_time(distance_profile, scrub_time).unwrap_or(0.0);
            speed_unit.distance_axis_from_meters(meters)
        }
    };
    ((x_value - x_min) / span).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::point;

    fn point_at(timestamp_seconds: f32, location: Option<gpui::Point<f32>>, speed_mps: Option<f32>) -> TelemetryPoint {
        TelemetryPoint {
            timestamp_seconds,
            location,
            altitude_meters: None,
            speed_mps,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_watts: None,
            respiration_rate_brpm: None,
            right_pco_mm: None,
            left_pco_mm: None,
        }
    }

    #[test]
    fn cumulative_distance_uses_gps_segments() {
        let points = vec![
            point_at(0.0, Some(point(37.0, -122.0)), None),
            point_at(10.0, Some(point(37.001, -122.0)), None),
            point_at(20.0, Some(point(37.002, -122.0)), None),
        ];
        let profile = cumulative_distance_profile(&points);
        assert_eq!(profile.len(), 3);
        assert!(total_distance_meters(&profile) > 0.0);
    }

    #[test]
    fn cumulative_distance_falls_back_to_speed_integration() {
        let points =
            vec![point_at(0.0, None, Some(10.0)), point_at(10.0, None, Some(10.0)), point_at(20.0, None, Some(10.0))];
        let profile = cumulative_distance_profile(&points);
        assert!((total_distance_meters(&profile) - 200.0).abs() < 0.01);
    }
}
