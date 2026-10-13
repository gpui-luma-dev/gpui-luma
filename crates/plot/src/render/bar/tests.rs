use super::*;
use crate::{BarLayer, BarLayout, CategoryScale, NumericScale, Series};
use gpui::Edges;
use super::super::geometry::{layout, scale_ticks};

fn data() -> Dataset {
    Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(0.0, 10.0), Sample::new(1.0, -10.0)]))
        .unwrap()
        .with_series(Series::new("b", "B", [Sample::new(0.0, 5.0), Sample::new(1.0, -5.0)]))
        .unwrap()
}
fn plot() -> CartesianPlot {
    CartesianPlot::new()
        .x_scale(CategoryScale::new(["A", "B"]))
        .y_scale(NumericScale::new().domain(-20.0..=20.0).tick_values([-20.0, 0.0, 20.0]))
        .bar(BarLayer::new("a"))
        .bar(BarLayer::new("b"))
}
fn bounds() -> Bounds<Pixels> {
    Bounds::new(point(px(10.0), px(20.0)), size(px(200.0), px(100.0)))
}

#[test]
fn four_orientations_keep_signed_lengths_and_match_axis_ticks() {
    for orientation in [BarOrientation::Up, BarOrientation::Down, BarOrientation::Left, BarOrientation::Right] {
        let plot = plot().bar_orientation(orientation);
        let geometry = layout(&data(), &plot, Edges::default(), bounds());
        let placement = &geometry.bars[0].placements[0];
        let negative = &geometry.bars[0].placements[1];
        let positive_delta = placement.endpoint - placement.baseline;
        let negative_delta = negative.endpoint - negative.baseline;
        match orientation {
            BarOrientation::Up => {
                assert_eq!(positive_delta.y, px(-25.0));
                assert_eq!(negative_delta.y, px(25.0));
            }
            BarOrientation::Down => {
                assert_eq!(positive_delta.y, px(25.0));
                assert_eq!(negative_delta.y, px(-25.0));
            }
            BarOrientation::Right => {
                assert_eq!(positive_delta.x, px(50.0));
                assert_eq!(negative_delta.x, px(-50.0));
            }
            BarOrientation::Left => {
                assert_eq!(positive_delta.x, px(-50.0));
                assert_eq!(negative_delta.x, px(50.0));
            }
        }
        let (category, value) = if orientation.is_horizontal() {
            (&geometry.y_ticks, &geometry.x_ticks)
        } else {
            (&geometry.x_ticks, &geometry.y_ticks)
        };
        assert_eq!(category.iter().map(|tick| tick.label.as_str()).collect::<Vec<_>>(), ["A", "B"]);
        assert_eq!(value[1].label, "0");
        assert_eq!(
            value[1].position,
            if orientation.is_horizontal() {
                placement.baseline.x
            } else {
                placement.baseline.y
            }
        );
        let (x, y) = scale_ticks(&data(), &plot);
        assert_eq!(
            x.iter().map(|tick| &tick.label).collect::<Vec<_>>(),
            geometry.x_ticks.iter().map(|tick| &tick.label).collect::<Vec<_>>()
        );
        assert_eq!(
            y.iter().map(|tick| &tick.label).collect::<Vec<_>>(),
            geometry.y_ticks.iter().map(|tick| &tick.label).collect::<Vec<_>>()
        );
    }
}

#[test]
fn stacks_accumulate_signed_values_and_expand_auto_bounds() {
    let plot = plot().y_scale(NumericScale::new()).bar_layout(BarLayout::new().grouping(BarGrouping::Stacked));
    let segments = segments(&data(), &plot);
    assert_eq!((segments[1][0].start, segments[1][0].end), (10.0, 15.0));
    assert_eq!((segments[1][1].start, segments[1][1].end), (-10.0, -15.0));
    let geometry = layout(&data(), &plot, Edges::default(), bounds());
    assert_eq!(geometry.y_ticks.first().unwrap().label, "-15.0");
    assert_eq!(geometry.y_ticks.last().unwrap().label, "15.0");
    assert_eq!(geometry.bars[0].rectangles[0].left(), geometry.bars[1].rectangles[0].left());
    assert_eq!(geometry.bars[1].placements[0].baseline, geometry.bars[0].placements[0].endpoint);
}

#[test]
fn stacks_use_coordinates_and_baselines_and_skip_missing_or_overflowing_values() {
    let data = Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(1.0, 8.0), Sample::new(0.0, 12.0)]))
        .unwrap()
        .with_series(Series::new("b", "B", [Sample::new(0.0, 15.0), Sample::new(1.0, f64::NAN)]))
        .unwrap();
    let plot = CartesianPlot::new()
        .bar_layout(BarLayout::new().grouping(BarGrouping::Stacked))
        .bar(BarLayer::new("a").baseline(10.0))
        .bar(BarLayer::new("b").baseline(10.0));
    let values = segments(&data, &plot);
    assert_eq!(values[1].len(), 1);
    assert_eq!((values[1][0].start, values[1][0].end), (12.0, 17.0));
    assert_eq!(values[1][0].sample_index, 0);
    let distinct = plot.clone().bar(BarLayer::new("b").baseline(0.0));
    assert_eq!(segments(&data, &distinct)[2][0].start, 0.0);
    let extreme = Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(0.0, f64::MAX)]))
        .unwrap()
        .with_series(Series::new("b", "B", [Sample::new(0.0, f64::MAX)]))
        .unwrap();
    assert!(segments(&extreme, &plot)[1].is_empty());
}

#[test]
fn shared_alignment_gap_and_invalid_settings_are_predictable() {
    for (alignment, expected) in [(BarAlignment::Start, -0.5), (BarAlignment::Center, -0.3), (BarAlignment::End, -0.1)]
    {
        let plot = plot().bar_layout(BarLayout::new().width(0.6).gap(0.0).alignment(alignment));
        let segments = segments(&data(), &plot);
        assert!((segments[0][0].category_start - expected).abs() < 1e-8);
        assert_eq!(segments[0][0].category_end, segments[1][0].category_start);
    }
    let plot = plot().bar_layout(BarLayout::new().gap(0.5));
    let segments = segments(&data(), &plot);
    assert!(segments[0][0].category_end < segments[1][0].category_start);
    let layout = BarLayout::new().width(f64::NAN).gap(f64::INFINITY);
    assert_eq!(layout.width_fraction(), 0.8);
    assert_eq!(layout.gap_fraction(), 0.1);
    assert_eq!(layout.width(-1.0).width_fraction(), 0.0);
    assert_eq!(layout.gap(2.0).gap_fraction(), 1.0);
}

#[test]
fn label_anchors_follow_baseline_and_endpoint_in_all_directions() {
    use crate::{BarLabel, BarLabelPlacement, TextPresentation};
    for orientation in [BarOrientation::Up, BarOrientation::Down, BarOrientation::Left, BarOrientation::Right] {
        let geometry = layout(&data(), &plot().bar_orientation(orientation), Edges::default(), bounds());
        for i in 0..2 {
            let model = model(
                &geometry.bars[0],
                geometry.bars[0].rectangles[i],
                &geometry.bars[0].placements[i],
                &crate::CartesianPresentation::default(),
                orientation,
            );
            let label = BarLabel::new("10", TextPresentation::new(gpui::white())).gap(px(2.0));
            let text_size = size(px(8.0), px(8.0));
            let center = label_origin(&model, &label, text_size) + point(px(4.0), px(4.0));
            let direction = model.endpoint - model.baseline;
            let offset = center - model.endpoint;
            if orientation.is_horizontal() {
                assert!(f32::from(offset.x) * f32::from(direction.x) > 0.0);
                assert_eq!(offset.y, px(0.0));
            } else {
                assert!(f32::from(offset.y) * f32::from(direction.y) > 0.0);
                assert_eq!(offset.x, px(0.0));
            }
            let inside = label.placement(BarLabelPlacement::InsideStart);
            let center = label_origin(&model, &inside, text_size) + point(px(4.0), px(4.0));
            assert!(model.bounds.contains(&center));
        }
    }
}

#[test]
fn bar_hit_testing_respects_signed_rectangles_gaps_and_reveal_clipping() {
    for orientation in [BarOrientation::Up, BarOrientation::Down, BarOrientation::Left, BarOrientation::Right] {
        let data = data();
        let geometry = layout(&data, &plot().bar_orientation(orientation), Edges::default(), bounds());
        let style = crate::CartesianPresentation::default();
        for i in 0..2 {
            let pointer = geometry.bars[0].rectangles[i].center();
            let selected = select(&geometry, &data, &style, pointer, bounds());
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].info.sample, data.series()[0].samples()[i]);
            let invisible = Bounds::new(bounds().origin, size(px(1.0), px(1.0)));
            assert!(select(&geometry, &data, &style, pointer, invisible).is_empty());
        }
        let a = geometry.bars[0].rectangles[0];
        let b = geometry.bars[1].rectangles[0];
        let pointer = if orientation.is_horizontal() {
            point(a.center().x, (a.bottom() + b.top()) / 2.0)
        } else {
            point((a.right() + b.left()) / 2.0, a.center().y)
        };
        assert!(select(&geometry, &data, &style, pointer, bounds()).is_empty());
    }
}

#[test]
fn large_sorted_viewport_keeps_original_indices_and_projects_only_visible_bars() {
    let samples: Vec<_> = (0..100_000).map(|index| Sample::new(index as f64, (index % 100) as f64 + 1.0)).collect();
    let data = Dataset::new().with_series(Series::new("a", "A", samples)).unwrap();
    let original = data.series()[0].samples().as_ptr();
    let plot = CartesianPlot::new()
        .x_scale(NumericScale::new().domain(50_000.5..=50_080.5))
        .y_scale(NumericScale::new().domain(0.0..=120.0))
        .bar(BarLayer::new("a"));
    let geometry = layout(&data, &plot, Edges::default(), bounds());
    assert_eq!(geometry.bars[0].valid_points, 100_000);
    assert_eq!(geometry.bars[0].rectangles.len(), 80);
    assert_eq!(geometry.bars[0].placements[0].sample_index, 50_001);
    assert_eq!(geometry.bars[0].placements.last().unwrap().sample_index, 50_080);
    assert_eq!(data.series()[0].samples().as_ptr(), original);
    assert!(plot.has_visible_bars(&data.series()[0]));
}

#[test]
fn viewport_stack_selection_preserves_offsets_and_unsorted_series_indices() {
    let data = Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(9.0, 3.0), Sample::new(1.0, 2.0), Sample::new(5.0, 4.0)]))
        .unwrap()
        .with_series(Series::new("b", "B", [Sample::new(5.0, 2.0), Sample::new(1.0, 1.0)]))
        .unwrap();
    let plot = CartesianPlot::new()
        .x_scale(NumericScale::new().domain(4.5..=5.5))
        .bar_layout(BarLayout::new().grouping(BarGrouping::Stacked))
        .bar(BarLayer::new("a"))
        .bar(BarLayer::new("b"));
    let geometry = layout(&data, &plot, Edges::default(), bounds());
    assert_eq!(geometry.bars[0].placements[0].sample_index, 2);
    assert_eq!(geometry.bars[1].placements[0].sample_index, 0);
    assert_eq!(geometry.bars[1].placements[0].baseline, geometry.bars[0].placements[0].endpoint);
    assert_eq!(geometry.bars[0].rectangles.len(), 1);
    assert!(!plot.clone().x_scale(NumericScale::new().domain(20.0..=30.0)).has_visible_bars(&data.series()[0]));
}
