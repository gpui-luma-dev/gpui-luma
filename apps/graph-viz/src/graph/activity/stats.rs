use serde::{Deserialize, Serialize};

/// Session-level activity statistics parsed from FIT (primarily the Session message).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ActivityStats {
    pub sport: Option<String>,
    pub sub_sport: Option<String>,
    pub elapsed_time_seconds: f32,
    pub timer_time_seconds: f32,
    pub total_distance_meters: f32,
    pub avg_speed_mps: f32,
    pub max_speed_mps: Option<f32>,
    pub total_ascent_meters: f32,
    pub total_descent_meters: Option<f32>,
    pub min_elevation_meters: Option<f32>,
    pub max_elevation_meters: Option<f32>,
    pub avg_heart_rate: Option<f32>,
    pub max_heart_rate: Option<f32>,
    pub avg_power: Option<f32>,
    pub max_power: Option<f32>,
    pub normalized_power: Option<f32>,
    pub intensity_factor: Option<f32>,
    pub training_stress_score: Option<f32>,
    pub threshold_power: Option<f32>,
    pub total_work_joules: Option<f32>,
    pub left_right_balance: Option<u16>,
    pub avg_cadence: Option<f32>,
    pub max_cadence: Option<f32>,
    pub total_strokes: Option<u32>,
    pub total_calories: Option<f32>,
    pub avg_temperature_c: Option<f32>,
    pub min_temperature_c: Option<f32>,
    pub max_temperature_c: Option<f32>,
    pub avg_respiration: Option<f32>,
    pub min_respiration: Option<f32>,
    pub max_respiration: Option<f32>,
    pub aerobic_training_effect: Option<f32>,
    pub anaerobic_training_effect: Option<f32>,
    pub training_load_peak: Option<f32>,
    pub time_standing_seconds: Option<f32>,
    pub stand_count: Option<u32>,
    pub avg_seated_power: Option<f32>,
    pub avg_standing_power: Option<f32>,
    pub max_seated_power: Option<f32>,
    pub max_standing_power: Option<f32>,
    pub num_laps: Option<u16>,
}

impl ActivityStats {
    pub fn left_right_balance_label(&self) -> Option<String> {
        self.left_right_balance.map(format_left_right_balance)
    }
}

pub fn format_left_right_balance(raw: u16) -> String {
    let right_side = raw & 0x8000 != 0;
    let percent = f32::from(raw & 0x7FFF) / 100.0;
    if right_side {
        format!("{percent:.0}% R / {:.0}% L", 100.0 - percent)
    } else {
        format!("{percent:.0}% L / {:.0}% R", 100.0 - percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_left_right_balance_percentages() {
        assert_eq!(format_left_right_balance(0x8000 | 5_186), "52% R / 48% L");
    }
}
