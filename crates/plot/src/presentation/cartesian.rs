use std::collections::HashMap;

use crate::data::SeriesId;

use super::{
    Axis, AxisPresentation, AxisSide, ChartMetrics, EmptyStatePresentation, GridPresentation, LegendPresentation,
    BarPresentation, AreaPresentation, LinePresentation,
};

/// Visual treatment for a Cartesian chart.
///
/// Line overrides are keyed by stable data identity. One default line treatment
/// makes newly added layers visible without coupling color to the dataset.
#[derive(Clone, Debug)]
pub struct CartesianPresentation {
    axes: Vec<Axis>,
    pub grid: GridPresentation,
    pub empty_state: Option<EmptyStatePresentation>,
    pub legend: LegendPresentation,
    pub metrics: ChartMetrics,
    default_line: LinePresentation,
    lines: HashMap<SeriesId, LinePresentation>,
    areas: HashMap<SeriesId, AreaPresentation>,
    default_area: Option<AreaPresentation>,
    default_bar: BarPresentation,
    bars: HashMap<SeriesId, BarPresentation>,
}

impl CartesianPresentation {
    pub fn new(axes: AxisPresentation, default_line: LinePresentation) -> Self {
        let legend = LegendPresentation::new(axes.stroke.color);
        Self {
            empty_state: Some(EmptyStatePresentation::new("No finite samples", axes.tick_labels.color)),
            grid: GridPresentation::new(axes.stroke.color),
            axes: vec![Axis::new(AxisSide::Bottom, axes.clone()), Axis::new(AxisSide::Left, axes)],
            legend,
            metrics: ChartMetrics::default(),
            default_line,
            lines: HashMap::new(),
            areas: HashMap::new(),
            default_area: None,
            default_bar: BarPresentation::new(default_line.stroke.color),
            bars: HashMap::new(),
        }
    }

    /// Replace the visible axes. An empty collection hides axes without hiding the grid.
    /// Each edge has one axis; the last component supplied for an edge wins.
    pub fn axes(mut self, axes: impl IntoIterator<Item = Axis>) -> Self {
        self.axes.clear();
        for axis in axes {
            self.axes.retain(|existing| existing.side != axis.side);
            self.axes.push(axis);
        }
        self
    }

    pub fn axis_components(&self) -> &[Axis] {
        &self.axes
    }

    pub fn grid(mut self, grid: GridPresentation) -> Self {
        self.grid = grid;
        self
    }

    pub fn legend(mut self, legend: LegendPresentation) -> Self {
        self.legend = legend;
        self
    }

    pub fn metrics(mut self, metrics: ChartMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn empty_state(mut self, empty_state: Option<EmptyStatePresentation>) -> Self {
        self.empty_state = empty_state;
        self
    }

    pub fn line(mut self, series: impl Into<SeriesId>, presentation: LinePresentation) -> Self {
        self.lines.insert(series.into(), presentation);
        self
    }

    pub fn area(mut self, series: impl Into<SeriesId>, presentation: AreaPresentation) -> Self {
        self.areas.insert(series.into(), presentation);
        self
    }

    pub(crate) fn has_hover_markers(&self) -> bool {
        let hover = |line: LinePresentation| {
            line.markers.is_some_and(|marker| marker.visibility != super::MarkerVisibility::Always)
        };
        hover(self.default_line)
            || self.lines.values().copied().any(hover)
            || self.areas.values().filter_map(|area| area.outline).any(hover)
            || self.default_area.and_then(|area| area.outline).is_some_and(hover)
    }

    pub(crate) fn has_slice_markers(&self) -> bool {
        let slice = |line: LinePresentation| {
            line.markers.is_some_and(|marker| marker.visibility == super::MarkerVisibility::HoverSlice)
        };
        slice(self.default_line)
            || self.lines.values().copied().any(slice)
            || self.areas.values().filter_map(|area| area.outline).any(slice)
            || self.default_area.and_then(|area| area.outline).is_some_and(slice)
    }

    pub fn bar(mut self, series: impl Into<SeriesId>, presentation: BarPresentation) -> Self {
        self.bars.insert(series.into(), presentation);
        self
    }
    pub fn bar_for(&self, series: &SeriesId) -> BarPresentation {
        self.bars.get(series).copied().unwrap_or(self.default_bar)
    }
    pub fn area_for(&self, series: &SeriesId) -> AreaPresentation {
        self.areas.get(series).copied().or(self.default_area).unwrap_or_else(|| {
            let line = self.line_for(series);
            AreaPresentation::new(line.stroke.color.opacity(0.25)).outline(line)
        })
    }

    pub fn line_for(&self, series: &SeriesId) -> LinePresentation {
        self.lines.get(series).copied().unwrap_or(self.default_line)
    }
}

/// Look-independent starter colors. Applications normally replace these via a theme adapter.
impl Default for CartesianPresentation {
    fn default() -> Self {
        Self::new(AxisPresentation::new(gpui::rgb(0x64748b).into()), LinePresentation::new(gpui::rgb(0x2563eb).into()))
            .grid(GridPresentation::new(gpui::rgb(0xe2e8f0).into()))
    }
}

impl From<super::PlotStyle> for CartesianPresentation {
    fn from(style: super::PlotStyle) -> Self {
        let mut result = Self::new(AxisPresentation::new(style.line.stroke.color), style.line).axes(style.axes);
        result.default_area = Some(style.area);
        result.default_bar = style.bar;
        result.grid = style.grid;
        result.legend = style.legend;
        result.metrics = style.metrics;
        result.empty_state = style.empty_state;
        for (id, series) in style.series {
            if let Some(bar) = series.bar {
                result.bars.insert(id.clone(), bar);
            }
            if let Some(line) = series.line {
                result.lines.insert(id.clone(), line);
            }
            if let Some(area) = series.area {
                result.areas.insert(id, area);
            }
        }
        result
    }
}
impl From<CartesianPresentation> for super::PlotStyle {
    fn from(presentation: CartesianPresentation) -> Self {
        let mut series = HashMap::new();
        for id in presentation.lines.keys().chain(presentation.areas.keys()).chain(presentation.bars.keys()) {
            series.insert(
                id.clone(),
                super::SeriesStyle {
                    line: Some(presentation.line_for(id)),
                    area: Some(presentation.area_for(id)),
                    bar: Some(presentation.bar_for(id)),
                },
            );
        }
        Self {
            bar: presentation.default_bar,
            axes: presentation.axes,
            line: presentation.default_line,
            area: presentation.default_area.unwrap_or_else(|| {
                AreaPresentation::new(presentation.default_line.stroke.color.opacity(0.25))
                    .outline(presentation.default_line)
            }),
            grid: presentation.grid,
            legend: presentation.legend,
            metrics: presentation.metrics,
            empty_state: presentation.empty_state,
            series,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_axes_share_scales_and_each_edge_has_one_component() {
        let style = AxisPresentation::new(gpui::black());
        let presentation = CartesianPresentation::new(style.clone(), LinePresentation::new(gpui::black())).axes([
            Axis::new(AxisSide::Left, style.clone()).title("Old"),
            Axis::new(AxisSide::Right, style.clone()).title("Right"),
            Axis::new(AxisSide::Left, style).title("New"),
        ]);
        assert_eq!(presentation.axis_components().len(), 2);
        assert_eq!(presentation.axis_components()[0].side, AxisSide::Right);
        assert_eq!(presentation.axis_components()[1].title.as_deref(), Some("New"));
        assert!(presentation.grid.horizontal.is_some());
        assert!(presentation.axes([]).axis_components().is_empty());
    }
}
