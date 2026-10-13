use super::*;
use crate::{Series, Sample};
use gpui::{size, TestAppContext, Render, Context};

fn data(samples: impl Into<std::sync::Arc<[Sample]>>) -> Dataset {
    Dataset::new().with_series(Series::new("a", "A", samples)).unwrap()
}
#[test]
fn weights_filter_invalid_values_and_preserve_indices_names_and_source() {
    let data = data([
        Sample::new(0.0, 3.0),
        Sample::new(1.0, 0.0),
        Sample::new(2.0, -2.0),
        Sample::new(f64::NAN, 4.0),
        Sample::new(4.0, 1.0),
        Sample::new(5.0, f64::INFINITY),
    ]);
    let names: Vec<SharedString> = ["First", "Zero", "Negative", "Invalid", "Last"].map(Into::into).to_vec();
    let slices = slices(&data, &PiePlot::new("a"), &names);
    assert_eq!(slices.len(), 2);
    assert_eq!(slices[0].sample_index, 0);
    assert_eq!(slices[1].sample_index, 4);
    assert_eq!(slices[1].label, "Last");
    assert_eq!(slices[0].fraction, 0.75);
    assert_eq!(slices[1].fraction, 0.25);
    assert_eq!(data.series()[0].samples().len(), 6);
    assert!(super::slices(&data, &PiePlot::new("absent"), &[]).is_empty());
}
#[test]
fn extreme_weights_normalize_without_overflow_and_empty_data_is_safe() {
    let data = data([Sample::new(0.0, f64::MAX), Sample::new(1.0, f64::MAX)]);
    let slices = slices(&data, &PiePlot::new("a"), &[]);
    assert_eq!(slices[0].fraction, 0.5);
    assert_eq!(slices[1].fraction, 0.5);
    let empty = super::slices(&Dataset::new(), &PiePlot::new("a"), &[]);
    assert!(empty.is_empty());
    assert!(sector_angles(&empty, &PiePlot::new("a")).is_empty());
}
#[test]
fn angles_follow_rotation_direction_and_close_the_full_circle() {
    let data = data([Sample::new(0.0, 275.0), Sample::new(1.0, 200.0), Sample::new(2.0, 90.0)]);
    let slices = slices(&data, &PiePlot::new("a"), &[]);
    for clockwise in [false, true] {
        let plot = PiePlot::new("a").start_angle(-90.0).clockwise(clockwise);
        let angles = sector_angles(&slices, &plot);
        assert_eq!(plot.start_angle_degrees(), 270.0);
        assert!((angles[0].0 - 270_f64.to_radians()).abs() < 1e-12);
        let sign = if clockwise { 1.0 } else { -1.0 };
        assert!(angles.iter().all(|(_, sweep)| sweep * sign > 0.0));
        assert!((angles.iter().map(|(_, sweep)| sweep).sum::<f64>() - TAU * sign).abs() < 1e-12);
        assert_eq!(PiePlot::new("a").start_angle(f64::NAN).start_angle_degrees(), 0.0);
    }
}
fn covers_origin(triangle: &[gpui::PathVertex<Pixels>]) -> bool {
    let p: Vec<_> = triangle.iter().map(|v| (f32::from(v.xy_position.x), f32::from(v.xy_position.y))).collect();
    let cross = |a: (f32, f32), b: (f32, f32)| a.0 * b.1 - a.1 * b.0;
    let edges = [cross(p[0], p[1]), cross(p[1], p[2]), cross(p[2], p[0])];
    edges.iter().all(|x| *x >= 0.0) || edges.iter().all(|x| *x <= 0.0)
}
#[test]
fn full_circle_and_donut_paths_are_finite_and_hole_is_not_filled() {
    for sweep in [TAU, -TAU, FRAC_PI_2, -FRAC_PI_2] {
        for inner in [0.0, 60.0] {
            let path = sector_path(Point::default(), px(100.0), px(inner), 0.0, sweep).unwrap();
            assert!(!path.vertices.is_empty());
            assert!(
                path.vertices
                    .iter()
                    .all(|v| f32::from(v.xy_position.x).is_finite() && f32::from(v.xy_position.y).is_finite())
            );
            if inner > 0.0 {
                assert!(!path.vertices.as_chunks::<3>().0.iter().any(|triangle| covers_origin(triangle)));
            }
        }
    }
    assert!(sector_path(Point::default(), px(0.0), px(0.0), 0.0, TAU).is_none());
}
struct Empty;
#[test]
fn hit_testing_excludes_holes_detached_band_gaps_and_hidden_sectors() {
    for direction in [-1.0, 1.0] {
        let layout = PieLayout {
            center: Point::default(),
            inner: px(40.0),
            sectors: vec![
                Sector {
                    index: 3,
                    start: 0.0,
                    sweep: direction * FRAC_PI_2,
                    radius: px(120.0),
                    gap: Some((px(100.0), px(108.0))),
                },
                Sector { index: 4, start: direction * FRAC_PI_2, sweep: 0.0, radius: px(100.0), gap: None },
            ],
            text: Vec::new(),
        };
        let pointer = |radius| polar(layout.center, px(radius), direction * FRAC_PI_2 / 2.0);
        assert_eq!(hit(&layout, pointer(20.0)), None);
        assert_eq!(hit(&layout, pointer(80.0)), Some(3));
        assert_eq!(hit(&layout, pointer(104.0)), None);
        assert_eq!(hit(&layout, pointer(114.0)), Some(3));
        assert_eq!(hit(&layout, pointer(130.0)), None);
        assert_eq!(hit(&layout, polar(layout.center, px(80.0), direction * 3.0 * FRAC_PI_2 / 2.0)), None);
    }
}
#[gpui::test]
fn concentric_series_normalize_independently_and_reveal_in_order(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    cx.update(|window, _| {
        let data = data([Sample::new(0.0, 3.0), Sample::new(1.0, 1.0)])
            .with_series(Series::new("b", "B", [Sample::new(0.0, 1.0), Sample::new(1.0, 3.0)]))
            .unwrap();
        let primary = slices(&data, &PiePlot::new("a"), &[]);
        let ring = slices(&data, &PiePlot::new("b"), &[]);
        assert_eq!(primary[0].fraction, 0.75);
        assert_eq!(ring[0].fraction, 0.25);
        assert_eq!(ring[0].series, crate::SeriesId::from("b"));
        let bounds = Bounds::new(Point::default(), size(px(300.0), px(300.0)));
        let content = PieContent { expansion: None, progress: 0.5, labels: None, active: None, center: None };
        let inner = layout(
            bounds,
            &primary,
            &PiePlot::new("a"),
            &PiePresentation { outer_radius: 0.65, ..Default::default() },
            &content,
            window,
        );
        let outer = layout(
            bounds,
            &ring,
            &PiePlot::new("b"),
            &PiePresentation { inner_radius: 0.78, ..Default::default() },
            &content,
            window,
        );
        assert_eq!(inner.center, outer.center);
        assert!(inner.sectors[0].radius < outer.inner);
        assert!((inner.sectors[0].sweep.abs() - TAU / 2.0).abs() < 1e-12);
        assert_eq!(inner.sectors[1].sweep, 0.0);
        assert!((outer.sectors.iter().map(|sector| sector.sweep.abs()).sum::<f64>() - TAU / 2.0).abs() < 1e-12);
    });
}
impl Render for Empty {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
    }
}
#[gpui::test]
fn donut_layout_keeps_hole_fixed_when_active_slice_expands(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    cx.update(|window, _| {
        let plot = PiePlot::new("a");
        let slices = slices(&data([Sample::new(0.0, 3.0), Sample::new(1.0, 2.0)]), &plot, &[]);
        let style = PiePresentation { inner_radius: 0.6, ..PiePresentation::default() };
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(300.0), px(260.0)));
        let plain = layout(
            bounds,
            &slices,
            &plot,
            &style,
            &PieContent { expansion: None, progress: 1.0, labels: None, active: None, center: None },
            window,
        );
        let active = layout(
            bounds,
            &slices,
            &plot,
            &style,
            &PieContent { expansion: None, progress: 1.0, labels: None, active: Some(0), center: None },
            window,
        );
        assert_eq!(plain.inner, active.inner);
        assert_eq!(active.sectors[0].radius, plain.sectors[0].radius + style.active_expansion);
        assert_eq!(active.sectors[1].radius, plain.sectors[1].radius);
        let tiny = layout(
            Bounds::new(Point::default(), size(px(0.0), px(0.0))),
            &slices,
            &plot,
            &style,
            &PieContent { expansion: None, progress: 1.0, labels: None, active: Some(0), center: None },
            window,
        );
        assert!(tiny.sectors.iter().all(|sector| sector.radius == px(0.0)));
    });
}
