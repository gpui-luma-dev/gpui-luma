use super::model::TelemetryPoint;

/// Standard Garmin-style power duration buckets (seconds).
pub const POWER_CURVE_DURATIONS_SEC: [f32; 14] =
    [1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1200.0, 1800.0, 3600.0, 7200.0];

pub const POWER_CURVE_DURATION_LABELS: [&str; 14] = [
    "1 sec", "2 sec", "5 sec", "10 sec", "20 sec", "30 sec", "1 min", "2 min", "5 min", "10 min", "20 min", "30 min",
    "1 hr", "2 hr",
];

#[derive(Debug, Clone, PartialEq)]
pub struct PowerCurvePoint {
    pub index: usize,
    pub duration_seconds: f32,
    pub duration_label: &'static str,
    pub max_mean_power_watts: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityPowerCurve {
    pub activity: Vec<PowerCurvePoint>,
    /// Reserved for future multi-file record overlay.
    pub record: Vec<PowerCurvePoint>,
}

pub fn compute_activity_power_curve(points: &[TelemetryPoint]) -> ActivityPowerCurve {
    let samples = power_samples(points);
    let ride_duration =
        samples.last().map(|(t, _)| *t).unwrap_or(0.0) - samples.first().map(|(t, _)| *t).unwrap_or(0.0);

    let activity = POWER_CURVE_DURATIONS_SEC
        .iter()
        .zip(POWER_CURVE_DURATION_LABELS.iter())
        .enumerate()
        .filter_map(|(index, (&duration, &label))| {
            if ride_duration + f32::EPSILON < duration {
                return None;
            }
            max_mean_power(&samples, duration).map(|max_mean_power_watts| PowerCurvePoint {
                index,
                duration_seconds: duration,
                duration_label: label,
                max_mean_power_watts,
            })
        })
        .collect();

    ActivityPowerCurve { activity, record: Vec::new() }
}

fn power_samples(points: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    points
        .iter()
        .filter_map(|point| point.power_watts.map(|power| (point.timestamp_seconds, power as f32)))
        .collect()
}

/// Maximum mean power (MMP) over any window of `duration_seconds`.
fn max_mean_power(samples: &[(f32, f32)], duration_seconds: f32) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }

    if duration_seconds <= 1.0 {
        return samples.iter().map(|(_, power)| *power).max_by(f32::total_cmp);
    }

    let window_samples = duration_seconds.round().max(2.0) as usize;
    if samples.len() < window_samples {
        return None;
    }

    let mut window_sum: f32 = samples[..window_samples].iter().map(|(_, power)| *power).sum();
    let mut best = window_sum / window_samples as f32;

    for start in 1..=samples.len() - window_samples {
        window_sum += samples[start + window_samples - 1].1 - samples[start - 1].1;
        best = best.max(window_sum / window_samples as f32);
    }

    Some(best)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::activity::load_sample_ride;

    #[test]
    fn sample_ride_produces_power_curve_points() {
        let ride = load_sample_ride().expect("sample ride");
        let curve = compute_activity_power_curve(&ride.points);
        assert!(curve.activity.len() >= 10, "expected many duration buckets");
        assert!(curve.activity.len() < POWER_CURVE_DURATIONS_SEC.len(), "ride shorter than 2 hr");

        let one_second = curve.activity.first().expect("1 sec point");
        assert_eq!(one_second.duration_label, "1 sec");
        assert!(one_second.max_mean_power_watts > 500.0);

        let mut last_index = None;
        for point in &curve.activity {
            if let Some(prev) = last_index {
                assert!(point.index > prev);
                assert!(point.max_mean_power_watts <= curve.activity[0].max_mean_power_watts + f32::EPSILON);
            }
            last_index = Some(point.index);
        }
    }

    #[test]
    fn max_mean_power_finds_plateau() {
        let samples: Vec<(f32, f32)> = (0..60)
            .map(|second| (second as f32, if (10..20).contains(&second) { 300.0 } else { 100.0 }))
            .collect();
        let mmp = max_mean_power(&samples, 10.0).expect("10 sec mmp");
        assert!((mmp - 300.0).abs() < 0.01);
    }
}
