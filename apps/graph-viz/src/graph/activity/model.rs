use gpui::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RideSummary {
    pub total_distance_meters: f32,
    pub total_duration_seconds: f32,
    pub elevation_gain_meters: f32,
    pub avg_speed_mps: f32,
    pub avg_heart_rate: Option<f32>,
    pub avg_power: Option<f32>,
}

impl RideSummary {
    pub fn from_stats(stats: &super::stats::ActivityStats) -> Self {
        Self {
            total_distance_meters: stats.total_distance_meters,
            total_duration_seconds: stats.elapsed_time_seconds,
            elevation_gain_meters: stats.total_ascent_meters,
            avg_speed_mps: stats.avg_speed_mps,
            avg_heart_rate: stats.avg_heart_rate,
            avg_power: stats.avg_power,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryPoint {
    /// Offset in seconds from ride start.
    pub timestamp_seconds: f32,
    /// Latitude and longitude in decimal degrees.
    pub location: Option<Point<f32>>,
    pub altitude_meters: Option<f32>,
    pub speed_mps: Option<f32>,
    pub heart_rate_bpm: Option<u8>,
    pub cadence_rpm: Option<u8>,
    pub power_watts: Option<u16>,
    pub respiration_rate_brpm: Option<f32>,
    /// Right platform center offset in millimeters (FIT `right_pco`).
    pub right_pco_mm: Option<f32>,
    /// Left platform center offset in millimeters (FIT `left_pco`).
    pub left_pco_mm: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RideActivity {
    pub stats: super::stats::ActivityStats,
    pub summary: RideSummary,
    pub laps: Vec<super::lap::LapSummary>,
    pub points: Vec<TelemetryPoint>,
    pub rider_weight_kg: Option<f32>,
}
