use crate::SeriesId;
/// Independent observations occupy concentric tracks; Stacked shares one track.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadialLayout {
    #[default]
    Concentric,
    Stacked,
}
/// Positive Y values map to angular lengths against a shared maximum.
#[derive(Clone, Debug)]
pub struct RadialPlot {
    pub(crate) series: SeriesId,
    pub(crate) maximum: Option<f64>,
    pub(crate) start: f64,
    pub(crate) sweep: f64,
    pub(crate) layout: RadialLayout,
}
impl RadialPlot {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { series: series.into(), maximum: None, start: 0.0, sweep: -360.0, layout: RadialLayout::Concentric }
    }
    /// Invalid maxima restore automatic scaling. Stacked auto scaling uses the total.
    pub fn maximum(mut self, value: f64) -> Self {
        self.maximum = (value.is_finite() && value > 0.0).then_some(value);
        self
    }
    /// Degrees from the rightward ray. Positive sweep grows clockwise; negative counterclockwise.
    pub fn angles(mut self, start: f64, sweep: f64) -> Self {
        self.start = if start.is_finite() {
            start.rem_euclid(360.0)
        } else {
            0.0
        };
        self.sweep = if sweep.is_finite() {
            sweep.clamp(-360.0, 360.0)
        } else {
            -360.0
        };
        self
    }
    pub fn layout(mut self, layout: RadialLayout) -> Self {
        self.layout = layout;
        self
    }
}
