use std::{collections::HashMap, sync::Arc};
use gpui::{Background, Pixels, px, rgb};
use super::{ChartMetrics, EmptyStatePresentation, LegendPresentation, TextPresentation};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PieLabelPlacement {
    Inside,
    #[default]
    Outside,
}
/// Focused pie appearance; no samples, angular mapping, or active selection state.
#[derive(Clone, Debug)]
pub struct PiePresentation {
    pub palette: Vec<Background>,
    /// Source sample-index overrides. Missing/omitted samples do not shift colors.
    pub slices: HashMap<usize, Background>,
    /// Hole radius / normal outer radius, clamped to 0..=0.95 by rendering.
    pub inner_radius: f32,
    /// Normal outer radius / available chart radius, clamped to 0..=1.
    pub outer_radius: f32,
    pub padding: Pixels,
    pub active_expansion: Pixels,
    /// Split active expansion into a main sector and outer band with this radial gap.
    pub active_band_gap: Option<Pixels>,
    pub label_placement: PieLabelPlacement,
    pub label_gap: Pixels,
    pub label_lines: bool,
    pub label_text: TextPresentation,
    pub center_title: TextPresentation,
    pub center_caption: TextPresentation,
    pub legend: LegendPresentation,
    pub metrics: ChartMetrics,
    pub empty_state: Option<EmptyStatePresentation>,
}
impl Default for PiePresentation {
    fn default() -> Self {
        let text = TextPresentation::new(rgb(0x64748b).into());
        let mut title = text.clone();
        title.font_size = px(28.0);
        title.line_height = px(34.0);
        let mut legend = LegendPresentation::new(text.color);
        legend.visible = false;
        Self {
            palette: [0x93c5fd, 0x2b7fff, 0x155dfc, 0x1447e6, 0x1e40af]
                .map(|color| Background::from(rgb(color)))
                .to_vec(),
            slices: HashMap::new(),
            inner_radius: 0.0,
            outer_radius: 1.0,
            padding: px(8.0),
            active_expansion: px(12.0),
            active_band_gap: None,
            label_placement: PieLabelPlacement::Outside,
            label_gap: px(20.0),
            label_lines: false,
            label_text: text.clone(),
            center_title: title,
            center_caption: text.clone(),
            legend,
            metrics: ChartMetrics {
                legend_swatch_width: px(8.0),
                legend_swatch_height: px(8.0),
                ..ChartMetrics::default()
            },
            empty_state: Some(EmptyStatePresentation::new("No positive finite values", text.color)),
        }
    }
}
impl PiePresentation {
    pub fn fill_for(&self, index: usize) -> Background {
        self.slices.get(&index).copied().unwrap_or_else(|| {
            if self.palette.is_empty() {
                rgb(0x3b82f6).into()
            } else {
                self.palette[index % self.palette.len()]
            }
        })
    }
}
/// Live pie appearance resolved each render. Hosts request redraws after theme changes.
pub trait PieStyleProvider: Send + Sync + 'static {
    fn resolve(&self) -> PiePresentation;
}
impl PieStyleProvider for PiePresentation {
    fn resolve(&self) -> PiePresentation {
        self.clone()
    }
}
impl<F: Fn() -> PiePresentation + Send + Sync + 'static> PieStyleProvider for F {
    fn resolve(&self) -> PiePresentation {
        self()
    }
}
pub(crate) type PieProvider = Arc<dyn PieStyleProvider>;

/// Concentric band ratios measured against the chart's common outer radius.
#[derive(Clone, Copy, Debug)]
pub struct PieRingPresentation {
    pub inner_radius: f32,
    pub outer_radius: f32,
}
impl PieRingPresentation {
    pub fn new(inner_radius: f32, outer_radius: f32) -> Self {
        Self { inner_radius, outer_radius }
    }
}
