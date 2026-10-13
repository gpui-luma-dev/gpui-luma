use std::collections::HashMap;
use gpui::Hsla;
use crate::data::SeriesId;
use super::{
    BarPresentation, AreaPresentation, Axis, AxisPresentation, AxisSide, ChartMetrics, EmptyStatePresentation,
    GridPresentation, LegendPresentation, LinePresentation,
};

/// Focused per-series overrides. Unset components inherit the plot defaults.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeriesStyle {
    pub line: Option<LinePresentation>,
    pub area: Option<AreaPresentation>,
    pub bar: Option<BarPresentation>,
}
impl SeriesStyle {
    /// A matching line and translucent area, without sample markers.
    pub fn new(color: Hsla) -> Self {
        let line = LinePresentation::new(color);
        Self {
            line: Some(line),
            area: Some(AreaPresentation::new(color.opacity(0.25)).outline(line)),
            bar: Some(BarPresentation::new(color)),
        }
    }
}
impl From<LinePresentation> for SeriesStyle {
    fn from(line: LinePresentation) -> Self {
        Self { line: Some(line), area: None, bar: None }
    }
}
impl From<AreaPresentation> for SeriesStyle {
    fn from(area: AreaPresentation) -> Self {
        Self { line: None, area: Some(area), bar: None }
    }
}

impl From<BarPresentation> for SeriesStyle {
    fn from(bar: BarPresentation) -> Self {
        Self { bar: Some(bar), ..Self::default() }
    }
}

/// One resolved plot style. No samples, scales, frame decoration or animation state.
/// Providers resolve this value each render; local overrides are applied afterward.
#[derive(Clone, Debug)]
pub struct PlotStyle {
    pub axes: Vec<Axis>,
    pub line: LinePresentation,
    pub area: AreaPresentation,
    pub bar: BarPresentation,
    pub grid: GridPresentation,
    pub legend: LegendPresentation,
    pub metrics: ChartMetrics,
    pub empty_state: Option<EmptyStatePresentation>,
    pub series: HashMap<SeriesId, SeriesStyle>,
}
impl PlotStyle {
    pub fn new(axis: AxisPresentation, line: LinePresentation) -> Self {
        Self {
            axes: vec![Axis::new(AxisSide::Bottom, axis.clone()), Axis::new(AxisSide::Left, axis.clone())],
            line,
            bar: BarPresentation::new(line.stroke.color),
            area: AreaPresentation::new(line.stroke.color.opacity(0.25)).outline(line),
            grid: GridPresentation::new(axis.stroke.color),
            legend: LegendPresentation::new(axis.stroke.color),
            metrics: ChartMetrics::default(),
            empty_state: Some(EmptyStatePresentation::new("No finite samples", axis.tick_labels.color)),
            series: HashMap::new(),
        }
    }
    pub(crate) fn set_series_style(&mut self, id: SeriesId, style: SeriesStyle) {
        let entry = self.series.entry(id).or_default();
        if style.line.is_some() {
            entry.line = style.line;
        }
        if style.bar.is_some() {
            entry.bar = style.bar;
        }
        if style.area.is_some() {
            entry.area = style.area;
        }
    }
}
impl Default for PlotStyle {
    fn default() -> Self {
        let mut style = Self::new(
            AxisPresentation::new(gpui::rgb(0x64748b).into()),
            LinePresentation::new(gpui::rgb(0x2563eb).into()),
        );
        style.grid = GridPresentation::new(gpui::rgb(0xe2e8f0).into());
        style
    }
}

/// A reusable, look-independent source of current plot appearance.
/// Resolve uses current theme state; the host must request a redraw after changes.
/// Static `PlotStyle` values and closures returning `PlotStyle` are also providers.
pub trait PlotStyleProvider: Send + Sync + 'static {
    fn resolve(&self) -> PlotStyle;
}
impl PlotStyleProvider for PlotStyle {
    fn resolve(&self) -> PlotStyle {
        self.clone()
    }
}
impl<F: Fn() -> PlotStyle + Send + Sync + 'static> PlotStyleProvider for F {
    fn resolve(&self) -> PlotStyle {
        self()
    }
}
