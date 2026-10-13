use std::f64::consts::{TAU, FRAC_PI_2};
use gpui::{
    Bounds, IntoElement, PathBuilder, Pixels, Point, ShapedLine, SharedString, TextAlign, Window, canvas, point,
    prelude::*, px,
};
use crate::{Dataset, PiePlot, PiePresentation, PieLabelPlacement, PieSlice, TextPresentation, Stroke};
use crate::views::pie::PieLabels;

/// Normalize by the largest weight before summing to avoid overflow at f64::MAX.
pub(crate) fn slices(data: &Dataset, plot: &PiePlot, names: &[SharedString]) -> Vec<PieSlice> {
    let Some(series) = data.get(plot.series()) else {
        return Vec::new();
    };
    let accepted = |sample: &&crate::Sample| sample.is_finite() && sample.y > 0.0;
    let maximum = series.samples().iter().filter(accepted).map(|sample| sample.y).fold(0.0, f64::max);
    if maximum == 0.0 {
        return Vec::new();
    }
    let sum: f64 = series.samples().iter().filter(accepted).map(|sample| sample.y / maximum).sum();
    series
        .samples()
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, sample)| sample.is_finite() && sample.y > 0.0)
        .map(|(index, sample)| PieSlice {
            ring_index: None,
            series: series.id().clone(),
            sample,
            sample_index: index,
            label: names.get(index).cloned().unwrap_or_else(|| sample.x.to_string().into()),
            fraction: (sample.y / maximum) / sum,
        })
        .filter(|slice| slice.fraction > 0.0)
        .collect()
}
struct Sector {
    index: usize,
    start: f64,
    sweep: f64,
    radius: Pixels,
    gap: Option<(Pixels, Pixels)>,
}
struct TextLayout {
    line: ShapedLine,
    origin: Point<Pixels>,
    height: Pixels,
    leader: Option<[Point<Pixels>; 2]>,
}
struct PieLayout {
    center: Point<Pixels>,
    inner: Pixels,
    sectors: Vec<Sector>,
    text: Vec<TextLayout>,
}
fn positive(value: Pixels) -> Pixels {
    super::axis::positive(value)
}
fn ratio(value: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 0.95) } else { 0.0 }
}
fn polar(center: Point<Pixels>, radius: Pixels, angle: f64) -> Point<Pixels> {
    center + point(radius * angle.cos() as f32, radius * angle.sin() as f32)
}
fn sector_angles(slices: &[PieSlice], plot: &PiePlot) -> Vec<(f64, f64)> {
    let direction = if plot.is_clockwise() { 1.0 } else { -1.0 };
    let start = plot.start_angle_degrees().to_radians();
    let mut cursor = start;
    slices
        .iter()
        .enumerate()
        .map(|(index, slice)| {
            // Close the final slice at the exact full-circle endpoint.
            let end = if index + 1 == slices.len() {
                start + TAU * direction
            } else {
                cursor + TAU * slice.fraction * direction
            };
            let result = (cursor, end - cursor);
            cursor = end;
            result
        })
        .collect()
}
fn text_layout(text: SharedString, style: &TextPresentation, center: Point<Pixels>, window: &mut Window) -> TextLayout {
    let line = super::axis::shape(text, style, window);
    TextLayout {
        origin: center - point(line.width() / 2.0, positive(style.line_height) / 2.0),
        line,
        height: positive(style.line_height),
        leader: None,
    }
}
struct PieContent {
    labels: Option<PieLabels>,
    active: Option<usize>,
    center: Option<(SharedString, SharedString)>,
    expansion: Option<std::collections::HashMap<usize, f32>>,
    progress: f32,
}
fn layout(
    bounds: Bounds<Pixels>,
    slices: &[PieSlice],
    plot: &PiePlot,
    style: &PiePresentation,
    content: &PieContent,
    window: &mut Window,
) -> PieLayout {
    let center = bounds.center();
    let labels = content.labels.as_ref();
    let active = content.active;
    let center_text = content.center.as_ref();
    let shaped: Vec<_> = labels.map_or_else(Vec::new, |labels| {
        slices
            .iter()
            .map(|slice| {
                let text = (labels.0)(slice);
                if text.is_empty() {
                    None
                } else {
                    Some(super::axis::shape(text, &style.label_text, window))
                }
            })
            .collect()
    });
    let outside = labels.is_some() && style.label_placement == PieLabelPlacement::Outside;
    let label_width = if outside {
        shaped.iter().flatten().map(ShapedLine::width).max().unwrap_or(px(0.0))
    } else {
        px(0.0)
    };
    let gap = positive(style.label_gap);
    let pad = positive(style.padding) + positive(style.active_expansion);
    let horizontal = if outside { label_width + gap } else { px(0.0) };
    let vertical = if outside {
        positive(style.label_text.line_height) / 2.0 + gap
    } else {
        px(0.0)
    };
    let radius = ((bounds.size.width - (pad + horizontal) * 2.0).min(bounds.size.height - (pad + vertical) * 2.0)
        / 2.0)
        .max(px(0.0));
    let outer_ratio = if style.outer_radius.is_finite() {
        style.outer_radius.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let radius = radius * outer_ratio;
    let inner = radius * ratio(style.inner_radius);
    let angles = sector_angles(slices, plot);
    let sectors: Vec<_> = slices
        .iter()
        .zip(&angles)
        .map(|(slice, (start, sweep))| {
            let weight = content
                .expansion
                .as_ref()
                .and_then(|weights| weights.get(&slice.sample_index))
                .copied()
                .unwrap_or(if active == Some(slice.sample_index) { 1.0 } else { 0.0 });
            let expansion = if radius > px(0.0) {
                positive(style.active_expansion) * weight
            } else {
                px(0.0)
            };
            let outer = radius + expansion;
            let gap = style.active_band_gap.filter(|_| expansion > px(0.0)).map(|gap| {
                let start = radius + expansion / 2.0;
                (start, (start + positive(gap) * weight).min(outer))
            });
            Sector {
                index: slice.sample_index,
                start: *start,
                sweep: {
                    let elapsed = (*start - plot.start_angle_degrees().to_radians()).abs();
                    let remaining = (TAU * f64::from(content.progress) - elapsed).max(0.0);
                    sweep.signum() * sweep.abs().min(remaining)
                },
                radius: outer,
                gap,
            }
        })
        .collect();
    let mut text = Vec::new();
    for (sector, line) in sectors.iter().zip(shaped) {
        let Some(line) = line else {
            continue;
        };
        if radius <= px(0.0) || sector.sweep.abs() <= f64::EPSILON {
            continue;
        }
        let angle = sector.start + sector.sweep / 2.0;
        let anchor = polar(
            center,
            if outside {
                sector.radius + gap
            } else {
                (inner + sector.radius) / 2.0
            },
            angle,
        );
        let origin = if outside {
            point(
                if angle.cos() >= 0.0 {
                    anchor.x
                } else {
                    anchor.x - line.width()
                },
                anchor.y - positive(style.label_text.line_height) / 2.0,
            )
        } else {
            anchor - point(line.width() / 2.0, positive(style.label_text.line_height) / 2.0)
        };
        // Omit labels that cannot fit in the canvas; do not let tiny layouts leak text.
        if origin.x < bounds.left()
            || origin.x + line.width() > bounds.right()
            || origin.y < bounds.top()
            || origin.y + positive(style.label_text.line_height) > bounds.bottom()
        {
            continue;
        }
        text.push(TextLayout {
            line,
            origin,
            height: positive(style.label_text.line_height),
            leader: (outside && style.label_lines).then(|| [polar(center, sector.radius, angle), anchor]),
        });
    }
    if slices.is_empty() {
        if let Some(empty) = &style.empty_state {
            let mut text_style = style.label_text.clone();
            text_style.color = empty.text_color;
            text.push(text_layout(empty.message.clone(), &text_style, center, window));
        }
    } else if let Some((title, caption)) = center_text {
        let title_height = if title.is_empty() {
            px(0.0)
        } else {
            positive(style.center_title.line_height)
        };
        let caption_height = if caption.is_empty() {
            px(0.0)
        } else {
            positive(style.center_caption.line_height)
        };
        if !title.is_empty() {
            text.push(text_layout(
                title.clone(),
                &style.center_title,
                center - point(px(0.0), caption_height / 2.0),
                window,
            ));
        }
        if !caption.is_empty() {
            text.push(text_layout(
                caption.clone(),
                &style.center_caption,
                center + point(px(0.0), title_height / 2.0),
                window,
            ));
        }
    }
    PieLayout { center, inner, sectors, text }
}

pub(super) fn append_arc(path: &mut PathBuilder, center: Point<Pixels>, radius: Pixels, start: f64, sweep: f64) {
    let steps = (sweep.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = sweep / steps as f64;
    for index in 0..steps {
        let a = start + index as f64 * step;
        let b = a + step;
        let k = (4.0 / 3.0 * (step / 4.0).tan()) as f32;
        let p0 = polar(center, radius, a);
        let p1 = polar(center, radius, b);
        let c0 = p0 + point(-radius * k * a.sin() as f32, radius * k * a.cos() as f32);
        let c1 = p1 + point(radius * k * b.sin() as f32, -radius * k * b.cos() as f32);
        path.cubic_bezier_to(p1, c0, c1);
    }
}
pub(super) fn sector_path(
    center: Point<Pixels>,
    outer: Pixels,
    inner: Pixels,
    start: f64,
    sweep: f64,
) -> Option<gpui::Path<Pixels>> {
    if outer <= px(0.0) || sweep.abs() <= f64::EPSILON {
        return None;
    }
    let mut path = PathBuilder::fill();
    path.move_to(polar(center, outer, start));
    append_arc(&mut path, center, outer, start, sweep);
    if inner > px(0.0) {
        path.line_to(polar(center, inner, start + sweep));
        append_arc(&mut path, center, inner, start + sweep, -sweep);
    } else {
        path.line_to(center);
    }
    path.close();
    path.build().ok()
}
pub(crate) fn pie_canvas(
    slices: Vec<PieSlice>,
    plot: PiePlot,
    style: PiePresentation,
    labels: Option<PieLabels>,
    active: Option<usize>,
    center: Option<(SharedString, SharedString)>,
    options: crate::views::pie::PieInteraction,
) -> impl IntoElement {
    let prepare_style = style.clone();
    let interactive = options.tooltip.is_some() || options.events.is_some();
    let events = options.events.clone();
    let click_events = events.clone();
    let pick: std::rc::Rc<std::cell::RefCell<Option<PieSlice>>> = Default::default();
    let painted_pick = pick.clone();
    let mut content = PieContent { labels, active, center, expansion: None, progress: options.draw_progress };
    let canvas = canvas(
        move |bounds, window, cx| {
            content.expansion =
                Some(interaction::expansions(&slices, active, bounds, options.active_motion, window, cx));
            let layout = layout(bounds, &slices, &plot, &prepare_style, &content, window);
            let hovered = hit(&layout, window.mouse_position())
                .and_then(|index| slices.iter().find(|slice| slice.sample_index == index))
                .cloned();
            *painted_pick.borrow_mut() = hovered.clone();
            if let Some(events) = &events {
                interaction::report_hover(hovered.clone(), events, window, cx);
            }
            let tooltip = options.tooltip.as_ref().and_then(|factory| {
                interaction::tooltip(hovered.as_ref(), &layout, bounds, factory, options.tooltip_motion, window, cx)
            });
            (layout, tooltip)
        },
        move |bounds, (layout, mut tooltip), window, cx| {
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                for sector in &layout.sectors {
                    if let Some(path) = sector_path(
                        layout.center,
                        sector.gap.map_or(sector.radius, |gap| gap.0),
                        layout.inner,
                        sector.start,
                        sector.sweep,
                    ) {
                        window.paint_path(path, style.fill_for(sector.index));
                    }
                    if let Some((_, inner)) = sector.gap
                        && inner < sector.radius
                        && let Some(path) = sector_path(layout.center, sector.radius, inner, sector.start, sector.sweep)
                    {
                        window.paint_path(path, style.fill_for(sector.index));
                    }
                }
                for text in &layout.text {
                    if let Some(leader) = text.leader {
                        super::line::paint_path(
                            leader,
                            Stroke { color: style.label_text.color.opacity(0.6), width: px(1.0) },
                            window,
                        );
                    }
                    text.line.paint(text.origin, text.height, TextAlign::Left, None, window, cx).ok();
                }
            });
            if let Some(tooltip) = &mut tooltip {
                tooltip.paint(window, cx);
            }
        },
    )
    .size_full();
    gpui::div()
        .id("pie-canvas")
        .size_full()
        .child(canvas)
        .when(interactive, |root| {
            root.on_mouse_move(|_, window, _| window.refresh()).on_hover(|_, window, _| window.refresh())
        })
        .when_some(click_events, |root, events| {
            root.on_click(move |_, _, cx| {
                if let Some(slice) = pick.borrow().clone() {
                    (events.0)(crate::PieSliceEvent::Select { slice }, cx);
                }
            })
        })
}

fn hit(layout: &PieLayout, pointer: Point<Pixels>) -> Option<usize> {
    let delta = pointer - layout.center;
    let x = f64::from(f32::from(delta.x));
    let y = f64::from(f32::from(delta.y));
    let distance = x.hypot(y);
    if distance < f64::from(f32::from(layout.inner)) || distance <= f64::EPSILON {
        return None;
    }
    let angle = y.atan2(x);
    layout
        .sectors
        .iter()
        .rev()
        .find(|sector| {
            let offset = if sector.sweep >= 0.0 {
                (angle - sector.start).rem_euclid(TAU)
            } else {
                (sector.start - angle).rem_euclid(TAU)
            };
            sector.sweep.abs() > f64::EPSILON
                && offset <= sector.sweep.abs()
                && distance <= f64::from(f32::from(sector.radius))
                && sector.gap.is_none_or(|(start, end)| {
                    distance <= f64::from(f32::from(start)) || distance >= f64::from(f32::from(end))
                })
        })
        .map(|sector| sector.index)
}

mod interaction;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ring_event_tests;
