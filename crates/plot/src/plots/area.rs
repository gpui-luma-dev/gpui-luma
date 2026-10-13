use crate::data::SeriesId;
use super::{Decimation, Interpolation, LineLayer};

/// Fill between one series and a finite Y baseline (zero by default).
/// Missing samples separate filled regions. Areas are painted behind line layers.
#[derive(Clone, Debug)]
pub struct AreaLayer {
    line: LineLayer,
    baseline: f64,
}

impl AreaLayer {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { line: LineLayer::new(series), baseline: 0.0 }
    }

    /// Baseline in data coordinates; included in automatic numeric Y bounds.
    /// Non-finite values restore the default zero baseline.
    pub fn baseline(mut self, value: f64) -> Self {
        self.baseline = if value.is_finite() { value } else { 0.0 };
        self
    }

    /// Connect the upper boundary using the same rules as a line layer.
    pub fn interpolation(mut self, interpolation: Interpolation) -> Self {
        self.line = self.line.interpolation(interpolation);
        self
    }

    /// Reduce drawing points while retaining source samples and scale domains.
    pub fn decimation(mut self, decimation: Decimation) -> Self {
        self.line = self.line.decimation(decimation);
        self
    }

    pub fn series(&self) -> &SeriesId {
        self.line.series()
    }
    pub fn baseline_value(&self) -> f64 {
        self.baseline
    }
    pub fn interpolation_policy(&self) -> Interpolation {
        self.line.interpolation_policy()
    }
    pub fn decimation_policy(&self) -> Decimation {
        self.line.decimation_policy()
    }
    pub(crate) fn boundary(&self) -> &LineLayer {
        &self.line
    }
}
