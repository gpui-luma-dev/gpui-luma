#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum XAxisMode {
    #[default]
    Time,
    Distance,
}

impl XAxisMode {
    pub fn from_toggle(distance_selected: bool) -> Self {
        if distance_selected { Self::Distance } else { Self::Time }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Time => "Time",
            Self::Distance => "Distance",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpeedUnit {
    #[default]
    Mph,
    Kmh,
}

impl SpeedUnit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mph => "mph",
            Self::Kmh => "km/h",
        }
    }

    pub fn from_toggle(kmh_selected: bool) -> Self {
        if kmh_selected { Self::Kmh } else { Self::Mph }
    }

    pub fn mps_to_display(self, meters_per_second: f32) -> f32 {
        match self {
            Self::Mph => meters_per_second * 2.236_936_3,
            Self::Kmh => meters_per_second * 3.6,
        }
    }

    pub fn to_mps(self, display: f32) -> f32 {
        match self {
            Self::Mph => display / 2.236_936_3,
            Self::Kmh => display / 3.6,
        }
    }

    pub fn format_distance(self, meters: f32) -> String {
        match self {
            Self::Mph => {
                let miles = meters / 1609.344;
                format!("{:.1} mi", miles)
            }
            Self::Kmh => {
                if meters >= 1000.0 {
                    format!("{:.1} km", meters / 1000.0)
                } else {
                    format!("{:.0} m", meters)
                }
            }
        }
    }

    pub fn distance_axis_from_meters(self, meters: f32) -> f32 {
        match self {
            Self::Mph => meters / 1609.344,
            Self::Kmh => meters / 1000.0,
        }
    }

    pub fn format_distance_axis_tick(self, display_value: f32, step: f32) -> String {
        let decimals = if step >= 1.0 - f32::EPSILON {
            0
        } else if step >= 0.1 - f32::EPSILON {
            1
        } else {
            2
        };
        match self {
            Self::Mph => format!("{display_value:.decimals$} mi"),
            Self::Kmh => format!("{display_value:.decimals$} km"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PowerUnit {
    #[default]
    Watts,
    WattsPerKg,
}

impl PowerUnit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Watts => "W",
            Self::WattsPerKg => "W/kg",
        }
    }

    pub fn from_toggle(w_per_kg_selected: bool) -> Self {
        if w_per_kg_selected {
            Self::WattsPerKg
        } else {
            Self::Watts
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MetricDisplay {
    pub speed_unit: SpeedUnit,
    pub x_axis: XAxisMode,
    pub power_unit: PowerUnit,
    pub rider_weight_kg: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_toggle_maps_units() {
        assert_eq!(SpeedUnit::from_toggle(false), SpeedUnit::Mph);
        assert_eq!(SpeedUnit::from_toggle(true), SpeedUnit::Kmh);
    }

    #[test]
    fn x_axis_toggle_maps_modes() {
        assert_eq!(XAxisMode::from_toggle(false), XAxisMode::Time);
        assert_eq!(XAxisMode::from_toggle(true), XAxisMode::Distance);
    }

    #[test]
    fn mph_and_kmh_convert_mps_to_display() {
        let mps = 10.0;
        assert!((SpeedUnit::Mph.mps_to_display(mps) - 22.369).abs() < 0.01);
        assert!((SpeedUnit::Kmh.mps_to_display(mps) - 36.0).abs() < 0.01);
    }

    #[test]
    fn format_distance_respects_unit_system() {
        let meters = 32_091.2;
        assert_eq!(SpeedUnit::Kmh.format_distance(meters), "32.1 km");
        assert_eq!(SpeedUnit::Mph.format_distance(meters), "19.9 mi");
        assert_eq!(SpeedUnit::Kmh.format_distance(850.0), "850 m");
    }

    #[test]
    fn power_unit_toggle_maps_modes() {
        assert_eq!(PowerUnit::from_toggle(false), PowerUnit::Watts);
        assert_eq!(PowerUnit::from_toggle(true), PowerUnit::WattsPerKg);
    }

    #[test]
    fn round_trip_display_units() {
        for unit in [SpeedUnit::Mph, SpeedUnit::Kmh] {
            let mps = 8.5;
            assert!((unit.to_mps(unit.mps_to_display(mps)) - mps).abs() < 0.001);
        }
    }
}
