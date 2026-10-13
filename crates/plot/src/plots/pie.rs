use crate::data::SeriesId;

/// One series mapped to angular shares. Finite, positive Y values contribute;
/// source order and original sample indices are retained. X is an optional numeric
/// category coordinate, not an angle. Invalid samples and nonpositive weights are omitted.
#[derive(Clone, Debug)]
pub struct PiePlot {
    series: SeriesId,
    start_angle: f64,
    clockwise: bool,
}
impl PiePlot {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { series: series.into(), start_angle: 0.0, clockwise: false }
    }
    /// Degrees from the rightward ray; positive angles point clockwise on screen.
    /// Non-finite inputs restore zero; finite inputs wrap to 0..360.
    pub fn start_angle(mut self, degrees: f64) -> Self {
        self.start_angle = if degrees.is_finite() {
            degrees.rem_euclid(360.0)
        } else {
            0.0
        };
        self
    }
    pub fn clockwise(mut self, clockwise: bool) -> Self {
        self.clockwise = clockwise;
        self
    }
    pub fn series(&self) -> &SeriesId {
        &self.series
    }
    pub fn start_angle_degrees(&self) -> f64 {
        self.start_angle
    }
    pub fn is_clockwise(&self) -> bool {
        self.clockwise
    }
}
