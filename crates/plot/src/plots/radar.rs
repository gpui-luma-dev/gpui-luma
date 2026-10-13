use crate::SeriesId;

/// Radar mapping: X is a zero-based spoke index, Y a nonnegative radial value.
/// All series share the same zero-based radial scale and category order.
#[derive(Clone, Debug)]
pub struct RadarPlot {
    pub(crate) series: Vec<SeriesId>,
    pub(crate) maximum: Option<f64>,
    pub(crate) start_angle: f64,
}
impl RadarPlot {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { series: vec![series.into()], maximum: None, start_angle: -90.0 }
    }
    pub fn series(mut self, series: impl Into<SeriesId>) -> Self {
        let series = series.into();
        if !self.series.contains(&series) {
            self.series.push(series);
        }
        self
    }
    /// Explicit positive radial maximum; invalid values restore automatic scaling.
    pub fn maximum(mut self, value: f64) -> Self {
        self.maximum = (value.is_finite() && value > 0.0).then_some(value);
        self
    }
    /// Degrees clockwise from the rightward ray. Default -90 places spoke zero at the top.
    pub fn start_angle(mut self, value: f64) -> Self {
        self.start_angle = if value.is_finite() {
            value.rem_euclid(360.0)
        } else {
            -90.0
        };
        self
    }
}
