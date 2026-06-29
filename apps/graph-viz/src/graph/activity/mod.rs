pub mod fit;
pub mod lap;
pub mod model;
pub mod stats;

mod distance;

use std::path::Path;

use anyhow::{Context, Result};

pub use distance::{
    cumulative_distance_profile, distance_axis_available, remap_time_samples_to_x, ride_distance_display_extent,
    scrub_x_fraction, total_distance_meters,
};
pub use fit::parse_fit_bytes;
pub use lap::LapSummary;
pub use model::{RideActivity, RideSummary, TelemetryPoint};
pub use stats::ActivityStats;

pub const SAMPLE_FIT_PATH: &str = "assets/garmin-data/23386792539_ACTIVITY.fit";

pub(crate) const SAMPLE_FIT_BYTES: &[u8] = include_bytes!("../../../assets/garmin-data/23386792539_ACTIVITY.fit");

/// Load the bundled sample ride from embedded FIT bytes.
pub fn load_sample_ride() -> Result<RideActivity> {
    parse_fit_bytes(SAMPLE_FIT_BYTES).context("failed to parse bundled sample FIT ride")
}

/// Load a ride from FIT bytes on disk relative to the graph-viz crate root.
pub fn load_fit_from_crate_path(relative_path: &str) -> Result<RideActivity> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let bytes = std::fs::read(&path).with_context(|| format!("failed to read FIT file at {}", path.display()))?;
    parse_fit_bytes(&bytes).with_context(|| format!("failed to parse FIT file at {}", path.display()))
}

/// Load a ride from any FIT byte slice.
pub fn load_fit_from_bytes(bytes: &[u8]) -> Result<RideActivity> {
    parse_fit_bytes(bytes).context("failed to parse FIT bytes")
}

/// Load a ride from any reader containing FIT data.
pub fn load_fit_from_reader(mut reader: impl std::io::Read) -> Result<RideActivity> {
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut bytes).context("failed to read FIT data")?;
    parse_fit_bytes(&bytes).context("failed to parse FIT data")
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parse_sample_fit_ride() {
        let ride = load_sample_ride().expect("sample ride should parse");
        assert!(ride.points.len() > 100, "expected many telemetry points, got {}", ride.points.len());
        assert!(ride.summary.total_distance_meters > 1000.0);
        assert!(ride.summary.total_duration_seconds > 60.0);
        assert!(ride.summary.elevation_gain_meters > 0.0);
        assert!(ride.summary.avg_speed_mps > 0.0);

        let with_hr = ride.points.iter().filter(|point| point.heart_rate_bpm.is_some()).count();
        let with_power = ride.points.iter().filter(|point| point.power_watts.is_some()).count();
        let with_location = ride.points.iter().filter(|point| point.location.is_some()).count();

        assert!(with_hr > 0, "expected heart rate samples");
        assert!(with_power > 0, "expected power samples");
        assert!(with_location > 0, "expected GPS samples");

        eprintln!(
            "ride summary: distance={:.1}m duration={:.0}s gain={:.0}m avg_speed={:.2}m/s avg_hr={:?} avg_power={:?} points={}",
            ride.summary.total_distance_meters,
            ride.summary.total_duration_seconds,
            ride.summary.elevation_gain_meters,
            ride.summary.avg_speed_mps,
            ride.summary.avg_heart_rate,
            ride.summary.avg_power,
            ride.points.len(),
        );
    }

    #[test]
    fn parse_sample_fit_from_crate_path() {
        let ride = load_fit_from_crate_path(SAMPLE_FIT_PATH).expect("sample ride path should parse");
        assert!(!ride.points.is_empty());
    }

    #[test]
    fn telemetry_timestamps_are_monotonic_from_ride_start() {
        let ride = load_sample_ride().expect("sample ride should parse");
        let mut last = -1.0_f32;
        for point in &ride.points {
            assert!(point.timestamp_seconds >= last, "timestamps should be non-decreasing");
            last = point.timestamp_seconds;
        }
        assert!(ride.points.last().unwrap().timestamp_seconds > 0.0);
    }

    #[test]
    fn load_fit_from_bytes_matches_sample_ride() {
        let ride = load_fit_from_bytes(SAMPLE_FIT_BYTES).expect("bytes should parse");
        assert!(!ride.points.is_empty());
    }

    #[test]
    fn cursor_reader_parses_sample_ride() {
        let ride = load_fit_from_reader(Cursor::new(SAMPLE_FIT_BYTES)).expect("cursor reader should parse");
        assert!(!ride.points.is_empty());
    }
}
