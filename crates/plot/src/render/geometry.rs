use gpui::{Bounds, Edges, Pixels, Point, point, px, size};

use crate::{
    data::{Dataset, Sample, SeriesId},
    plots::{CartesianPlot, Interpolation, LineLayer, Scale},
};

#[derive(Clone, Copy)]
pub(super) struct Range {
    pub(super) min: f64,
    pub(super) max: f64,
}

impl Range {
    pub(super) fn normalized(self, value: f64) -> f32 {
        if self.min == self.max {
            return 0.5;
        }
        let span = self.max - self.min;
        if span.is_finite() {
            ((value - self.min) / span) as f32
        } else {
            ((value / 2.0 - self.min / 2.0) / (self.max / 2.0 - self.min / 2.0)) as f32
        }
    }
}

pub(super) fn normalized(range: Range, value: f64, scale: &Scale) -> f32 {
    if matches!(scale,Scale::Numeric(s) if s.log_base().is_some()) {
        Range { min: range.min.ln(), max: range.max.ln() }.normalized(value.ln())
    } else {
        range.normalized(value)
    }
}
pub(crate) struct LineGeometry {
    pub(crate) series: SeriesId,
    pub(crate) interpolation: Interpolation,
    pub(crate) runs: Vec<Vec<Point<Pixels>>>,
    pub(crate) samples: Vec<Vec<Sample>>,
    pub(crate) valid_points: usize,
    pub(crate) visible_points: usize,
}

pub(crate) struct AreaGeometry {
    pub(crate) boundary: LineGeometry,
    pub(crate) baseline: Pixels,
}

pub(crate) struct CartesianGeometry {
    pub(crate) plot_bounds: Bounds<Pixels>,
    y_domain: (f64, f64),
    x_scale: Scale,
    y_scale: Scale,
    orientation: crate::BarOrientation,
    pub(crate) x_domain: (f64, f64),
    pub(crate) x_ticks: Vec<TickGeometry>,
    pub(crate) y_ticks: Vec<TickGeometry>,
    pub(crate) lines: Vec<LineGeometry>,
    pub(crate) areas: Vec<AreaGeometry>,
    pub(crate) bars: Vec<super::bar::BarGeometry>,
}

impl CartesianGeometry {
    pub(super) fn data_position(&self, sample: Sample) -> Option<Point<Pixels>> {
        if !self.x_scale.accepts(sample.x) || !self.y_scale.accepts(sample.y) {
            return None;
        }
        let c = normalized(Range { min: self.x_domain.0, max: self.x_domain.1 }, sample.x, &self.x_scale);
        let v = normalized(Range { min: self.y_domain.0, max: self.y_domain.1 }, sample.y, &self.y_scale);
        if !c.is_finite() || !v.is_finite() {
            return None;
        }
        let b = self.plot_bounds;
        Some(match self.orientation {
            crate::BarOrientation::Up => point(b.left() + b.size.width * c, b.bottom() - b.size.height * v),
            crate::BarOrientation::Down => point(b.left() + b.size.width * c, b.top() + b.size.height * v),
            crate::BarOrientation::Right => point(b.left() + b.size.width * v, b.top() + b.size.height * c),
            crate::BarOrientation::Left => point(b.right() - b.size.width * v, b.top() + b.size.height * c),
        })
    }
    pub(super) fn x_position(&self, value: f64) -> Pixels {
        self.plot_bounds.left()
            + self.plot_bounds.size.width
                * normalized(Range { min: self.x_domain.0, max: self.x_domain.1 }, value, &self.x_scale)
    }
}

pub(crate) struct TickGeometry {
    pub(crate) position: Pixels,
    pub(crate) label: String,
}

/// Tick text doesn't depend on screen bounds or axis placement.
pub(super) fn scale_ticks(data: &Dataset, plot: &CartesianPlot) -> (Vec<TickGeometry>, Vec<TickGeometry>) {
    let (x, y) = scale_ranges(data, plot);
    screen_ticks(Bounds::new(point(px(0.0), px(0.0)), size(px(1.0), px(1.0))), x, y, plot)
}

pub(crate) fn layout(
    data: &Dataset,
    plot: &CartesianPlot,
    insets: Edges<Pixels>,
    bounds: Bounds<Pixels>,
) -> CartesianGeometry {
    let (left, right) = fit_insets(insets.left, insets.right, bounds.size.width);
    let (top, bottom) = fit_insets(insets.top, insets.bottom, bounds.size.height);
    let plot_bounds = Bounds::new(
        bounds.origin + point(left, top),
        size(bounds.size.width - left - right, bounds.size.height - bottom - top),
    );
    let (x, y) = scale_ranges(data, plot);
    let lines = plot
        .lines()
        .iter()
        .filter_map(|layer| data.get(layer.series()).map(|series| (layer, series)))
        .map(|(layer, series)| {
            let (runs, samples, valid_points, visible_points) =
                line_runs(series.samples(), x, y, plot_bounds, plot.x_scale_spec(), plot.y_scale_spec(), layer);
            LineGeometry {
                series: series.id().clone(),
                interpolation: layer.interpolation_policy(),
                valid_points,
                visible_points,
                runs,
                samples,
            }
        })
        .collect();
    let areas = plot
        .areas()
        .iter()
        .filter(|layer| plot.y_scale_spec().accepts(layer.baseline_value()))
        .filter_map(|layer| data.get(layer.series()).map(|series| (layer, series)))
        .map(|(layer, series)| {
            let (runs, samples, valid_points, visible_points) = line_runs(
                series.samples(),
                x,
                y,
                plot_bounds,
                plot.x_scale_spec(),
                plot.y_scale_spec(),
                layer.boundary(),
            );
            AreaGeometry {
                boundary: LineGeometry {
                    series: series.id().clone(),
                    interpolation: layer.interpolation_policy(),
                    runs,
                    samples,
                    valid_points,
                    visible_points,
                },
                baseline: plot_bounds.bottom()
                    - plot_bounds.size.height * normalized(y, layer.baseline_value(), plot.y_scale_spec()),
            }
        })
        .collect();
    let bars = super::bar::project(data, plot, x, y, plot_bounds);
    let (x_ticks, y_ticks) = screen_ticks(plot_bounds, x, y, plot);
    CartesianGeometry {
        y_domain: (y.min, y.max),
        x_scale: plot.x_scale_spec().clone(),
        y_scale: plot.y_scale_spec().clone(),
        orientation: if plot.bars().is_empty() {
            crate::BarOrientation::Up
        } else {
            plot.bar_orientation_spec()
        },
        bars,
        plot_bounds,
        x_domain: (x.min, x.max),
        x_ticks,
        y_ticks,
        lines,
        areas,
    }
}

fn screen_ticks(
    bounds: Bounds<Pixels>,
    category: Range,
    value: Range,
    plot: &CartesianPlot,
) -> (Vec<TickGeometry>, Vec<TickGeometry>) {
    use crate::BarOrientation;
    let orientation = if plot.bars().is_empty() {
        BarOrientation::Up
    } else {
        plot.bar_orientation_spec()
    };
    match orientation {
        BarOrientation::Up | BarOrientation::Down => (
            scale_ticks_at(bounds.left(), bounds.size.width, category, plot.x_scale_spec()),
            scale_ticks_at(
                if orientation == BarOrientation::Up {
                    bounds.bottom()
                } else {
                    bounds.top()
                },
                if orientation == BarOrientation::Up {
                    -bounds.size.height
                } else {
                    bounds.size.height
                },
                value,
                plot.y_scale_spec(),
            ),
        ),
        BarOrientation::Left | BarOrientation::Right => (
            scale_ticks_at(
                if orientation == BarOrientation::Right {
                    bounds.left()
                } else {
                    bounds.right()
                },
                if orientation == BarOrientation::Right {
                    bounds.size.width
                } else {
                    -bounds.size.width
                },
                value,
                plot.y_scale_spec(),
            ),
            scale_ticks_at(bounds.top(), bounds.size.height, category, plot.x_scale_spec()),
        ),
    }
}

fn fit_insets(start: Pixels, end: Pixels, available: Pixels) -> (Pixels, Pixels) {
    let start = positive(start);
    let end = positive(end);
    let total = start + end;
    if total > available && total > px(0.0) {
        let factor = f32::from(available.max(px(0.0))) / f32::from(total);
        (start * factor, end * factor)
    } else {
        (start, end)
    }
}

fn positive(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() && value > px(0.0) {
        value
    } else {
        px(0.0)
    }
}

fn scale_ranges(data: &Dataset, plot: &CartesianPlot) -> (Range, Range) {
    if let (Scale::Numeric(x), Scale::Numeric(y)) = (plot.x_scale_spec(), plot.y_scale_spec())
        && let (Some(x), Some(y)) = (x.domain_bounds(), y.domain_bounds())
    {
        return (Range { min: *x.start(), max: *x.end() }, Range { min: *y.start(), max: *y.end() });
    }
    let samples = plot
        .series_ids()
        .filter_map(|id| data.get(id))
        .flat_map(|series| series.samples())
        .filter(|sample| plot.accepts_sample(sample));
    let (mut x, mut y) = ranges(samples);
    for layer in plot.areas() {
        if plot.y_scale_spec().accepts(layer.baseline_value())
            && data
                .get(layer.series())
                .is_some_and(|series| series.samples().iter().any(|sample| plot.accepts_sample(sample)))
        {
            y.min = y.min.min(layer.baseline_value());
            y.max = y.max.max(layer.baseline_value());
        }
    }
    for segment in super::bar::segments(data, plot).into_iter().flatten() {
        x.min = x.min.min(segment.sample.x - 0.5);
        x.max = x.max.max(segment.sample.x + 0.5);
        y.min = y.min.min(segment.start).min(segment.end);
        y.max = y.max.max(segment.start).max(segment.end);
    }
    let mut x = scale_range(plot.x_scale_spec(), x);
    if !plot.bars().is_empty()
        && let Scale::Category(scale) = plot.x_scale_spec()
    {
        x = Range { min: -0.5, max: scale.labels().len().max(1) as f64 - 0.5 };
    }
    (x, scale_range(plot.y_scale_spec(), y))
}

fn scale_range(scale: &Scale, automatic: Range) -> Range {
    match scale {
        Scale::Numeric(scale) => scale.domain_bounds().map_or_else(
            || {
                if scale.log_base().is_some() && automatic.min <= 0.0 {
                    Range { min: 1.0, max: scale.log_base().unwrap_or(10.0) }
                } else {
                    automatic
                }
            },
            |domain| Range { min: *domain.start(), max: *domain.end() },
        ),
        Scale::Category(scale) => Range { min: 0.0, max: scale.labels().len().saturating_sub(1) as f64 },
    }
}

fn scale_ticks_at(origin: Pixels, length: Pixels, range: Range, scale: &Scale) -> Vec<TickGeometry> {
    match scale {
        Scale::Numeric(scale) => match scale.explicit_ticks() {
            Some(values) => values
                .iter()
                .copied()
                .filter(|value| *value >= range.min && *value <= range.max)
                .map(|value| TickGeometry {
                    position: origin + length * normalized(range, value, &Scale::Numeric(scale.clone())),
                    label: scale.label_for(value).map_or_else(|| value.to_string(), str::to_owned),
                })
                .collect(),
            None => {
                if let Some(base) = scale.log_base() {
                    let lo = range.min.log(base);
                    let hi = range.max.log(base);
                    let step = ((hi.floor() - lo.ceil()) / (scale.ticks().max(2) - 1) as f64).ceil().max(1.0);
                    let mut values = Vec::new();
                    let mut power = lo.ceil();
                    while power <= hi.floor() && values.len() < scale.ticks() {
                        let v = base.powf(power);
                        if v.is_finite() && v >= range.min && v <= range.max {
                            values.push(v);
                        }
                        power += step;
                    }
                    if values.is_empty() {
                        values.push(range.min);
                        if range.max > range.min {
                            values.push(range.max);
                        }
                    }
                    values
                        .into_iter()
                        .map(|v| TickGeometry {
                            position: origin + length * normalized(range, v, &Scale::Numeric(scale.clone())),
                            label: v.to_string(),
                        })
                        .collect()
                } else {
                    ticks(origin, length, range, scale.ticks())
                }
            }
        },
        Scale::Category(scale) => scale
            .labels()
            .iter()
            .enumerate()
            .map(|(index, label)| TickGeometry {
                position: origin + length * range.normalized(index as f64),
                label: label.to_string(),
            })
            .collect(),
    }
}

fn ranges<'a>(samples: impl Iterator<Item = &'a Sample>) -> (Range, Range) {
    let mut finite = samples.copied().filter(|sample| sample.is_finite());
    let Some(first) = finite.next() else {
        return (Range { min: 0.0, max: 1.0 }, Range { min: 0.0, max: 1.0 });
    };
    let mut x = Range { min: first.x, max: first.x };
    let mut y = Range { min: first.y, max: first.y };
    for sample in finite {
        x.min = x.min.min(sample.x);
        x.max = x.max.max(sample.x);
        y.min = y.min.min(sample.y);
        y.max = y.max.max(sample.y);
    }
    (x, y)
}

fn ticks(origin: Pixels, length: Pixels, range: Range, count: usize) -> Vec<TickGeometry> {
    if range.min == range.max {
        return vec![TickGeometry { position: origin + length / 2.0, label: range.min.to_string() }];
    }
    let step = (range.max - range.min) / (count - 1) as f64;
    let precision = tick_precision(step);
    (0..count)
        .map(|index| {
            let fraction = index as f32 / (count - 1) as f32;
            TickGeometry {
                position: origin + length * fraction,
                label: format_tick(interpolate(range, f64::from(fraction)), precision),
            }
        })
        .collect()
}

fn interpolate(range: Range, fraction: f64) -> f64 {
    let span = range.max - range.min;
    if span.is_finite() {
        range.min + span * fraction
    } else {
        range.min * (1.0 - fraction) + range.max * fraction
    }
}

fn tick_precision(step: f64) -> usize {
    if !step.is_finite() || step == 0.0 {
        return 0;
    }
    (0..=4)
        .find(|precision| {
            let scaled = step * 10_f64.powi(*precision as i32);
            (scaled - scaled.round()).abs() <= 1e-8
        })
        .unwrap_or(4)
}

fn format_tick(value: f64, precision: usize) -> String {
    if value != 0.0 && !(1e-4..1e6).contains(&value.abs()) {
        return format!("{value:.3e}");
    }
    let threshold = 0.5 * 10_f64.powi(-(precision as i32));
    let value = if value.abs() < threshold { 0.0 } else { value };
    format!("{value:.precision$}")
}

type ProjectedRuns = (Vec<Vec<Point<Pixels>>>, Vec<Vec<Sample>>, usize, usize);

fn line_runs(
    samples: &[Sample],
    x: Range,
    y: Range,
    bounds: Bounds<Pixels>,
    x_scale: &Scale,
    y_scale: &Scale,
    layer: &LineLayer,
) -> ProjectedRuns {
    let accepts = |sample: &Sample| x_scale.accepts(sample.x) && y_scale.accepts(sample.y);
    let mut runs = Vec::new();
    let mut retained_samples = Vec::new();
    let mut valid_points = 0;
    let mut visible_points = 0;
    let mut cursor = 0;
    while cursor < samples.len() {
        if !accepts(&samples[cursor]) {
            cursor += 1;
            continue;
        }
        let start = cursor;
        while cursor < samples.len() && accepts(&samples[cursor]) {
            cursor += 1;
        }
        valid_points += cursor - start;
        let mut draw = |visible: &[Sample]| {
            visible_points += visible.len();
            let mut points = Vec::new();
            let mut observations = Vec::new();
            super::decimation::reduce(
                visible,
                |value| (f32::from(bounds.size.width) * normalized(x, value, x_scale)).floor(),
                layer.decimation_policy(),
                |sample| {
                    observations.push(sample);
                    points.push(point(
                        bounds.origin.x + bounds.size.width * normalized(x, sample.x, x_scale),
                        bounds.origin.y + bounds.size.height * (1.0 - normalized(y, sample.y, y_scale)),
                    ))
                },
            );
            runs.push(points);
            retained_samples.push(observations);
        };
        let accepted = &samples[start..cursor];
        if matches!(x_scale, Scale::Numeric(scale) if scale.domain_bounds().is_some()) {
            super::viewport::select(
                accepted,
                x.min,
                x.max,
                usize::from(layer.interpolation_policy() == Interpolation::Monotone),
                draw,
            );
        } else {
            draw(accepted);
        }
    }
    (runs, retained_samples, valid_points, visible_points)
}

#[cfg(test)]
mod tests {
    use crate::Series;
    use super::*;
    use crate::Decimation;
    use gpui::{px, size};

    fn numeric_runs(samples: &[Sample], x: Range, y: Range, bounds: Bounds<Pixels>) -> Vec<Vec<Point<Pixels>>> {
        let scale = crate::plots::NumericScale::new().into();
        line_runs(samples, x, y, bounds, &scale, &scale, &LineLayer::new("a")).0
    }

    fn bounds() -> Bounds<Pixels> {
        Bounds::new(point(px(10.0), px(20.0)), size(px(100.0), px(80.0)))
    }

    fn test_data(samples: impl Into<std::sync::Arc<[Sample]>>) -> Dataset {
        Dataset::new().with_series(crate::Series::new("a", "A", samples)).expect("unique series")
    }

    fn labels(ticks: &[TickGeometry]) -> Vec<&str> {
        ticks.iter().map(|tick| tick.label.as_str()).collect()
    }

    #[test]
    fn logarithmic_projection_ticks_gaps_and_annotations_share_mapping() {
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "A",
                [
                    Sample::new(1.0, 1.0),
                    Sample::new(10.0, 10.0),
                    Sample::new(20.0, 0.0),
                    Sample::new(30.0, -1.0),
                    Sample::new(100.0, 100.0),
                ],
            ))
            .unwrap();
        let plot = CartesianPlot::new()
            .x_scale(crate::NumericScale::new().logarithmic(10.0).domain(1.0..=100.0))
            .y_scale(crate::NumericScale::new().logarithmic(10.0).domain(1.0..=100.0))
            .line(crate::LineLayer::new("a"));
        let bounds = Bounds::new(Point::default(), size(px(100.0), px(100.0)));
        let geometry = layout(&data, &plot, Edges::all(px(0.0)), bounds);
        assert_eq!(geometry.lines[0].valid_points, 3);
        assert_eq!(geometry.lines[0].runs.len(), 2);
        assert_eq!(geometry.lines[0].runs[0][1], point(px(50.0), px(50.0)));
        assert_eq!(geometry.data_position(Sample::new(10.0, 10.0)), Some(point(px(50.0), px(50.0))));
        assert!(geometry.data_position(Sample::new(0.0, 10.0)).is_none());
        assert_eq!(geometry.x_ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["1", "10", "100"]);
        assert_eq!(geometry.x_position(10.0), px(50.0));
    }
    #[test]
    fn log_bar_requires_positive_baseline_and_empty_domains_remain_finite() {
        let data = Dataset::new().with_series(Series::new("a", "A", [Sample::new(0.0, 100.0)])).unwrap();
        let base = CartesianPlot::new().y_scale(crate::NumericScale::new().logarithmic(10.0));
        let hidden = layout(&data, &base.clone().bar(crate::BarLayer::new("a")), Edges::all(px(0.0)), bounds());
        assert!(hidden.bars[0].rectangles.is_empty());
        let shown =
            layout(&data, &base.clone().bar(crate::BarLayer::new("a").baseline(1.0)), Edges::all(px(0.0)), bounds());
        assert_eq!(shown.bars[0].rectangles.len(), 1);
        let empty = layout(&Dataset::new(), &base.line(crate::LineLayer::new("a")), Edges::all(px(0.0)), bounds());
        assert!(empty.y_ticks.iter().all(|t| f32::from(t.position).is_finite()));
    }
    #[test]
    fn custom_numeric_tick_labels_preserve_date_spacing_and_sample_mapping() {
        let data = test_data([Sample::new(0.0, 1.0), Sample::new(30.0, 3.0), Sample::new(90.0, 2.0)]);
        let base = CartesianPlot::new().line(crate::LineLayer::new("a"));
        let raw = layout(&data, &base, Edges::default(), bounds());
        let labeled =
            base.x_scale(crate::NumericScale::new().tick_labels([(0.0, "Apr 1"), (30.0, "May 1"), (90.0, "Jun 30")]));
        let geometry = layout(&data, &labeled, Edges::default(), bounds());
        assert_eq!(labels(&geometry.x_ticks), ["Apr 1", "May 1", "Jun 30"]);
        assert_eq!(geometry.lines[0].runs, raw.lines[0].runs);
        assert_eq!(geometry.x_domain, raw.x_domain);
        let distance = geometry.x_ticks[1].position - geometry.x_ticks[0].position;
        assert!((f32::from(distance) - f32::from(geometry.plot_bounds.size.width) / 3.0).abs() < 1e-4);
    }

    #[test]
    fn area_baselines_expand_only_automatic_y_bounds_of_existing_series() {
        let data = test_data([Sample::new(2.0, 5.0), Sample::new(4.0, 10.0)]);
        let base = CartesianPlot::new().area(crate::AreaLayer::new("a"));
        let geometry = layout(&data, &base, Edges::default(), bounds());
        assert_eq!(geometry.x_domain, (2.0, 4.0));
        assert_eq!(geometry.y_ticks.first().expect("Y ticks").label.parse::<f64>().expect("numeric label"), 0.0);
        assert_eq!(geometry.areas[0].baseline, geometry.plot_bounds.bottom());
        let raised = CartesianPlot::new()
            .area(crate::AreaLayer::new("a").baseline(20.0))
            .area(crate::AreaLayer::new("absent").baseline(1000.0));
        let geometry = layout(&data, &raised, Edges::default(), bounds());
        assert_eq!(geometry.y_ticks.last().expect("Y ticks").label.parse::<f64>().expect("numeric label"), 20.0);
        assert_eq!(geometry.areas.len(), 1);
        assert_eq!(geometry.areas[0].baseline, geometry.plot_bounds.top());
        let explicit = base.y_scale(crate::NumericScale::new().domain(5.0..=10.0));
        let geometry = layout(&data, &explicit, Edges::default(), bounds());
        assert_eq!(geometry.y_ticks.first().expect("Y ticks").label.parse::<f64>().expect("numeric label"), 5.0);
        assert!(geometry.areas[0].baseline > geometry.plot_bounds.bottom());
        let negative = test_data([Sample::new(2.0, -10.0), Sample::new(4.0, -5.0)]);
        let geometry =
            layout(&negative, &CartesianPlot::new().area(crate::AreaLayer::new("a")), Edges::default(), bounds());
        assert_eq!(geometry.y_ticks.last().expect("Y ticks").label.parse::<f64>().expect("numeric label"), 0.0);
        assert_eq!(geometry.areas[0].baseline, geometry.plot_bounds.top());
    }

    #[test]
    fn area_boundaries_reuse_line_reduction_and_keep_viewport_gaps_open() {
        let mut samples: Vec<_> = (0..200).map(|i| Sample::new(i as f64, (i as f64 / 10.0).sin())).collect();
        samples[100].y = f64::NAN;
        let data = test_data(samples);
        let plot = CartesianPlot::new()
            .line(crate::LineLayer::new("a").decimation(Decimation::MinMax).interpolation(Interpolation::Monotone))
            .area(crate::AreaLayer::new("a").decimation(Decimation::MinMax).interpolation(Interpolation::Monotone));
        let geometry =
            layout(&data, &plot, Edges::default(), Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(80.0))));
        let boundary = &geometry.areas[0].boundary;
        assert_eq!(boundary.runs, geometry.lines[0].runs);
        assert_eq!(boundary.valid_points, 199);
        assert_eq!(boundary.runs.len(), 2);
        assert!(boundary.runs.iter().map(Vec::len).sum::<usize>() < 199);
        let gap = plot.x_scale(crate::NumericScale::new().domain(99.5..=100.5));
        let geometry = layout(&data, &gap, Edges::default(), bounds());
        assert!(geometry.lines[0].runs.is_empty());
        assert!(geometry.areas[0].boundary.runs.is_empty());
        assert!(data.series()[0].samples()[100].y.is_nan());
    }

    #[test]
    fn viewport_keeps_crossing_neighbors_and_full_dataset_y_domain() {
        let data = test_data((0..100).map(|index| Sample::new(index as f64, index as f64)).collect::<Vec<_>>());
        let plot = CartesianPlot::new()
            .x_scale(crate::NumericScale::new().domain(40.5..=45.5))
            .line(crate::LineLayer::new("a").decimation(Decimation::MinMax));
        let geometry = layout(&data, &plot, Edges::default(), bounds());
        assert_eq!(geometry.x_domain, (40.5, 45.5));
        assert_eq!(geometry.lines[0].valid_points, 100);
        assert_eq!(geometry.lines[0].visible_points, 7);
        assert_eq!(geometry.lines[0].runs[0].len(), 7);
        assert!(geometry.lines[0].runs[0].first().expect("neighbor").x < geometry.plot_bounds.left());
        assert!(geometry.lines[0].runs[0].last().expect("neighbor").x > geometry.plot_bounds.right());
        assert_eq!(geometry.y_ticks.first().expect("ticks").label, "0.00");
        assert_eq!(geometry.y_ticks.last().expect("ticks").label, "99.00");
        let moved = plot.x_scale(crate::NumericScale::new().domain(70.5..=75.5));
        let moved_geometry = layout(&data, &moved, Edges::default(), bounds());
        assert_eq!(labels(&geometry.y_ticks), labels(&moved_geometry.y_ticks));
    }

    #[test]
    fn viewport_never_bridges_missing_data_or_draws_disjoint_offscreen_runs() {
        let data = test_data([
            Sample::new(0.0, 0.0),
            Sample::new(10.0, 1.0),
            Sample::new(15.0, f64::NAN),
            Sample::new(20.0, 2.0),
            Sample::new(30.0, 3.0),
        ]);
        let base = CartesianPlot::new().line(crate::LineLayer::new("a"));
        let gap = base.clone().x_scale(crate::NumericScale::new().domain(12.0..=18.0));
        let geometry = layout(&data, &gap, Edges::default(), bounds());
        assert!(geometry.lines[0].runs.is_empty());
        assert_eq!(geometry.lines[0].visible_points, 0);
        assert!(!gap.has_visible_samples(data.series()[0].samples()));
        let crossing = base.clone().x_scale(crate::NumericScale::new().domain(4.0..=6.0));
        let geometry = layout(&data, &crossing, Edges::default(), bounds());
        assert_eq!(geometry.lines[0].runs.len(), 1);
        assert_eq!(geometry.lines[0].visible_points, 2);
        assert!(crossing.has_visible_samples(data.series()[0].samples()));
        let outside = base.x_scale(crate::NumericScale::new().domain(40.0..=50.0));
        assert!(layout(&data, &outside, Edges::default(), bounds()).lines[0].runs.is_empty());
        assert!(!outside.has_visible_samples(data.series()[0].samples()));
    }

    #[test]
    fn interpolation_keeps_missing_intervals_open_before_and_after_decimation() {
        let data = test_data([
            Sample::new(0.0, 2.0),
            Sample::new(1.0, 3.0),
            Sample::new(2.5, 8.0),
            Sample::new(4.0, 9.0),
            Sample::new(5.0, 9.0),
            Sample::new(6.5, 4.0),
            Sample::new(7.25, f64::NAN),
            Sample::new(8.0, 1.0),
            Sample::new(9.0, 2.0),
            Sample::new(10.5, 7.0),
            Sample::new(12.0, 8.0),
        ]);
        for decimation in [Decimation::None, Decimation::MinMax] {
            let plot = CartesianPlot::new()
                .line(crate::LineLayer::new("a").decimation(decimation).interpolation(Interpolation::Monotone));
            let geometry = layout(&data, &plot, Edges::default(), bounds());
            let line = &geometry.lines[0];
            assert_eq!(line.interpolation, Interpolation::Monotone);
            assert_eq!(line.valid_points, 10);
            assert_eq!(line.runs.len(), 2);
            assert!(line.runs[0].last().expect("left edge").x < line.runs[1][0].x);
            for run in &line.runs {
                let segments = super::super::interpolation::monotone(run).expect("smooth independent run");
                assert_eq!(segments.last().expect("segments").end, *run.last().expect("last observation"));
            }
            let gap = plot.x_scale(crate::NumericScale::new().domain(6.75..=7.75));
            assert!(layout(&data, &gap, Edges::default(), bounds()).lines[0].runs.is_empty());
        }
        assert_eq!(data.series()[0].samples().len(), 11);
        assert!(data.series()[0].samples()[6].y.is_nan());
    }

    #[test]
    fn decimation_preserves_gaps_domains_and_source_samples() {
        let mut samples: Vec<Sample> = (0..200).map(|index| Sample::new(index as f64, 0.0)).collect();
        samples[20].y = 50.0;
        samples[80].y = -40.0;
        samples[100].y = f64::NAN;
        let data = test_data(samples);
        let raw = CartesianPlot::new().line(crate::LineLayer::new("a"));
        let reduced = CartesianPlot::new().line(crate::LineLayer::new("a").decimation(crate::Decimation::MinMax));
        let area = Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(80.0)));
        let raw_geometry = layout(&data, &raw, Edges::default(), area);
        let reduced_geometry = layout(&data, &reduced, Edges::default(), area);
        assert_eq!(raw_geometry.lines[0].valid_points, 199);
        assert_eq!(reduced_geometry.lines[0].valid_points, 199);
        let raw_runs = &raw_geometry.lines[0].runs;
        let reduced_runs = &reduced_geometry.lines[0].runs;
        assert_eq!(reduced_runs.len(), 2);
        assert!(reduced_runs.iter().map(Vec::len).sum::<usize>() < 199);
        for (raw, reduced) in raw_runs.iter().zip(reduced_runs) {
            assert_eq!(raw.first(), reduced.first());
            assert_eq!(raw.last(), reduced.last());
        }
        assert!(reduced_runs[0].contains(&raw_runs[0][20]));
        assert!(reduced_runs[0].contains(&raw_runs[0][80]));
        assert_eq!(labels(&raw_geometry.x_ticks), labels(&reduced_geometry.x_ticks));
        assert_eq!(labels(&raw_geometry.y_ticks), labels(&reduced_geometry.y_ticks));
        assert_eq!(data.series()[0].samples().len(), 200);
        assert!(data.series()[0].samples()[100].y.is_nan());
    }

    #[test]
    fn categories_keep_configured_order_and_missing_positions() {
        use crate::{CategoryScale, LineLayer};
        let data = test_data([Sample::new(1.0, 18.0), Sample::new(f64::NAN, f64::NAN), Sample::new(3.0, 27.0)]);
        let plot = CartesianPlot::new()
            .x_scale(CategoryScale::new(["Mon", "Tue", "Wed", "Thu", "Fri"]))
            .line(LineLayer::new("a"));
        let geometry = layout(&data, &plot, Edges::all(px(0.0)), bounds());
        assert_eq!(labels(&geometry.x_ticks), ["Mon", "Tue", "Wed", "Thu", "Fri"]);
        assert_eq!(
            geometry.x_ticks.iter().map(|tick| tick.position).collect::<Vec<_>>(),
            [px(10.0), px(35.0), px(60.0), px(85.0), px(110.0)]
        );
        assert_eq!(geometry.lines[0].runs.len(), 2);
        assert_eq!(geometry.lines[0].runs[0][0].x, geometry.x_ticks[1].position);
        assert_eq!(geometry.lines[0].runs[1][0].x, geometry.x_ticks[3].position);
    }

    #[test]
    fn vertical_categories_align_with_y_coordinates() {
        use crate::{CategoryScale, LineLayer};
        let data = test_data([Sample::new(0.0, 0.0), Sample::new(1.0, 2.0)]);
        let plot = CartesianPlot::new().y_scale(CategoryScale::new(["Low", "Mid", "High"])).line(LineLayer::new("a"));
        let geometry = layout(&data, &plot, Edges::all(px(0.0)), bounds());
        assert_eq!(labels(&geometry.y_ticks), ["Low", "Mid", "High"]);
        assert_eq!(
            geometry.y_ticks.iter().map(|tick| tick.position).collect::<Vec<_>>(),
            [px(100.0), px(60.0), px(20.0)]
        );
        assert_eq!(geometry.lines[0].runs[0][0].y, geometry.y_ticks[0].position);
        assert_eq!(geometry.lines[0].runs[0][1].y, geometry.y_ticks[2].position);
    }

    #[test]
    fn unknown_or_fractional_categories_break_runs_without_distorting_numeric_domain() {
        use crate::{CategoryScale, LineLayer};
        let data = test_data([
            Sample::new(0.0, 1.0),
            Sample::new(0.5, 900.0),
            Sample::new(1.0, 2.0),
            Sample::new(5.0, 900.0),
            Sample::new(2.0, 3.0),
        ]);
        let plot = CartesianPlot::new().x_scale(CategoryScale::new(["A", "B", "C"])).line(LineLayer::new("a"));
        let geometry = layout(&data, &plot, Edges::all(px(0.0)), bounds());
        assert_eq!(geometry.lines[0].runs.iter().map(Vec::len).collect::<Vec<_>>(), [1, 1, 1]);
        assert_eq!(geometry.y_ticks.last().expect("ticks").label, "3.0");
    }

    #[test]
    fn explicit_ticks_use_numeric_distance_and_preserve_line_mapping() {
        use crate::{LineLayer, NumericScale};
        let data = test_data([Sample::new(0.0, 18.0), Sample::new(100.0, 27.0)]);
        let base = CartesianPlot::new().line(LineLayer::new("a"));
        let explicit = base.clone().x_scale(NumericScale::new().tick_values([
            100.0,
            10.0,
            0.0,
            -0.0,
            50.0,
            10.0,
            f64::NAN,
            f64::INFINITY,
            -1.0,
            101.0,
        ]));
        let a = layout(&data, &base, Edges::all(px(0.0)), bounds());
        let b = layout(&data, &explicit, Edges::all(px(0.0)), bounds());
        assert_eq!(labels(&b.x_ticks), ["0", "10", "50", "100"]);
        assert_eq!(
            b.x_ticks.iter().map(|tick| tick.position).collect::<Vec<_>>(),
            [px(10.0), px(20.0), px(60.0), px(110.0)]
        );
        assert_eq!(a.lines[0].runs, b.lines[0].runs);
        let no_ticks = layout(&data, &base.x_scale(NumericScale::new().tick_values([])), Edges::all(px(0.0)), bounds());
        assert!(no_ticks.x_ticks.is_empty());
        assert_eq!(a.lines[0].runs, no_ticks.lines[0].runs);
    }

    #[test]
    fn selected_fractional_ticks_retain_their_values() {
        use crate::NumericScale;
        let scale = NumericScale::new().tick_values([1.23456, 1.23457]);
        let ticks = scale_ticks_at(px(0.0), px(100.0), Range { min: 1.0, max: 2.0 }, &scale.into());
        assert_eq!(labels(&ticks), ["1.23456", "1.23457"]);
    }

    #[test]
    fn empty_single_category_and_constant_numeric_scales_are_safe() {
        use crate::{CategoryScale, LineLayer, NumericScale};
        let data = test_data([Sample::new(0.0, 18.0)]);
        let base = CartesianPlot::new().line(LineLayer::new("a"));
        let one = layout(&data, &base.clone().x_scale(CategoryScale::new(["Only"])), Edges::all(px(0.0)), bounds());
        assert_eq!(labels(&one.x_ticks), ["Only"]);
        assert_eq!(one.x_ticks[0].position, px(60.0));
        assert_eq!(one.lines[0].runs[0][0].x, one.x_ticks[0].position);
        assert_eq!(one.y_ticks.len(), 1);
        assert_eq!(one.y_ticks[0].position, px(60.0));
        let selected = layout(
            &data,
            &base.clone().y_scale(NumericScale::new().tick_values([18.0])),
            Edges::all(px(0.0)),
            bounds(),
        );
        assert_eq!(selected.y_ticks[0].position, selected.lines[0].runs[0][0].y);
        let empty = layout(&data, &base.x_scale(CategoryScale::new(Vec::<&str>::new())), Edges::all(px(0.0)), bounds());
        assert!(empty.x_ticks.is_empty());
        assert!(empty.lines[0].runs.is_empty());
    }

    #[test]
    fn relocating_y_axis_preserves_scale_mapping_and_tick_values() {
        use crate::plots::LineLayer;
        let data = Dataset::new()
            .with_series(crate::data::Series::new("a", "A", [Sample::new(0.0, 10.0), Sample::new(1.0, 20.0)]))
            .expect("unique series");
        let plot = CartesianPlot::new().line(LineLayer::new("a"));
        let left =
            layout(&data, &plot, Edges { left: px(32.0), right: px(4.0), top: px(8.0), bottom: px(32.0) }, bounds());
        let right =
            layout(&data, &plot, Edges { left: px(4.0), right: px(32.0), top: px(8.0), bottom: px(32.0) }, bounds());
        assert_eq!(left.plot_bounds.size, right.plot_bounds.size);
        for (a, b) in left.lines[0].runs[0].iter().zip(&right.lines[0].runs[0]) {
            assert_eq!(*a - left.plot_bounds.origin, *b - right.plot_bounds.origin);
        }
        let labels =
            |geometry: &CartesianGeometry| geometry.y_ticks.iter().map(|tick| tick.label.clone()).collect::<Vec<_>>();
        assert_eq!(labels(&left), labels(&right));
        assert_eq!(right.y_ticks[0].label, "10.0");
        assert_eq!(right.y_ticks[0].position, right.plot_bounds.bottom());
        assert_eq!(right.y_ticks.last().expect("ticks").label, "20.0");
        assert_eq!(right.y_ticks.last().expect("ticks").position, right.plot_bounds.origin.y);
        assert_eq!(right.lines[0].runs[0][0].y, right.y_ticks[0].position);
    }

    #[test]
    fn oversized_gutters_fit_small_and_zero_bounds() {
        for extent in [0.0, 10.0] {
            let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(extent), px(extent)));
            let geometry = layout(&Dataset::new(), &CartesianPlot::new(), Edges::all(px(100.0)), bounds);
            assert!(geometry.plot_bounds.size.width >= px(0.0));
            assert!(geometry.plot_bounds.size.height >= px(0.0));
            assert!(geometry.x_ticks.iter().chain(&geometry.y_ticks).all(|tick| f32::from(tick.position).is_finite()));
        }
    }

    #[test]
    fn maps_negative_values_and_inverts_screen_y() {
        let runs = numeric_runs(
            &[Sample::new(-2.0, -4.0), Sample::new(2.0, 4.0)],
            Range { min: -2.0, max: 2.0 },
            Range { min: -4.0, max: 4.0 },
            bounds(),
        );
        assert_eq!(runs[0], vec![point(px(10.0), px(100.0)), point(px(110.0), px(20.0))]);
    }

    #[test]
    fn missing_samples_break_connections() {
        let runs = numeric_runs(
            &[Sample::new(0.0, 0.0), Sample::new(f64::NAN, 1000.0), Sample::new(1.0, 1.0)],
            Range { min: 0.0, max: 1.0 },
            Range { min: 0.0, max: 1.0 },
            bounds(),
        );
        assert_eq!(runs, vec![vec![point(px(10.0), px(100.0))], vec![point(px(110.0), px(20.0))]]);
    }

    #[test]
    fn extreme_finite_ranges_do_not_overflow() {
        let runs = numeric_runs(
            &[Sample::new(-f64::MAX, -f64::MAX), Sample::new(0.0, 0.0), Sample::new(f64::MAX, f64::MAX)],
            Range { min: -f64::MAX, max: f64::MAX },
            Range { min: -f64::MAX, max: f64::MAX },
            bounds(),
        );
        assert_eq!(runs[0][1], point(px(60.0), px(60.0)));
    }

    #[test]
    fn tick_labels_keep_fractional_precision() {
        let ticks = ticks(px(0.0), px(100.0), Range { min: 18.0, max: 56.0 }, 5);
        let labels: Vec<_> = ticks.iter().map(|tick| tick.label.as_str()).collect();
        assert_eq!(labels, ["18.0", "27.5", "37.0", "46.5", "56.0"]);
    }

    #[test]
    fn extreme_finite_ranges_have_finite_tick_labels() {
        let ticks = ticks(px(0.0), px(100.0), Range { min: -f64::MAX, max: f64::MAX }, 3);
        assert!(ticks.iter().all(|tick| tick.label != "NaN" && tick.label != "inf"));
        assert_eq!(ticks[0].label, "-1.798e308");
        assert_eq!(ticks[2].label, "1.798e308");
    }
}

#[cfg(test)]
mod bar_tests {
    use super::*;
    use crate::{BarLayer, CategoryScale, NumericScale, Series};

    #[test]
    fn grouped_signed_bars_fit_category_bands_and_skip_missing_values() {
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "A",
                [Sample::new(0.0, 10.0), Sample::new(1.0, -10.0), Sample::new(2.0, f64::NAN)],
            ))
            .unwrap()
            .with_series(Series::new("b", "B", [Sample::new(0.0, 5.0), Sample::new(1.0, -5.0)]))
            .unwrap();
        let plot = CartesianPlot::new()
            .x_scale(CategoryScale::new(["A", "B", "C"]))
            .bar(BarLayer::new("a"))
            .bar(BarLayer::new("b"));
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(300.0), px(200.0)));
        let geometry = layout(&data, &plot, Edges::default(), bounds);
        assert_eq!(geometry.x_domain, (-0.5, 2.5));
        assert_eq!(geometry.x_ticks[0].position, px(50.0));
        assert_eq!(geometry.bars[0].rectangles.len(), 2);
        let first = geometry.bars[0].rectangles[0];
        let second = geometry.bars[1].rectangles[0];
        assert!((f32::from(first.left()) - 12.0).abs() < 1e-4);
        assert!((f32::from(second.left() - first.right()) - 4.0).abs() < 1e-4);
        assert!((f32::from(first.size.width) - 36.0).abs() < 1e-4);
        assert_eq!(first.bottom(), px(100.0));
        assert_eq!(geometry.bars[0].rectangles[1].top(), px(100.0));
        assert!(second.right() < px(100.0));
        let explicit = plot.y_scale(NumericScale::new().domain(-20.0..=20.0));
        assert_eq!(layout(&data, &explicit, Edges::default(), bounds).bars[0].rectangles[0].size.height, px(50.0));
    }

    #[test]
    fn numeric_bars_include_baseline_and_slot_edges_in_auto_bounds() {
        let data = Dataset::new().with_series(Series::new("a", "A", [Sample::new(2.0, 5.0)])).unwrap();
        let plot = CartesianPlot::new().bar(BarLayer::new("a"));
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(100.0), px(100.0)));
        let geometry = layout(&data, &plot, Edges::default(), bounds);
        assert_eq!(geometry.x_domain, (1.5, 2.5));
        assert_eq!(geometry.bars[0].rectangles[0].bottom(), px(100.0));
        assert_eq!(geometry.bars[0].rectangles[0].top(), px(0.0));
        let outside = plot.x_scale(NumericScale::new().domain(10.0..=20.0));
        assert!(layout(&data, &outside, Edges::default(), bounds).bars[0].rectangles.is_empty());
    }
}
