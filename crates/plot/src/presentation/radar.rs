use std::collections::HashMap;
use gpui::{Background, Pixels, px, rgb};
use super::{Stroke, TextPresentation, LegendPresentation, ChartMetrics};
use crate::SeriesId;

/// Independent fill, outline and sample dots for a radar series.
#[derive(Clone, Debug)]
pub struct RadarSeriesPresentation {
    pub fill: Option<Background>,
    pub stroke: Option<Stroke>,
    pub dot_radius: Pixels,
    pub dot_color: gpui::Hsla,
}
impl RadarSeriesPresentation {
    pub fn new(color: gpui::Hsla) -> Self {
        Self {
            fill: Some(color.opacity(0.5).into()),
            stroke: Some(Stroke { color, width: px(2.0) }),
            dot_radius: px(0.0),
            dot_color: color,
        }
    }
}
/// Shape of concentric grid boundaries; series remain polygons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadarGridShape {
    #[default]
    Polygon,
    Circle,
}

/// Radar visuals; grid levels are fractions of the common radial maximum.
#[derive(Clone, Debug)]
pub struct RadarPresentation {
    pub series: HashMap<SeriesId, RadarSeriesPresentation>,
    pub palette: Vec<gpui::Hsla>,
    pub grid: Option<Stroke>,
    pub grid_levels: Vec<f32>,
    pub spokes: bool,
    pub grid_shape: RadarGridShape,
    /// Optional concentric background fills, painted largest first.
    pub grid_fill: Option<Background>,
    pub legend: LegendPresentation,
    pub metrics: ChartMetrics,
    pub padding: Pixels,
    pub label_gap: Pixels,
    pub empty_state: Option<super::EmptyStatePresentation>,
    pub label_text: TextPresentation,
}
impl Default for RadarPresentation {
    fn default() -> Self {
        Self {
            series: HashMap::new(),
            palette: vec![rgb(0x93c5fd).into(), rgb(0x2b7fff).into()],
            grid: Some(Stroke { color: rgb(0x64748b).into(), width: px(1.0) }),
            grid_levels: vec![0.25, 0.5, 0.75, 1.0],
            spokes: true,
            grid_shape: RadarGridShape::Polygon,
            grid_fill: None,
            legend: {
                let mut legend = LegendPresentation::new(rgb(0x64748b).into());
                legend.visible = false;
                legend
            },
            metrics: ChartMetrics::default(),
            padding: px(12.0),
            label_gap: px(10.0),
            empty_state: Some(super::EmptyStatePresentation::new("No valid radar observations", rgb(0x64748b).into())),
            label_text: TextPresentation::new(rgb(0x64748b).into()),
        }
    }
}
impl RadarPresentation {
    pub(crate) fn series_style(&self, id: &SeriesId, index: usize) -> RadarSeriesPresentation {
        self.series.get(id).cloned().unwrap_or_else(|| {
            RadarSeriesPresentation::new(
                self.palette.get(index % self.palette.len().max(1)).copied().unwrap_or_else(|| rgb(0x3b82f6).into()),
            )
        })
    }
}
