use std::collections::HashMap;
use gpui::{AnyElement, App, AvailableSpace, Bounds, Pixels, Point, Window, fill, point, px, size};
use crate::{
    data::{Dataset, Sample, SeriesId},
    plots::{BarAlignment, BarGrouping, BarOrientation, CartesianPlot},
    presentation::BarPresentation,
    views::{BarRenderModel, BarTemplateSource},
};
use super::geometry::Range;

pub(super) struct BarSegment {
    pub sample: Sample,
    pub sample_index: usize,
    pub category_start: f64,
    pub category_end: f64,
    pub start: f64,
    pub end: f64,
}

/// Accumulate stacks before projecting so automatic scales include complete stacks.
pub(super) fn segments(data: &Dataset, plot: &CartesianPlot) -> Vec<Vec<BarSegment>> {
    segments_in(data, plot, None)
}
fn segments_in(data: &Dataset, plot: &CartesianPlot, viewport: Option<Range>) -> Vec<Vec<BarSegment>> {
    let layout = plot.bar_layout_spec();
    let count = if layout.grouping == BarGrouping::Stacked {
        1.0
    } else {
        plot.bars().len().max(1) as f64
    };
    let mut stacks: HashMap<(u64, u64), (f64, f64)> = HashMap::new();
    plot.bars()
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let Some(series) = data.get(layer.series()) else {
                return Vec::new();
            };
            if !plot.y_scale_spec().accepts(layer.baseline_value()) {
                return Vec::new();
            }
            let samples = series.samples();
            let (sample_start, sample_end) = if let Some(viewport) = viewport.filter(|_| series.sorted_x()) {
                (
                    samples.partition_point(|sample| sample.x < viewport.min - 0.5),
                    samples.partition_point(|sample| sample.x <= viewport.max + 0.5),
                )
            } else {
                (0, samples.len())
            };
            samples[sample_start..sample_end]
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, sample)| {
                    plot.accepts_sample(sample)
                        && viewport.is_none_or(|range| sample.x + 0.5 >= range.min && sample.x - 0.5 <= range.max)
                })
                .filter_map(|(sample_index, sample)| {
                    let mut start = layer.baseline_value();
                    let mut end = sample.y;
                    if layout.grouping == BarGrouping::Stacked {
                        let key = (bits(sample.x), bits(start));
                        let offsets = stacks.entry(key).or_insert((start, start));
                        let delta = sample.y - start;
                        let offset = if delta >= 0.0 { &mut offsets.0 } else { &mut offsets.1 };
                        start = *offset;
                        end = start + delta;
                        if !plot.y_scale_spec().accepts(end) {
                            return None;
                        }
                        *offset = end;
                    }
                    let slot_width = layout.width / count;
                    let inset = if count > 1.0 {
                        slot_width * layout.gap / 2.0
                    } else {
                        0.0
                    };
                    let group_start = sample.x - 0.5
                        + match layout.alignment {
                            BarAlignment::Start => 0.0,
                            BarAlignment::Center => (1.0 - layout.width) / 2.0,
                            BarAlignment::End => 1.0 - layout.width,
                        };
                    let slot_start = group_start
                        + if layout.grouping == BarGrouping::Stacked {
                            0.0
                        } else {
                            index as f64 * slot_width
                        };
                    Some(BarSegment {
                        sample,
                        sample_index: sample_index + sample_start,
                        start,
                        end,
                        category_start: slot_start + inset,
                        category_end: slot_start + slot_width - inset,
                    })
                })
                .collect()
        })
        .collect()
}
fn bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

pub(super) struct BarPlacement {
    pub baseline: Point<Pixels>,
    pub endpoint: Point<Pixels>,
    pub sample: Sample,
    pub sample_index: usize,
}
pub(super) struct BarGeometry {
    pub series: SeriesId,
    pub valid_points: usize,
    pub rectangles: Vec<Bounds<Pixels>>,
    pub placements: Vec<BarPlacement>,
}

pub(super) fn project(
    data: &Dataset,
    plot: &CartesianPlot,
    category: Range,
    value: Range,
    bounds: Bounds<Pixels>,
) -> Vec<BarGeometry> {
    let orientation = plot.bar_orientation_spec();
    segments_in(data, plot, Some(category))
        .into_iter()
        .zip(plot.bars())
        .filter_map(|(segments, layer)| {
            let series = data.get(layer.series())?;
            let valid_points = if matches!(plot.x_scale_spec(), crate::Scale::Numeric(_))
                && matches!(plot.y_scale_spec(), crate::Scale::Numeric(_))
            {
                series.finite_count()
            } else {
                series.samples().iter().filter(|sample| plot.accepts_sample(sample)).count()
            };
            let mut rectangles = Vec::new();
            let mut placements = Vec::new();
            for segment in segments {
                if segment.category_end < category.min || segment.category_start > category.max {
                    continue;
                }
                let c0 = super::geometry::normalized(category, segment.category_start, plot.x_scale_spec());
                let c1 = super::geometry::normalized(category, segment.category_end, plot.x_scale_spec());
                let v0 = super::geometry::normalized(value, segment.start, plot.y_scale_spec());
                let v1 = super::geometry::normalized(value, segment.end, plot.y_scale_spec());
                if ![c0, c1, v0, v1].into_iter().all(f32::is_finite) {
                    continue;
                }
                let position = |c: f32, v: f32| match orientation {
                    BarOrientation::Up => {
                        point(bounds.left() + bounds.size.width * c, bounds.bottom() - bounds.size.height * v)
                    }
                    BarOrientation::Down => {
                        point(bounds.left() + bounds.size.width * c, bounds.top() + bounds.size.height * v)
                    }
                    BarOrientation::Right => {
                        point(bounds.left() + bounds.size.width * v, bounds.top() + bounds.size.height * c)
                    }
                    BarOrientation::Left => {
                        point(bounds.right() - bounds.size.width * v, bounds.top() + bounds.size.height * c)
                    }
                };
                let a = position(c0, v0);
                let b = position(c1, v1);
                let rectangle =
                    Bounds::new(point(a.x.min(b.x), a.y.min(b.y)), size((a.x - b.x).abs(), (a.y - b.y).abs()));
                rectangles.push(rectangle);
                placements.push(BarPlacement {
                    baseline: position((c0 + c1) / 2.0, v0),
                    endpoint: position((c0 + c1) / 2.0, v1),
                    sample: segment.sample,
                    sample_index: segment.sample_index,
                });
            }
            Some(BarGeometry { series: layer.series().clone(), valid_points, rectangles, placements })
        })
        .collect()
}

pub(super) fn prepare_templates(
    geometry: &[BarGeometry],
    presentation: &crate::CartesianPresentation,
    orientation: BarOrientation,
    template: &BarTemplateSource,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement> {
    let mut elements = Vec::new();
    for bar in geometry {
        for (bounds, placement) in bar.rectangles.iter().zip(&bar.placements) {
            if bounds.size.width <= px(0.0) || bounds.size.height <= px(0.0) {
                continue;
            }
            let model = BarRenderModel {
                bounds: *bounds,
                baseline: placement.baseline,
                endpoint: placement.endpoint,
                orientation,
                presentation: presentation.bar_for(&bar.series),
                series: bar.series.clone(),
                sample: placement.sample,
                sample_index: placement.sample_index,
            };
            let mut element = template.0.render(&model, window, cx);
            element.prepaint_as_root(bounds.origin, bounds.size.map(AvailableSpace::Definite), window, cx);
            elements.push(element);
        }
    }
    elements
}

pub(super) fn paint_bar(geometry: &BarGeometry, style: BarPresentation, window: &mut Window) {
    for bounds in &geometry.rectangles {
        window.paint_quad(fill(*bounds, style.fill));
    }
}

#[cfg(test)]
mod tests;

fn model(
    bar: &BarGeometry,
    bounds: Bounds<Pixels>,
    placement: &BarPlacement,
    style: &crate::CartesianPresentation,
    orientation: BarOrientation,
) -> BarRenderModel {
    BarRenderModel {
        bounds,
        baseline: placement.baseline,
        endpoint: placement.endpoint,
        orientation,
        presentation: style.bar_for(&bar.series),
        series: bar.series.clone(),
        sample: placement.sample,
        sample_index: placement.sample_index,
    }
}

pub(super) struct LabelLayout {
    line: gpui::ShapedLine,
    origin: Point<Pixels>,
    height: Pixels,
}

pub(super) fn label_origin(
    model: &BarRenderModel,
    label: &crate::BarLabel,
    text_size: gpui::Size<Pixels>,
) -> Point<Pixels> {
    use crate::BarLabelPlacement;
    let horizontal = model.orientation.is_horizontal();
    let delta = model.endpoint - model.baseline;
    let forward = if horizontal {
        delta.x >= px(0.0)
    } else {
        delta.y >= px(0.0)
    };
    let sign = if forward { 1.0 } else { -1.0 };
    let gap = super::axis::positive(label.gap);
    let half = if horizontal {
        text_size.width / 2.0
    } else {
        text_size.height / 2.0
    };
    let offset = |value| {
        if horizontal {
            point(value, px(0.0))
        } else {
            point(px(0.0), value)
        }
    };
    let center = match label.placement {
        BarLabelPlacement::InsideStart => model.baseline + offset((gap + half) * sign),
        BarLabelPlacement::InsideEnd => model.endpoint - offset((gap + half) * sign),
        BarLabelPlacement::OutsideEnd => model.endpoint + offset((gap + half) * sign),
        BarLabelPlacement::Center => model.bounds.center(),
    };
    center - point(text_size.width / 2.0, text_size.height / 2.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_labels(
    geometry: &[BarGeometry],
    presentation: &crate::CartesianPresentation,
    orientation: BarOrientation,
    labels: &crate::views::BarLabelSource,
    visible: Bounds<Pixels>,
    canvas_bounds: Bounds<Pixels>,
    window: &mut Window,
) -> Vec<LabelLayout> {
    let mut result = Vec::new();
    for bar in geometry {
        for (bounds, placement) in bar.rectangles.iter().zip(&bar.placements) {
            if bounds.size.width <= px(0.0)
                || bounds.size.height <= px(0.0)
                || visible.size.width <= px(0.0)
                || placement.endpoint.x < visible.left()
                || placement.endpoint.x > visible.right()
                || placement.endpoint.y < visible.top()
                || placement.endpoint.y > visible.bottom()
            {
                continue;
            }
            let model = model(bar, *bounds, placement, presentation, orientation);
            for label in (labels.0)(&model) {
                if label.text.is_empty() {
                    continue;
                }
                let line = super::axis::shape(label.text.clone(), &label.text_style, window);
                let height = super::axis::positive(label.text_style.line_height);
                let size = size(line.width(), height);
                let mut origin = label_origin(&model, &label, size);
                if label.placement != crate::BarLabelPlacement::OutsideEnd {
                    if origin.x < bounds.left()
                        || origin.x + size.width > bounds.right()
                        || origin.y < bounds.top()
                        || origin.y + size.height > bounds.bottom()
                    {
                        continue;
                    }
                } else {
                    if size.width > canvas_bounds.size.width || height > canvas_bounds.size.height {
                        continue;
                    }
                    origin.x = origin.x.clamp(canvas_bounds.left(), canvas_bounds.right() - size.width);
                    origin.y = origin.y.clamp(canvas_bounds.top(), canvas_bounds.bottom() - height);
                }
                result.push(LabelLayout { line, origin, height });
            }
        }
    }
    result
}

pub(super) fn paint_labels(labels: &[LabelLayout], window: &mut Window, cx: &mut App) {
    for label in labels {
        label.line.paint(label.origin, label.height, gpui::TextAlign::Left, None, window, cx).ok();
    }
}

/// Hit the visible layout rectangle, independent of the shape's painting.
pub(super) fn select(
    geometry: &super::geometry::CartesianGeometry,
    data: &Dataset,
    style: &crate::CartesianPresentation,
    pointer: Point<Pixels>,
    visible: Bounds<Pixels>,
) -> Vec<super::points::HoverPoint> {
    if !visible.contains(&pointer) {
        return Vec::new();
    }
    for (index, bar) in geometry.bars.iter().enumerate().rev() {
        for (bounds, placement) in bar.rectangles.iter().zip(&bar.placements).rev() {
            if bounds.contains(&pointer) && bounds.size.width > px(0.0) && bounds.size.height > px(0.0) {
                let Some(series) = data.get(&bar.series) else {
                    continue;
                };
                // Anchor to the visible center, so clipped/offscreen endpoints cannot pull the tooltip away.
                let clipped = bounds.intersect(&visible);
                return vec![super::points::HoverPoint {
                    bar: true,
                    area: false,
                    layer: index,
                    position: clipped.center(),
                    info: crate::PlotPoint {
                        series: bar.series.clone(),
                        label: series.label().to_owned(),
                        sample: placement.sample,
                        color: style.line_for(&bar.series).stroke.color,
                    },
                }];
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tooltip_tests;
