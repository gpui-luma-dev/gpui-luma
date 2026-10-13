use std::f64::consts::TAU;
use gpui::{Bounds, Pixels, Point, PathBuilder, IntoElement, TextAlign, canvas, point, px, size, fill, prelude::*};
use crate::{Dataset, RadarPlot};
use crate::views::radar::RadarChartBuilder;
use super::axis::positive;

// Reject invalid coordinates and duplicate spokes rather than inventing a polygon edge.
fn values(data: &Dataset, plot: &RadarPlot, count: usize) -> Vec<Vec<Option<f64>>> {
    plot.series
        .iter()
        .map(|id| {
            let mut values = vec![None; count];
            let mut seen = vec![false; count];
            if let Some(series) = data.get(id) {
                for sample in series.samples() {
                    if !sample.x.is_finite() || sample.x < 0.0 || sample.x.fract() != 0.0 || sample.x >= count as f64 {
                        continue;
                    }
                    let index = sample.x as usize;
                    if seen[index] {
                        values[index] = None;
                        continue;
                    }
                    seen[index] = true;
                    values[index] = (sample.y.is_finite() && sample.y >= 0.0).then_some(sample.y);
                }
            }
            values
        })
        .collect()
}
fn maximum(values: &[Vec<Option<f64>>], plot: &RadarPlot) -> f64 {
    plot.maximum
        .unwrap_or_else(|| values.iter().flatten().flatten().copied().fold(0.0, f64::max).max(f64::MIN_POSITIVE))
}
fn vertex(
    center: Point<Pixels>,
    radius: Pixels,
    start: f64,
    index: usize,
    count: usize,
    fraction: f64,
) -> Point<Pixels> {
    let angle = start.to_radians() + TAU * index as f64 / count as f64;
    center + point(radius * (angle.cos() * fraction) as f32, radius * (angle.sin() * fraction) as f32)
}
fn closed(points: &[Point<Pixels>]) -> impl Iterator<Item = Point<Pixels>> + '_ {
    points.iter().copied().chain(points.first().copied())
}
pub(crate) fn radar_canvas(builder: RadarChartBuilder, style: crate::RadarPresentation) -> impl IntoElement {
    let count = builder.categories.len();
    let values = values(&builder.data, &builder.plot, count);
    let maximum = maximum(&values, &builder.plot);
    let prepare_style = style.clone();
    let categories = builder.categories.clone();
    let labels = builder.labels.clone();
    let prepare = builder.clone();
    let prepare_values = values.clone();
    let interactive = builder.tooltip.is_some() || builder.events.is_some();
    let events = builder.events.clone();
    let picked: std::rc::Rc<std::cell::RefCell<Option<crate::RadarSpoke>>> = Default::default();
    let prepared_pick = picked.clone();
    let canvas = canvas(
        move |bounds: Bounds<Pixels>, window, _cx| {
            let lines: Vec<_> = categories
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let label = labels.as_ref().map_or_else(|| name.clone(), |labels| labels(index, name));
                    super::axis::shape(label, &prepare_style.label_text, window)
                })
                .collect();
            let width = lines.iter().map(|line| line.width()).max().unwrap_or(px(0.0));
            let pad = positive(prepare_style.padding) + positive(prepare_style.label_gap);
            let radius = ((bounds.size.width - 2.0 * (pad + width)) / 2.0)
                .min((bounds.size.height - 2.0 * (pad + positive(prepare_style.label_text.line_height))) / 2.0)
                .max(px(0.0));
            let hovered = if prepare.progress > 0.0 {
                hovered_spoke(window.mouse_position(), bounds, radius, prepare.plot.start_angle, count)
                    .and_then(|index| spoke(&prepare, &prepare_values, index).map(|info| (index, info)))
            } else {
                None
            };
            let target = hovered.as_ref().map(|(_, info)| info.clone());
            *prepared_pick.borrow_mut() = target.clone();
            if let Some(events) = &prepare.events {
                super::selection::report(target, events, window, _cx);
            }
            let tooltip = prepare.tooltip.as_ref().and_then(|factory| {
                let (index, info) = hovered.as_ref()?;
                let distance = prepare_values.iter().filter_map(|values| values[*index]).fold(0.0, f64::max);
                let anchor = vertex(
                    bounds.center(),
                    radius,
                    prepare.plot.start_angle,
                    *index,
                    count,
                    (distance / maximum).clamp(0.0, 1.0) * f64::from(prepare.progress),
                );
                tooltip::tooltip(Some(info), anchor, bounds, factory, prepare.tooltip_motion, window, _cx)
            });
            // Clear pending tooltip ownership when leaving the plot or encountering an empty spoke.
            let tooltip = if hovered.is_none() {
                prepare.tooltip.as_ref().and_then(|factory| {
                    tooltip::tooltip(None, bounds.center(), bounds, factory, prepare.tooltip_motion, window, _cx)
                })
            } else {
                tooltip
            };
            (bounds.center(), radius, lines, tooltip)
        },
        move |bounds, (center, radius, lines, mut tooltip), window, cx| {
            if count < 3 || radius <= px(0.0) {
                return;
            }
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                let mut levels: Vec<_> = style
                    .grid_levels
                    .iter()
                    .copied()
                    .filter(|level| level.is_finite() && *level > 0.0 && *level <= 1.0)
                    .collect();
                levels.sort_by(|a, b| b.total_cmp(a));
                for level in levels {
                    let points: Vec<_> = match style.grid_shape {
                        crate::RadarGridShape::Polygon => (0..count)
                            .map(|i| vertex(center, radius, builder.plot.start_angle, i, count, f64::from(level)))
                            .collect(),
                        crate::RadarGridShape::Circle => {
                            (0..96).map(|i| vertex(center, radius, 0.0, i, 96, f64::from(level))).collect()
                        }
                    };
                    if let Some(fill) = style.grid_fill {
                        let mut path = PathBuilder::fill();
                        path.move_to(points[0]);
                        for p in &points[1..] {
                            path.line_to(*p);
                        }
                        path.close();
                        if let Ok(path) = path.build() {
                            window.paint_path(path, fill);
                        }
                    }
                    if let Some(stroke) = style.grid {
                        super::line::paint_path(closed(&points), stroke, window);
                    }
                }
                if style.spokes
                    && let Some(stroke) = style.grid
                {
                    for i in 0..count {
                        super::line::paint_path(
                            [center, vertex(center, radius, builder.plot.start_angle, i, count, 1.0)],
                            stroke,
                            window,
                        );
                    }
                }
                for (index, values) in values.iter().enumerate() {
                    let presentation = style.series_style(&builder.plot.series[index], index);
                    let points: Vec<_> = values
                        .iter()
                        .enumerate()
                        .map(|(i, value)| {
                            value.map(|v| {
                                vertex(
                                    center,
                                    radius,
                                    builder.plot.start_angle,
                                    i,
                                    count,
                                    (v / maximum).clamp(0.0, 1.0) * f64::from(builder.progress),
                                )
                            })
                        })
                        .collect();
                    if points.iter().all(Option::is_some) && builder.progress > 0.0 {
                        let polygon: Vec<_> = points.iter().flatten().copied().collect();
                        if let Some(fill) = presentation.fill {
                            let mut path = PathBuilder::fill();
                            path.move_to(polygon[0]);
                            for p in &polygon[1..] {
                                path.line_to(*p);
                            }
                            path.close();
                            if let Ok(path) = path.build() {
                                window.paint_path(path, fill);
                            }
                        }
                    }
                    if let Some(stroke) = presentation.stroke {
                        for i in 0..count {
                            if let (Some(a), Some(b)) = (points[i], points[(i + 1) % count]) {
                                super::line::paint_path([a, b], stroke, window);
                            }
                        }
                    }
                    let dot = positive(presentation.dot_radius);
                    if dot > px(0.0) && builder.progress > 0.0 {
                        for p in points.iter().flatten() {
                            window.paint_quad(
                                fill(
                                    Bounds::new(*p - point(dot, dot), size(dot * 2.0, dot * 2.0)),
                                    presentation.dot_color,
                                )
                                .corner_radii(dot),
                            );
                        }
                    }
                }
                if values.iter().flatten().all(Option::is_none)
                    && let Some(empty) = &style.empty_state
                {
                    let mut text = style.label_text.clone();
                    text.color = empty.text_color;
                    let line = super::axis::shape(empty.message.clone(), &text, window);
                    line.paint(
                        center - point(line.width() / 2.0, positive(text.line_height) / 2.0),
                        positive(text.line_height),
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    )
                    .ok();
                }

                for (i, line) in lines.iter().enumerate() {
                    let anchor =
                        vertex(center, radius + positive(style.label_gap), builder.plot.start_angle, i, count, 1.0);
                    let direction = anchor - center;
                    let x = if direction.x.abs() < px(0.1) {
                        anchor.x - line.width() / 2.0
                    } else if direction.x < px(0.0) {
                        anchor.x - line.width()
                    } else {
                        anchor.x
                    };
                    let origin = point(x, anchor.y - positive(style.label_text.line_height) / 2.0);
                    line.paint(origin, positive(style.label_text.line_height), TextAlign::Left, None, window, cx).ok();
                }
            });
            if let Some(tooltip) = &mut tooltip {
                tooltip.paint(window, cx);
            }
        },
    )
    .size_full();
    gpui::div()
        .id("radar-canvas")
        .size_full()
        .child(canvas)
        .when(interactive, |root| {
            root.on_mouse_move(|_, window, _| window.refresh()).on_hover(|_, window, _| window.refresh())
        })
        .when_some(events, |root, events| {
            root.on_click(move |_, _, cx| {
                if let Some(target) = picked.borrow().clone() {
                    (events.0)(crate::SelectionEvent::Select { target }, cx);
                }
            })
        })
}

// Angular picking works with or without dots, and reports all series on the spoke.
fn hovered_spoke(
    pointer: Point<Pixels>,
    bounds: Bounds<Pixels>,
    radius: Pixels,
    start: f64,
    count: usize,
) -> Option<usize> {
    if count < 3 || radius <= px(0.0) || !bounds.contains(&pointer) {
        return None;
    }
    let delta = pointer - bounds.center();
    let x = f64::from(f32::from(delta.x));
    let y = f64::from(f32::from(delta.y));
    let distance = x.hypot(y);
    if distance < 4.0 || distance > f64::from(f32::from(radius)) {
        return None;
    }
    let coordinate = (y.atan2(x) - start.to_radians()).rem_euclid(TAU) / TAU * count as f64;
    Some(coordinate.round() as usize % count)
}
fn spoke(builder: &RadarChartBuilder, values: &[Vec<Option<f64>>], index: usize) -> Option<crate::RadarSpoke> {
    let points = builder
        .plot
        .series
        .iter()
        .enumerate()
        .filter_map(|(layer, id)| {
            values[layer][index]?;
            let series = builder.data.get(id)?;
            let (sample_index, sample) =
                series.samples().iter().enumerate().find(|(_, sample)| sample.x == index as f64)?;
            Some(crate::RadarPoint {
                series: id.clone(),
                label: series.label().to_owned().into(),
                sample: *sample,
                sample_index,
            })
        })
        .collect::<Vec<_>>();
    (!points.is_empty()).then(|| crate::RadarSpoke {
        category: builder.categories[index].clone(),
        category_index: index,
        points,
    })
}
mod tooltip;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Sample, Series};
    #[test]
    fn angular_picking_rotates_wraps_and_excludes_center_and_outside() {
        let bounds = Bounds::new(point(px(20.0), px(30.0)), size(px(200.0), px(200.0)));
        for start in [-90.0, 0.0, 270.0] {
            for index in 0..6 {
                let pointer = vertex(bounds.center(), px(80.0), start, index, 6, 1.0);
                assert_eq!(hovered_spoke(pointer, bounds, px(90.0), start, 6), Some(index));
            }
        }
        assert_eq!(hovered_spoke(bounds.center(), bounds, px(90.0), -90.0, 6), None);
        assert_eq!(hovered_spoke(point(px(210.0), px(220.0)), bounds, px(90.0), -90.0, 6), None);
        assert_eq!(hovered_spoke(bounds.center(), bounds, px(90.0), 0.0, 2), None);
    }
    #[test]
    fn spokes_map_by_coordinate_with_shared_scale_and_invalid_gaps() {
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "A",
                [
                    Sample::new(2.0, 40.0),
                    Sample::new(0.0, 20.0),
                    Sample::new(1.0, -1.0),
                    Sample::new(3.0, f64::NAN),
                    Sample::new(4.0, 10.0),
                    Sample::new(4.0, 30.0),
                    Sample::new(0.5, 1000.0),
                    Sample::new(f64::NAN, 1000.0),
                ],
            ))
            .unwrap()
            .with_series(Series::new("b", "B", [Sample::new(0.0, 80.0), Sample::new(1.0, 0.0)]))
            .unwrap();
        let plot = RadarPlot::new("a").series("b").series("b");
        let values = values(&data, &plot, 5);
        assert_eq!(values[0], [Some(20.0), None, Some(40.0), None, None]);
        assert_eq!(values[1], [Some(80.0), Some(0.0), None, None, None]);
        assert_eq!(values.len(), 2);
        assert_eq!(maximum(&values, &plot), 80.0);
        assert_eq!(maximum(&values, &plot.clone().maximum(50.0)), 50.0);
        assert_eq!(maximum(&values, &plot.clone().maximum(f64::NAN)), 80.0);
        assert!(maximum(&[], &plot).is_finite());
    }
    #[test]
    fn radial_motion_grows_from_fixed_center_and_closes_boundary() {
        let center = point(px(150.0), px(100.0));
        let top = vertex(center, px(80.0), -90.0, 0, 6, 1.0);
        assert!((f32::from(top.x) - 150.0).abs() < 0.001);
        assert_eq!(top.y, px(20.0));
        let halfway = vertex(center, px(80.0), -90.0, 0, 6, 0.5);
        assert_eq!(halfway.y, px(60.0));
        for i in 0..6 {
            assert_eq!(vertex(center, px(80.0), -90.0, i, 6, 0.0), center);
        }
        let polygon: Vec<_> = (0..6).map(|i| vertex(center, px(80.0), -90.0, i, 6, 1.0)).collect();
        let boundary: Vec<_> = closed(&polygon).collect();
        assert_eq!(boundary.len(), 7);
        assert_eq!(boundary.first(), boundary.last());
    }
}

#[cfg(test)]
mod tooltip_tests;
