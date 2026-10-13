use std::f64::consts::TAU;
use gpui::{Bounds, Pixels, Point, IntoElement, Window, canvas, point, px, TextAlign, prelude::*};
use crate::{RadialBar, RadialLayout, RadialPresentation};
use crate::views::radial::RadialChartBuilder;
use super::axis::positive;
struct ArcBar {
    info: RadialBar,
    inner: Pixels,
    outer: Pixels,
    start: f64,
    sweep: f64,
}
fn ratio(value: f32, default: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        default
    }
}
fn polar(center: Point<Pixels>, radius: Pixels, angle: f64) -> Point<Pixels> {
    center + point(radius * angle.cos() as f32, radius * angle.sin() as f32)
}
pub(crate) fn observations(builder: &RadialChartBuilder) -> Vec<RadialBar> {
    let Some(series) = builder.data.get(&builder.plot.series) else {
        return Vec::new();
    };
    let max = series.samples().iter().filter(|s| s.is_finite() && s.y > 0.0).map(|s| s.y).fold(0.0, f64::max);
    if max == 0.0 {
        return Vec::new();
    }
    let sum: f64 = series.samples().iter().filter(|s| s.is_finite() && s.y > 0.0).map(|s| s.y / max).sum();
    series
        .samples()
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, s)| s.is_finite() && s.y > 0.0)
        .map(|(index, sample)| {
            let fraction = if let Some(maximum) = builder.plot.maximum {
                (sample.y / maximum).min(1.0)
            } else if builder.plot.layout == RadialLayout::Stacked {
                (sample.y / max) / sum
            } else {
                sample.y / max
            };
            RadialBar {
                series: series.id().clone(),
                sample,
                sample_index: index,
                label: builder.names.get(index).cloned().unwrap_or_else(|| sample.x.to_string().into()),
                fraction,
            }
        })
        .collect()
}
fn arcs(builder: &RadialChartBuilder, style: &RadialPresentation, radius: Pixels) -> Vec<ArcBar> {
    let bars = observations(builder);
    let outer = radius * ratio(style.outer_radius, 1.0);
    let inner = outer * ratio(style.inner_radius, 0.3);
    let count = if builder.plot.layout == RadialLayout::Stacked {
        1
    } else {
        bars.len()
    };
    if count == 0 || outer <= inner {
        return Vec::new();
    }
    let space = outer - inner;
    let gap = positive(style.gap).min(space / count as f32);
    let thickness = ((space - gap * (count - 1) as f32) / count as f32).max(px(0.0));
    let start = builder.plot.start.to_radians();
    let sweep = builder.plot.sweep.to_radians();
    let mut used = 0.0;
    bars.into_iter()
        .enumerate()
        .map(|(index, info)| {
            let stacked = builder.plot.layout == RadialLayout::Stacked;
            let fraction = if stacked {
                info.fraction.min((1.0_f64 - used).max(0.0))
            } else {
                info.fraction
            };
            let bar_start = start
                + if stacked {
                    sweep * used * f64::from(builder.progress)
                } else {
                    0.0
                };
            used += fraction;
            let inner = inner
                + if stacked {
                    px(0.0)
                } else {
                    (thickness + gap) * index as f32
                };
            ArcBar {
                info,
                inner,
                outer: inner + thickness,
                start: bar_start,
                sweep: sweep * fraction * f64::from(builder.progress),
            }
        })
        .collect()
}
fn cap_radius(inner: Pixels, outer: Pixels, sweep: f64) -> Pixels {
    let angle = sweep.abs().min(TAU);
    let separation = angle.min(TAU - angle);
    ((outer - inner) / 2.0).min((inner + outer) / 2.0 * (separation / 2.0).sin() as f32)
}
// One closed outline paints body and caps together, so translucent fills blend once.
fn arc_path(
    center: Point<Pixels>,
    inner: Pixels,
    outer: Pixels,
    start: f64,
    sweep: f64,
    rounded: bool,
) -> Option<gpui::Path<Pixels>> {
    if outer <= inner || sweep.abs() <= f64::EPSILON {
        return None;
    }
    if !rounded || sweep.abs() >= TAU - 1e-6 {
        return super::pie::sector_path(center, outer, inner, start, sweep);
    }
    let mid = (inner + outer) / 2.0;
    let cap = cap_radius(inner, outer, sweep);
    let end = start + sweep;
    let mut path = gpui::PathBuilder::fill();
    path.move_to(polar(center, outer, start));
    super::pie::append_arc(&mut path, center, outer, start, sweep);
    let end_center = polar(center, mid, end);
    path.line_to(polar(end_center, cap, end));
    super::pie::append_arc(&mut path, end_center, cap, end, sweep.signum() * std::f64::consts::PI);
    path.line_to(polar(center, inner, end));
    super::pie::append_arc(&mut path, center, inner, end, -sweep);
    let start_center = polar(center, mid, start);
    path.line_to(polar(start_center, cap, start + std::f64::consts::PI));
    super::pie::append_arc(
        &mut path,
        start_center,
        cap,
        start + std::f64::consts::PI,
        sweep.signum() * std::f64::consts::PI,
    );
    path.close();
    path.build().ok()
}
fn paint_arc(
    center: Point<Pixels>,
    radii: (Pixels, Pixels),
    start: f64,
    sweep: f64,
    color: gpui::Background,
    rounded: bool,
    window: &mut Window,
) {
    if let Some(path) = arc_path(center, radii.0, radii.1, start, sweep, rounded) {
        window.paint_path(path, color);
    }
}
fn spoke_angles(start: f64, sweep: f64, count: usize) -> impl Iterator<Item = f64> {
    let count = count.min(360);
    (0..count).map(move |i| start + sweep * i as f64 / count as f64)
}
fn hit(bars: &[ArcBar], center: Point<Pixels>, pointer: Point<Pixels>, rounded: bool) -> Option<usize> {
    let delta = pointer - center;
    let x = f64::from(f32::from(delta.x));
    let y = f64::from(f32::from(delta.y));
    let distance = x.hypot(y);
    let angle = y.atan2(x);
    bars.iter()
        .enumerate()
        .rev()
        .find(|(_, bar)| {
            if bar.outer <= bar.inner || bar.sweep.abs() <= f64::EPSILON {
                return false;
            }
            let offset = if bar.sweep >= 0.0 {
                (angle - bar.start).rem_euclid(TAU)
            } else {
                (bar.start - angle).rem_euclid(TAU)
            };
            let band = distance >= f64::from(f32::from(bar.inner)) && distance <= f64::from(f32::from(bar.outer));
            if band && offset <= bar.sweep.abs() {
                return true;
            }
            if rounded {
                let mid = (bar.inner + bar.outer) / 2.0;
                let cap = cap_radius(bar.inner, bar.outer, bar.sweep);
                [bar.start, bar.start + bar.sweep].iter().any(|angle| {
                    let d = pointer - polar(center, mid, *angle);
                    f32::from(d.x).hypot(f32::from(d.y)) <= f32::from(cap)
                })
            } else {
                false
            }
        })
        .map(|(index, _)| index)
}
pub(crate) fn radial_canvas(builder: RadialChartBuilder) -> impl IntoElement {
    let style = (builder.style)();
    let prepare = builder.clone();
    let prepare_style = style.clone();
    let interactive = builder.tooltip.is_some() || builder.events.is_some();
    let events = builder.events.clone();
    let picked: std::rc::Rc<std::cell::RefCell<Option<crate::RadialBar>>> = Default::default();
    let prepared_pick = picked.clone();
    let canvas = canvas(
        move |bounds: Bounds<Pixels>, window, cx| {
            let center = bounds.center();
            let radius =
                (bounds.size.width.min(bounds.size.height) / 2.0 - positive(prepare_style.padding)).max(px(0.0));
            let bars = arcs(&prepare, &prepare_style, radius);
            let hovered = if bounds.contains(&window.mouse_position()) {
                hit(&bars, center, window.mouse_position(), prepare_style.rounded)
            } else {
                None
            };
            let target = hovered.map(|i| bars[i].info.clone());
            *prepared_pick.borrow_mut() = target.clone();
            if let Some(events) = &prepare.events {
                super::selection::report(target, events, window, cx);
            }
            let tooltip = prepare.tooltip.as_ref().and_then(|factory| {
                let target = hovered.map(|i| &bars[i]);
                let position = target
                    .map_or(center, |bar| polar(center, (bar.inner + bar.outer) / 2.0, bar.start + bar.sweep / 2.0));
                tooltip::tooltip(
                    target.map(|bar| &bar.info),
                    position,
                    bounds,
                    factory,
                    prepare.tooltip_motion,
                    window,
                    cx,
                )
            });
            (center, radius, bars, tooltip)
        },
        move |bounds, (center, radius, bars, mut tooltip), window, cx| {
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                if observations(&builder).is_empty()
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

                let track_start = builder.plot.start.to_radians();
                let track_sweep = builder.plot.sweep.to_radians();
                let tracks = if builder.plot.layout == RadialLayout::Stacked {
                    bars.get(..1).unwrap_or(&[])
                } else {
                    &bars
                };
                for bar in tracks {
                    if let Some(track) = style.track {
                        paint_arc(center, (bar.inner, bar.outer), track_start, track_sweep, track, false, window);
                    }
                    if let Some(stroke) = style.grid {
                        let r = (bar.inner + bar.outer) / 2.0;
                        let points: Vec<_> =
                            (0..=96).map(|i| polar(center, r, track_start + track_sweep * i as f64 / 96.0)).collect();
                        super::line::paint_path(points, stroke, window);
                    }
                }
                if let Some(stroke) = style.grid {
                    for angle in spoke_angles(track_start, track_sweep, style.grid_spokes) {
                        super::line::paint_path(
                            [
                                polar(
                                    center,
                                    radius * ratio(style.outer_radius, 1.0) * ratio(style.inner_radius, 0.3),
                                    angle,
                                ),
                                polar(
                                    center,
                                    radius * ratio(style.outer_radius, 1.0) + positive(style.padding) / 2.0,
                                    angle,
                                ),
                            ],
                            stroke,
                            window,
                        );
                    }
                }
                for bar in &bars {
                    paint_arc(
                        center,
                        (bar.inner, bar.outer),
                        bar.start,
                        bar.sweep,
                        style.fill_for(bar.info.sample_index),
                        style.rounded,
                        window,
                    );
                }
                if let Some(labels) = &builder.labels {
                    for bar in &bars {
                        let text = labels(&bar.info);
                        let mid = (bar.inner + bar.outer) / 2.0;
                        if mid <= px(0.0) || positive(style.label_text.line_height) > bar.outer - bar.inner {
                            continue;
                        }
                        let glyphs: Vec<_> = text
                            .chars()
                            .map(|ch| super::axis::shape(ch.to_string().into(), &style.label_text, window))
                            .collect();
                        let width: gpui::Pixels = glyphs.iter().map(|g| g.width()).sum();
                        if f64::from(f32::from(width)) + 8.0 > f64::from(f32::from(mid)) * bar.sweep.abs() {
                            continue;
                        }
                        let mut offset = 4.0;
                        for glyph in glyphs {
                            let w = f64::from(f32::from(glyph.width()));
                            let angle = bar.start + bar.sweep.signum() * (offset + w / 2.0) / f64::from(f32::from(mid));
                            let anchor = polar(center, mid, angle);
                            glyph
                                .paint(
                                    anchor - point(glyph.width() / 2.0, positive(style.label_text.line_height) / 2.0),
                                    positive(style.label_text.line_height),
                                    TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                )
                                .ok();
                            offset += w;
                        }
                    }
                }
                if let Some((title, caption)) = &builder.center {
                    for (text, text_style, offset) in [
                        (title, &style.center_title, -positive(style.center_caption.line_height) / 2.0),
                        (caption, &style.center_caption, positive(style.center_title.line_height) / 2.0),
                    ] {
                        let line = super::axis::shape(text.clone(), text_style, window);
                        line.paint(
                            center + point(-line.width() / 2.0, offset - positive(text_style.line_height) / 2.0),
                            positive(text_style.line_height),
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        )
                        .ok();
                    }
                }
            });
            if let Some(tooltip) = &mut tooltip {
                tooltip.paint(window, cx);
            }
        },
    )
    .size_full();
    gpui::div()
        .id("radial-canvas")
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
mod tooltip;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dataset, Series, Sample};
    fn builder() -> RadialChartBuilder {
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "A",
                [Sample::new(0.0, 100.0), Sample::new(1.0, 50.0), Sample::new(2.0, -1.0), Sample::new(f64::NAN, 25.0)],
            ))
            .unwrap();
        RadialChartBuilder::new(data, "a").bar_names(["First", "Second"])
    }
    #[test]
    fn clamped_spokes_cover_the_same_sweep_as_360_and_zero_is_empty() {
        for sweep in [-TAU, TAU, std::f64::consts::PI] {
            let expected: Vec<_> = spoke_angles(0.7, sweep, 360).collect();
            assert_eq!(spoke_angles(0.7, sweep, 720).collect::<Vec<_>>(), expected);
            assert_eq!(spoke_angles(0.7, sweep, usize::MAX).collect::<Vec<_>>(), expected);
            assert!((expected[359] - (0.7 + sweep * 359.0 / 360.0)).abs() < 1e-12);
        }
        assert_eq!(spoke_angles(0.0, TAU, 0).count(), 0);
        assert_eq!(spoke_angles(0.0, TAU, 12).count(), 12);
    }
    #[test]
    fn rounded_outline_covers_body_and_caps_once_in_both_directions() {
        fn contains(triangle: &[gpui::PathVertex<Pixels>], p: Point<Pixels>) -> bool {
            let cross =
                |a: Point<Pixels>, b: Point<Pixels>| f32::from(a.x) * f32::from(b.y) - f32::from(a.y) * f32::from(b.x);
            let a = triangle[0].xy_position - p;
            let b = triangle[1].xy_position - p;
            let c = triangle[2].xy_position - p;
            let edges = [cross(a, b), cross(b, c), cross(c, a)];
            edges.iter().all(|v| *v > 1e-4) || edges.iter().all(|v| *v < -1e-4)
        }
        for sign in [-1.0, 1.0] {
            let sweep = sign * std::f64::consts::FRAC_PI_2;
            let path = arc_path(Point::default(), px(60.0), px(100.0), 0.0, sweep, true).unwrap();
            for angle in [-sign * 0.08, sign * 0.12, sweep + sign * 0.08] {
                let p = polar(Point::default(), px(81.0), angle);
                assert_eq!(
                    path.vertices.as_chunks::<3>().0.iter().filter(|triangle| contains(*triangle, p)).count(),
                    1
                );
            }
            for sweep in [sign * 0.01, sign * (TAU - 0.01), sign * TAU] {
                let path = arc_path(Point::default(), px(60.0), px(100.0), 0.0, sweep, true).unwrap();
                assert!(
                    path.vertices
                        .iter()
                        .all(|v| f32::from(v.xy_position.x).is_finite() && f32::from(v.xy_position.y).is_finite())
                );
            }
        }
    }
    #[test]
    fn concentric_bands_preserve_source_and_exclude_gaps_tracks_and_unrevealed_arcs() {
        let b = builder();
        let style = RadialPresentation::default();
        let bars = arcs(&b, &style, px(100.0));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[1].info.sample_index, 1);
        assert_eq!(bars[0].info.fraction, 1.0);
        assert_eq!(bars[1].info.fraction, 0.5);
        assert!(bars[0].outer < bars[1].inner);
        let center = Point::default();
        assert_eq!(hit(&bars, center, polar(center, (bars[0].inner + bars[0].outer) / 2.0, -1.0), false), Some(0));
        assert_eq!(hit(&bars, center, polar(center, (bars[0].outer + bars[1].inner) / 2.0, -1.0), false), None);
        assert_eq!(hit(&bars, center, polar(center, (bars[1].inner + bars[1].outer) / 2.0, 1.0), false), None);
        assert_eq!(hit(&bars, center, center, false), None);
        let empty = arcs(&b.clone().draw_progress(0.0), &style, px(100.0));
        assert_eq!(hit(&empty, center, polar(center, px(50.0), -1.0), true), None);
        let partial = arcs(&b.draw_progress(0.5), &style, px(100.0));
        assert!((partial[0].sweep.abs() - TAU / 2.0).abs() < 1e-12);
    }
    #[test]
    fn stacks_share_one_band_and_overflow_safe_normalization_closes_semicircle() {
        let data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, f64::MAX), Sample::new(1.0, f64::MAX)]))
            .unwrap();
        let b = RadialChartBuilder::new(data, "a").layout(RadialLayout::Stacked).angles(180.0, 180.0);
        let bars = arcs(&b, &RadialPresentation::default(), px(100.0));
        assert_eq!(bars[0].info.fraction, 0.5);
        assert_eq!(bars[0].inner, bars[1].inner);
        assert_eq!(bars[0].outer, bars[1].outer);
        assert_eq!(bars[0].start + bars[0].sweep, bars[1].start);
        assert!((bars.iter().map(|bar| bar.sweep).sum::<f64>() - TAU / 2.0).abs() < 1e-12);
        let clipped =
            arcs(&builder().layout(RadialLayout::Stacked).maximum(50.0), &RadialPresentation::default(), px(100.0));
        assert_eq!(clipped[1].sweep, 0.0);
    }
    #[test]
    fn rounded_caps_are_pickable_but_degenerate_bands_are_not() {
        let bars = arcs(&builder().maximum(200.0), &RadialPresentation::default(), px(100.0));
        let bar = &bars[0];
        let center = Point::default();
        let mid = (bar.inner + bar.outer) / 2.0;
        let pointer = polar(center, mid, 0.03);
        assert_eq!(hit(&bars, center, pointer, false), None);
        assert_eq!(hit(&bars, center, pointer, true), Some(0));
        assert!(
            arcs(&builder(), &RadialPresentation { inner_radius: 1.0, ..Default::default() }, px(100.0)).is_empty()
        );
    }
}

#[cfg(test)]
mod tooltip_tests;
