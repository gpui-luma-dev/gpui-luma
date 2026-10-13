use crate::data::{Sample, SeriesId};

use super::{BarLayout, BarOrientation, BarLayer, AreaLayer, LineLayer, NumericScale, Scale};

/// A data-free Cartesian plot specification.
///
/// Layers select series from a dataset supplied by the view; scales describe numeric mapping.
/// Presentation, interaction state, layout, and rendering are deliberately outside this type.
#[derive(Clone, Debug)]
pub struct CartesianPlot {
    x_scale: Scale,
    y_scale: Scale,
    lines: Vec<LineLayer>,
    areas: Vec<AreaLayer>,
    bars: Vec<BarLayer>,
    bar_layout: BarLayout,
    bar_orientation: BarOrientation,
}

impl CartesianPlot {
    pub fn new() -> Self {
        Self {
            x_scale: NumericScale::new().into(),
            y_scale: NumericScale::new().into(),
            lines: Vec::new(),
            areas: Vec::new(),
            bars: Vec::new(),
            bar_layout: BarLayout::default(),
            bar_orientation: BarOrientation::Up,
        }
    }

    pub fn x_scale(mut self, scale: impl Into<Scale>) -> Self {
        self.x_scale = scale.into();
        self
    }

    pub fn y_scale(mut self, scale: impl Into<Scale>) -> Self {
        self.y_scale = scale.into();
        self
    }

    pub fn line(mut self, layer: LineLayer) -> Self {
        self.lines.push(layer);
        self
    }

    pub fn area(mut self, layer: AreaLayer) -> Self {
        self.areas.push(layer);
        self
    }
    /// Add a bar layer using the shared layout and orientation.
    pub fn bar(mut self, layer: BarLayer) -> Self {
        self.bars.push(layer);
        self
    }
    pub fn bar_layout(mut self, layout: BarLayout) -> Self {
        self.bar_layout = layout;
        self
    }
    /// Map logical category/value scales to bar screen axes. Use on bar-only charts;
    /// line and area layers retain their conventional Cartesian mapping.
    pub fn bar_orientation(mut self, orientation: BarOrientation) -> Self {
        self.bar_orientation = orientation;
        self
    }
    pub fn bar_layout_spec(&self) -> BarLayout {
        self.bar_layout
    }
    pub fn bar_orientation_spec(&self) -> BarOrientation {
        self.bar_orientation
    }
    pub fn bars(&self) -> &[BarLayer] {
        &self.bars
    }
    pub fn areas(&self) -> &[AreaLayer] {
        &self.areas
    }
    pub(crate) fn series_ids(&self) -> impl Iterator<Item = &SeriesId> {
        self.lines
            .iter()
            .map(LineLayer::series)
            .chain(self.areas.iter().map(AreaLayer::series))
            .chain(self.bars.iter().map(BarLayer::series))
    }

    pub fn x_scale_spec(&self) -> &Scale {
        &self.x_scale
    }

    pub fn y_scale_spec(&self) -> &Scale {
        &self.y_scale
    }

    pub(crate) fn accepts_sample(&self, sample: &Sample) -> bool {
        self.x_scale.accepts(sample.x) && self.y_scale.accepts(sample.y)
    }

    pub(crate) fn has_visible_bars(&self, series: &crate::data::Series) -> bool {
        let samples = series.samples();
        let visible = if let Scale::Numeric(scale) = &self.x_scale
            && let Some(domain) = scale.domain_bounds()
            && series.sorted_x()
        {
            let start = samples.partition_point(|sample| sample.x < *domain.start() - 0.5);
            let end = samples.partition_point(|sample| sample.x <= *domain.end() + 0.5);
            &samples[start..end]
        } else {
            samples
        };
        visible.iter().any(|sample| {
            self.accepts_sample(sample)
                && match &self.x_scale {
                    Scale::Numeric(scale) => scale
                        .domain_bounds()
                        .is_none_or(|domain| sample.x + 0.5 >= *domain.start() && sample.x - 0.5 <= *domain.end()),
                    Scale::Category(_) => true,
                }
        })
    }

    /// Include crossing segments even when both endpoints lie outside the X viewport.
    pub(crate) fn has_visible_samples(&self, samples: &[Sample]) -> bool {
        let domain = match &self.x_scale {
            Scale::Numeric(scale) => scale.domain_bounds(),
            Scale::Category(_) => None,
        };
        let mut previous: Option<f64> = None;
        for sample in samples {
            if !self.accepts_sample(sample) {
                previous = None;
                continue;
            }
            let Some(domain) = domain else { return true };
            if domain.contains(&sample.x)
                || previous.is_some_and(|x| x.min(sample.x) <= *domain.end() && x.max(sample.x) >= *domain.start())
            {
                return true;
            }
            previous = Some(sample.x);
        }
        false
    }

    pub fn lines(&self) -> &[LineLayer] {
        &self.lines
    }
}

impl Default for CartesianPlot {
    fn default() -> Self {
        Self::new()
    }
}
