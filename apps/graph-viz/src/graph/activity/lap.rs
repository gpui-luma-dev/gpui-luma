use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LapSummary {
    pub index: u16,
    pub elapsed_time_seconds: f32,
    pub timer_time_seconds: f32,
    pub total_distance_meters: f32,
    pub avg_speed_mps: f32,
    pub max_speed_mps: Option<f32>,
    pub avg_heart_rate: Option<f32>,
    pub max_heart_rate: Option<f32>,
    pub avg_power: Option<f32>,
    pub max_power: Option<f32>,
    pub normalized_power: Option<f32>,
    pub total_ascent_meters: f32,
    pub total_descent_meters: Option<f32>,
    pub total_calories: Option<f32>,
    pub total_work_joules: Option<f32>,
    pub avg_cadence: Option<f32>,
    pub lap_trigger: Option<String>,
}
