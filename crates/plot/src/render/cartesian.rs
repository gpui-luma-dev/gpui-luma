use std::{time::Instant, rc::Rc, cell::RefCell};

use gpui::{AnyElement, Bounds, div, ContentMask, IntoElement, Pixels, canvas, prelude::*};

use crate::{
    data::Dataset,
    plots::CartesianPlot,
    presentation::CartesianPresentation,
    views::{ChartRenderStats, RenderObserver, PointTooltip, PointEvents, PlotPointEvent},
};
use super::{
    axis::{self, AxisLayout},
    geometry::{CartesianGeometry, layout, scale_ticks},
    grid::paint_grid,
    line::paint_line,
    area::paint_area,
};

struct PointHitTest {
    geometry: Rc<CartesianGeometry>,
    visible: Bounds<Pixels>,
}

struct CartesianPrepaint {
    geometry: Rc<CartesianGeometry>,
    axes: Vec<AxisLayout>,
    stats: Option<ChartRenderStats>,
    hovered: Vec<super::points::HoverPoint>,
    tooltip: Option<AnyElement>,
    bar_elements: Option<Vec<AnyElement>>,
    bar_labels: Vec<super::bar::LabelLayout>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cartesian_canvas(
    data: Dataset,
    plot: CartesianPlot,
    presentation: CartesianPresentation,
    observer: Option<RenderObserver>,
    draw_progress: f32,
    point_tooltip: Option<PointTooltip>,
    point_events: Option<PointEvents>,
    bar_options: crate::views::BarViewOptions,
    annotations: Vec<crate::ChartAnnotation>,
) -> impl IntoElement {
    let axes = presentation.axis_components().to_vec();
    let padding = presentation.metrics.plot_padding;
    let measured = observer.is_some();
    let interactive = super::points::interactive(
        &presentation,
        point_tooltip.is_some() || point_events.is_some() || bar_options.tooltip.is_some(),
    );
    let hover_style = presentation.clone();
    let bar_style = presentation.clone();
    let hover_data = data.clone();
    // Input handlers share the latest measured geometry without re-projecting data.
    let hit_test: Rc<RefCell<Option<PointHitTest>>> = Rc::default();
    let paint_hit_test = hit_test.clone();
    let hover_events = point_events.clone();
    let click_data = data.clone();
    let click_style = presentation.clone();
    let canvas = canvas(
        move |bounds, window, cx| {
            let started = measured.then(Instant::now);
            // Measure scale labels before reserving space for the visible edges.
            let (x_ticks, y_ticks) = scale_ticks(&data, &plot);
            let axes: Vec<_> = axes.iter().map(|axis| axis::measure(axis, &x_ticks, &y_ticks, window)).collect();
            let geometry = Rc::new(layout(&data, &plot, axis::insets(&axes, padding), bounds));
            let stats = started.map(|started| ChartRenderStats {
                source_points: plot
                    .series_ids()
                    .filter_map(|id| data.get(id))
                    .map(|series| series.samples().len())
                    .sum(),
                valid_points: geometry
                    .lines
                    .iter()
                    .chain(geometry.areas.iter().map(|area| &area.boundary))
                    .map(|line| line.valid_points)
                    .sum::<usize>()
                    + geometry.bars.iter().map(|bar| bar.valid_points).sum::<usize>(),
                visible_points: geometry
                    .lines
                    .iter()
                    .chain(geometry.areas.iter().map(|area| &area.boundary))
                    .map(|line| line.visible_points)
                    .sum::<usize>()
                    + geometry.bars.iter().map(|bar| bar.rectangles.len()).sum::<usize>(),
                x_domain: geometry.x_domain,
                retained_points: geometry
                    .lines
                    .iter()
                    .chain(geometry.areas.iter().map(|area| &area.boundary))
                    .flat_map(|line| &line.runs)
                    .map(Vec::len)
                    .sum::<usize>()
                    + geometry.bars.iter().map(|bar| bar.rectangles.len()).sum::<usize>(),
                runs: geometry
                    .lines
                    .iter()
                    .chain(geometry.areas.iter().map(|area| &area.boundary))
                    .map(|line| line.runs.len())
                    .sum(),
                plot_width: geometry.plot_bounds.size.width.into(),
                plot_offset_x: (geometry.plot_bounds.origin.x - bounds.origin.x).into(),
                plot_height: geometry.plot_bounds.size.height.into(),
                prepare_time: started.elapsed(),
                paint_time: std::time::Duration::ZERO,
            });
            let visible = reveal_bounds(geometry.plot_bounds, draw_progress);
            let hovered = if interactive {
                {
                    let bars = super::bar::select(&geometry, &hover_data, &bar_style, window.mouse_position(), visible);
                    if bars.is_empty() {
                        super::points::select(&geometry, &hover_data, &hover_style, window.mouse_position(), visible)
                    } else {
                        bars
                    }
                }
            } else {
                Vec::new()
            };
            if let Some(observer) = &hover_events {
                super::points::report_hover(&hovered, observer, window, cx);
                *paint_hit_test.borrow_mut() = Some(PointHitTest { geometry: geometry.clone(), visible });
            }
            let tooltip = if let Some(factory) = &bar_options.tooltip {
                let selected = super::bar::select(&geometry, &hover_data, &bar_style, window.mouse_position(), visible);
                super::points::tooltip_target(
                    &selected,
                    &PointTooltip { factory: factory.clone(), motion: bar_options.motion },
                    visible,
                    &geometry,
                    false,
                    window,
                    cx,
                )
            } else {
                point_tooltip.as_ref().and_then(|factory| {
                    super::points::tooltip_target(
                        &hovered,
                        factory,
                        visible,
                        &geometry,
                        hover_style.has_slice_markers(),
                        window,
                        cx,
                    )
                })
            };
            let bar_elements = bar_options.template.as_ref().map(|template| {
                super::bar::prepare_templates(
                    &geometry.bars,
                    &bar_style,
                    plot.bar_orientation_spec(),
                    template,
                    window,
                    cx,
                )
            });
            let bar_labels = bar_options.labels.as_ref().map_or_else(Vec::new, |labels| {
                super::bar::prepare_labels(
                    &geometry.bars,
                    &bar_style,
                    plot.bar_orientation_spec(),
                    labels,
                    visible,
                    bounds,
                    window,
                )
            });
            CartesianPrepaint { geometry, axes, stats, hovered, tooltip, bar_elements, bar_labels }
        },
        move |bounds, mut prepaint, window, cx| {
            let started = measured.then(Instant::now);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                paint_grid(&prepaint.geometry, presentation.grid, window);
                for axis in &prepaint.axes {
                    axis::paint(axis, &prepaint.geometry, bounds, window, cx);
                }
                window.with_content_mask(
                    Some(ContentMask { bounds: reveal_bounds(prepaint.geometry.plot_bounds, draw_progress) }),
                    |window| {
                        for (index, area) in prepaint.geometry.areas.iter().enumerate() {
                            paint_area(
                                area,
                                presentation.area_for(&area.boundary.series),
                                prepaint
                                    .hovered
                                    .iter()
                                    .find(|point| point.area && point.layer == index)
                                    .map(|point| point.position),
                                window,
                            );
                        }
                        if let Some(elements) = &mut prepaint.bar_elements {
                            for element in elements {
                                element.paint(window, cx);
                            }
                        } else {
                            for bar in &prepaint.geometry.bars {
                                super::bar::paint_bar(bar, presentation.bar_for(&bar.series), window);
                            }
                        }
                        for (index, line) in prepaint.geometry.lines.iter().enumerate() {
                            paint_line(
                                &line.runs,
                                presentation.line_for(&line.series),
                                line.interpolation,
                                prepaint
                                    .hovered
                                    .iter()
                                    .find(|point| !point.area && point.layer == index)
                                    .map(|point| point.position),
                                window,
                            );
                        }
                    },
                );
            });
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                super::bar::paint_labels(&prepaint.bar_labels, window, cx);
            });
            super::annotation::paint(&annotations, &prepaint.geometry, window, cx);
            if let Some(tooltip) = &mut prepaint.tooltip {
                tooltip.paint(window, cx);
            }
            if let (Some(observer), Some(mut stats), Some(started)) = (&observer, prepaint.stats, started) {
                stats.paint_time = started.elapsed();
                observer.report(stats, cx);
            }
        },
    )
    .size_full();
    div()
        .id("plot-canvas")
        .size_full()
        .child(canvas)
        .when(interactive, |root| {
            root.on_mouse_move(|_, window, _| window.refresh()).on_hover(|_, window, _| window.refresh())
        })
        .when_some(point_events, |root, observer| {
            root.on_click(move |event, _, cx| {
                let points: Vec<_> = hit_test.borrow().as_ref().map_or_else(Vec::new, |hit_test| {
                    select_observation(
                        &hit_test.geometry,
                        &click_data,
                        &click_style,
                        event.position(),
                        hit_test.visible,
                    )
                    .into_iter()
                    .map(|point| point.info)
                    .collect()
                });
                if !points.is_empty() {
                    observer.report(PlotPointEvent::Select { points }, cx);
                }
            })
        })
}

// Clip only data painting: curves are built from their complete runs, keeping
// interpolation derivatives and gaps identical throughout the reveal.
fn reveal_bounds(mut bounds: Bounds<Pixels>, progress: f32) -> Bounds<Pixels> {
    bounds.size.width *= progress;
    bounds
}

fn select_observation(
    geometry: &CartesianGeometry,
    data: &Dataset,
    style: &CartesianPresentation,
    pointer: gpui::Point<Pixels>,
    visible: Bounds<Pixels>,
) -> Vec<super::points::HoverPoint> {
    let bars = super::bar::select(geometry, data, style, pointer, visible);
    if bars.is_empty() {
        super::points::select(geometry, data, style, pointer, visible)
    } else {
        bars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    #[test]
    fn bar_selection_reports_source_values_without_markers_or_tooltips() {
        use gpui::{point, size, px, Edges};
        let data = Dataset::new().with_series(crate::Series::new("a", "A", [crate::Sample::new(0.0, 5.0)])).unwrap();
        let plot = crate::CartesianPlot::new()
            .x_scale(crate::CategoryScale::new(["Only"]))
            .bar(crate::BarLayer::new("a"));
        let geometry =
            layout(&data, &plot, Edges::all(px(0.0)), Bounds::new(point(px(0.0), px(0.0)), size(px(200.0), px(200.0))));
        let selected = select_observation(
            &geometry,
            &data,
            &CartesianPresentation::from(crate::CartesianStyle::default()),
            geometry.bars[0].rectangles[0].center(),
            geometry.plot_bounds,
        );
        assert_eq!(selected.len(), 1);
        assert!(selected[0].bar);
        assert_eq!(selected[0].info.sample.y, 5.0);
    }
    #[test]
    fn reveal_advances_across_the_plot_without_moving_its_origin_or_y_bounds() {
        let bounds = Bounds::new(point(px(30.0), px(20.0)), size(px(200.0), px(100.0)));
        for (progress, width) in [(0.0, 0.0), (0.5, 100.0), (1.0, 200.0)] {
            let mask = reveal_bounds(bounds, progress);
            assert_eq!(mask.origin, bounds.origin);
            assert_eq!(mask.size.height, bounds.size.height);
            assert_eq!(mask.size.width, px(width));
        }
    }
}
