use gpui::{App, IntoElement, RenderOnce, Window, div, prelude::*};

use super::{LegendItem, LegendView, RenderObserver, PointTooltip, PointEvents, StyleSource};

use crate::{
    data::Dataset,
    plots::CartesianPlot,
    presentation::{CartesianPresentation, SwatchPresentation},
    render,
};

/// A stateless GPUI view of a Cartesian specification and its visual treatment.
///
/// It combines data, specification, and presentation by value without introducing
/// data interaction state. Transient point hover and SDK tooltip attachments are
/// retained by the window. Its legend is a normal GPUI layout layer; private rendering
/// paints the axes, grid, and series underneath.
#[derive(IntoElement)]
pub struct CartesianChartView {
    annotations: Vec<crate::ChartAnnotation>,
    legend_interaction: crate::LegendInteraction,
    id: gpui::ElementId,
    data: Dataset,
    plot: CartesianPlot,
    style: StyleSource,
    observer: Option<RenderObserver>,
    draw_progress: f32,
    point_tooltip: Option<PointTooltip>,
    point_events: Option<PointEvents>,
    bar_options: super::BarViewOptions,
}

impl CartesianChartView {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: gpui::ElementId,
        data: Dataset,
        plot: CartesianPlot,
        style: StyleSource,
        observer: Option<RenderObserver>,
        draw_progress: f32,
        point_tooltip: Option<PointTooltip>,
        point_events: Option<PointEvents>,
        bar_options: super::BarViewOptions,
        legend_interaction: crate::LegendInteraction,
        annotations: Vec<crate::ChartAnnotation>,
    ) -> Self {
        Self {
            id,
            data,
            plot,
            style,
            observer,
            draw_progress,
            point_tooltip,
            point_events,
            bar_options,
            legend_interaction,
            annotations,
        }
    }
}

impl RenderOnce for CartesianChartView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let presentation = CartesianPresentation::from(self.style.resolve());
        let legend = presentation.legend;
        let metrics = presentation.metrics;
        let empty_state =
            (!self
                .plot
                .lines()
                .iter()
                .map(|layer| layer.series())
                .chain(self.plot.areas().iter().map(|layer| layer.series()))
                .filter_map(|id| self.data.get(id))
                .any(|series| self.plot.has_visible_samples(series.samples()))
                && !self.plot.bars().iter().any(|layer| {
                    self.data.get(layer.series()).is_some_and(|series| self.plot.has_visible_bars(series))
                }))
            .then(|| presentation.empty_state.clone())
            .flatten();
        let legend_targets: Vec<_> = self
            .plot
            .lines()
            .iter()
            .map(|l| l.series())
            .chain(self.plot.areas().iter().map(|l| l.series()))
            .chain(self.plot.bars().iter().map(|l| l.series()))
            .filter(|id| self.data.get(id).is_some())
            .map(|id| crate::LegendTarget::Series(id.clone()))
            .collect();
        let mut legend_items: Vec<_> = self
            .plot
            .lines()
            .iter()
            .filter_map(|layer| {
                let series = self.data.get(layer.series())?;
                Some(LegendItem {
                    label: series.label().to_owned().into(),
                    swatch: SwatchPresentation::Line(presentation.line_for(series.id()).stroke),
                })
            })
            .collect();

        for layer in self.plot.areas() {
            if let Some(series) = self.data.get(layer.series()) {
                let style = presentation.area_for(series.id());
                legend_items.push(LegendItem {
                    label: series.label().to_owned().into(),
                    swatch: SwatchPresentation::Fill(style.fill),
                });
            }
        }

        for layer in self.plot.bars() {
            if let Some(series) = self.data.get(layer.series()) {
                legend_items.push(LegendItem {
                    label: series.label().to_owned().into(),
                    swatch: SwatchPresentation::Fill(presentation.bar_for(series.id()).fill),
                });
            }
        }
        div()
            .id(self.id)
            .size_full()
            .flex()
            .flex_col()
            .gap(metrics.legend_gap)
            .when(legend.visible, |this| {
                this.child(
                    LegendView::new(legend_items, legend, metrics)
                        .targets(legend_targets)
                        .interaction(self.legend_interaction),
                )
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(render::cartesian_canvas(
                        self.data,
                        self.plot,
                        presentation,
                        self.observer,
                        self.draw_progress,
                        self.point_tooltip,
                        self.point_events,
                        self.bar_options,
                        self.annotations,
                    ))
                    .when_some(empty_state, |this, empty_state| {
                        this.child(
                            div()
                                .absolute()
                                .inset_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(empty_state.text_color)
                                .child(empty_state.message),
                        )
                    }),
            )
    }
}
