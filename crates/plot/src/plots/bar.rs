use crate::data::SeriesId;

/// Direction of increasing bar values. Negative values extend the opposite way.
/// Samples always use X for category and Y for value; scales keep these logical roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarOrientation {
    #[default]
    Up,
    Down,
    Left,
    Right,
}
impl BarOrientation {
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Placement of series sharing a category. Stacks accumulate positive and negative
/// deltas from each layer's baseline separately; different baselines form separate stacks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarGrouping {
    #[default]
    Grouped,
    Stacked,
}

/// Alignment of the occupied group within its unit-wide category band.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarAlignment {
    Start,
    #[default]
    Center,
    End,
}

/// Shared bar placement policy, independent of painting and orientation.
#[derive(Clone, Copy, Debug)]
pub struct BarLayout {
    pub(crate) grouping: BarGrouping,
    pub(crate) alignment: BarAlignment,
    pub(crate) width: f64,
    pub(crate) gap: f64,
}
impl Default for BarLayout {
    fn default() -> Self {
        Self { grouping: BarGrouping::Grouped, alignment: BarAlignment::Center, width: 0.8, gap: 0.1 }
    }
}
impl BarLayout {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn grouping(mut self, grouping: BarGrouping) -> Self {
        self.grouping = grouping;
        self
    }
    pub fn alignment(mut self, alignment: BarAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    /// Occupied fraction of each unit-wide category band, clamped to 0..=1.
    /// Non-finite values restore 0.8.
    pub fn width(mut self, value: f64) -> Self {
        self.width = fraction(value, 0.8);
        self
    }
    /// Empty fraction of each grouped series slot, clamped to 0..=1.
    /// Non-finite values restore 0.1. Ignored for stacks and single-series charts.
    pub fn gap(mut self, value: f64) -> Self {
        self.gap = fraction(value, 0.1);
        self
    }
    pub fn width_fraction(self) -> f64 {
        self.width
    }
    pub fn gap_fraction(self) -> f64 {
        self.gap
    }
    pub fn grouping_policy(self) -> BarGrouping {
        self.grouping
    }
    pub fn alignment_policy(self) -> BarAlignment {
        self.alignment
    }
}
fn fraction(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

/// Select a bar series. Layout and orientation belong to the chart.
#[derive(Clone, Debug)]
pub struct BarLayer {
    series: SeriesId,
    baseline: f64,
}
impl BarLayer {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { series: series.into(), baseline: 0.0 }
    }
    /// Finite value baseline, included in automatic bounds. Invalid values restore zero.
    pub fn baseline(mut self, value: f64) -> Self {
        self.baseline = if value.is_finite() { value } else { 0.0 };
        self
    }
    pub fn series(&self) -> &SeriesId {
        &self.series
    }
    pub fn baseline_value(&self) -> f64 {
        self.baseline
    }
}
