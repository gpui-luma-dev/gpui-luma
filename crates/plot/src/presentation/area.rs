use gpui::Background;
use super::LinePresentation;

/// Area paint, independent of samples, scales, and timing.
/// Accepts native GPUI solid colors, gradients, and pattern backgrounds.
/// Gradients use GPUI path bounds independently for each contiguous filled region.
#[derive(Clone, Copy, Debug)]
pub struct AreaPresentation {
    pub fill: Background,
    /// Optional boundary stroke and sample markers; baseline edges are never stroked.
    pub outline: Option<LinePresentation>,
}

impl AreaPresentation {
    pub fn new(fill: impl Into<Background>) -> Self {
        Self { fill: fill.into(), outline: None }
    }
    pub fn outline(mut self, outline: LinePresentation) -> Self {
        self.outline = Some(outline);
        self
    }
}
