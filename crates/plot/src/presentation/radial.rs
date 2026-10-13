use std::collections::HashMap;
use gpui::{Background, Pixels, px};
use super::{Stroke, TextPresentation};
/// Radial bar visuals, independent of samples and animation state.
#[derive(Clone, Debug)]
pub struct RadialPresentation {
    pub legend: super::LegendPresentation,
    pub metrics: super::ChartMetrics,
    pub palette: Vec<Background>,
    pub bars: HashMap<usize, Background>,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub gap: Pixels,
    pub padding: Pixels,
    pub track: Option<Background>,
    pub rounded: bool,
    /// Shared polar grid: one circular centerline per track plus radial spokes.
    pub grid: Option<Stroke>,
    pub grid_spokes: usize,
    pub empty_state: Option<super::EmptyStatePresentation>,
    pub label_text: TextPresentation,
    pub center_title: TextPresentation,
    pub center_caption: TextPresentation,
}
impl Default for RadialPresentation {
    fn default() -> Self {
        let pie = super::PiePresentation::default();
        Self {
            legend: pie.legend,
            metrics: pie.metrics,
            palette: pie.palette,
            bars: HashMap::new(),
            inner_radius: 0.3,
            outer_radius: 1.0,
            gap: px(4.0),
            padding: px(12.0),
            track: Some(gpui::rgb(0x64748b).opacity(0.15).into()),
            rounded: false,
            grid: None,
            grid_spokes: 12,
            empty_state: Some(super::EmptyStatePresentation::new(
                "No positive radial values",
                gpui::rgb(0x64748b).into(),
            )),
            label_text: pie.label_text,
            center_title: pie.center_title,
            center_caption: pie.center_caption,
        }
    }
}
impl RadialPresentation {
    pub(crate) fn fill_for(&self, index: usize) -> Background {
        self.bars.get(&index).copied().unwrap_or_else(|| {
            self.palette
                .get(index % self.palette.len().max(1))
                .copied()
                .unwrap_or_else(|| gpui::rgb(0x3b82f6).into())
        })
    }
}
