use gpui::{App, Bounds, Edges, Pixels, Point, ShapedLine, SharedString, TextAlign, TextRun, Window, font, point, px};

use crate::presentation::{Axis, AxisSide, TextPresentation};
use super::{
    geometry::{CartesianGeometry, TickGeometry},
    line::paint_path,
};

pub(super) struct AxisLayout {
    axis: Axis,
    labels: Vec<ShapedLine>,
    title: Option<ShapedLine>,
}

pub(super) fn positive(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() && value > px(0.0) {
        value
    } else {
        px(0.0)
    }
}

pub(super) fn measure(
    axis: &Axis,
    x_ticks: &[TickGeometry],
    y_ticks: &[TickGeometry],
    window: &mut Window,
) -> AxisLayout {
    let ticks = if axis.side.is_horizontal() { x_ticks } else { y_ticks };
    AxisLayout {
        axis: axis.clone(),
        labels: ticks
            .iter()
            .map(|tick| shape(tick.label.clone().into(), &axis.presentation.tick_labels, window))
            .collect(),
        title: axis.title.as_ref().map(|title| shape(title.clone(), &axis.presentation.title, window)),
    }
}

pub(super) fn shape(text: SharedString, presentation: &TextPresentation, window: &mut Window) -> ShapedLine {
    let run = TextRun {
        len: text.len(),
        font: font(presentation.font_family.clone()),
        color: presentation.color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window.text_system().shape_line(text, presentation.font_size, &[run], None)
}

pub(super) fn insets(axes: &[AxisLayout], padding: Pixels) -> Edges<Pixels> {
    let mut insets = Edges::all(positive(padding));
    for layout in axes {
        let style = &layout.axis.presentation;
        let label_height = positive(style.tick_labels.line_height);
        let slot = fixed_label_width(style);
        let label_width = slot.unwrap_or_else(|| layout.labels.iter().map(ShapedLine::width).max().unwrap_or(px(0.0)));
        let title_height = if layout.title.is_some() {
            positive(style.title.line_height)
        } else {
            px(0.0)
        };
        let title_width = layout.title.as_ref().map(ShapedLine::width).unwrap_or(px(0.0));
        let gap = positive(style.label_gap);
        let tick = positive(style.tick_length);
        if layout.axis.side.is_horizontal() {
            let title_band = if layout.title.is_some() {
                title_height + gap
            } else {
                px(0.0)
            };
            let gutter = positive(style.gutter).max(tick + gap + label_height + title_band);
            match layout.axis.side {
                AxisSide::Top => insets.top = insets.top.max(gutter),
                AxisSide::Bottom => insets.bottom = insets.bottom.max(gutter),
                _ => {}
            }
            // Endpoint tick labels extend beyond the data rectangle.
            if let Some(first) = layout.labels.first() {
                insets.left = insets.left.max(slot.unwrap_or_else(|| first.width()) / 2.0);
            }
            if let Some(last) = layout.labels.last() {
                insets.right = insets.right.max(slot.unwrap_or_else(|| last.width()) / 2.0);
            }
        } else {
            let gutter = positive(style.gutter).max(tick + gap + label_width.max(title_width));
            match layout.axis.side {
                AxisSide::Left => insets.left = insets.left.max(gutter),
                AxisSide::Right => insets.right = insets.right.max(gutter),
                _ => {}
            }
            // Vertical titles head the label column, above the topmost tick label.
            let heading = if layout.title.is_some() {
                title_height + gap
            } else {
                px(0.0)
            };
            insets.top = insets.top.max(heading + label_height / 2.0);
            insets.bottom = insets.bottom.max(label_height / 2.0);
        }
    }
    insets
}

pub(super) fn axis_line(bounds: Bounds<Pixels>, side: AxisSide) -> [Point<Pixels>; 2] {
    match side {
        AxisSide::Top => [bounds.origin, point(bounds.right(), bounds.origin.y)],
        AxisSide::Bottom => [point(bounds.origin.x, bounds.bottom()), point(bounds.right(), bounds.bottom())],
        AxisSide::Left => [bounds.origin, point(bounds.origin.x, bounds.bottom())],
        AxisSide::Right => [point(bounds.right(), bounds.origin.y), point(bounds.right(), bounds.bottom())],
    }
}

fn tick_line(bounds: Bounds<Pixels>, side: AxisSide, position: Pixels, length: Pixels) -> [Point<Pixels>; 2] {
    match side {
        AxisSide::Top => [point(position, bounds.origin.y), point(position, bounds.origin.y - length)],
        AxisSide::Bottom => [point(position, bounds.bottom()), point(position, bounds.bottom() + length)],
        AxisSide::Left => [point(bounds.origin.x, position), point(bounds.origin.x - length, position)],
        AxisSide::Right => [point(bounds.right(), position), point(bounds.right() + length, position)],
    }
}

pub(super) fn paint(
    layout: &AxisLayout,
    geometry: &CartesianGeometry,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let side = layout.axis.side;
    let style = &layout.axis.presentation;
    let ticks = if side.is_horizontal() {
        &geometry.x_ticks
    } else {
        &geometry.y_ticks
    };
    let plot = geometry.plot_bounds;
    let tick_length = positive(style.tick_length);
    let gap = positive(style.label_gap);
    paint_path(axis_line(plot, side), style.stroke, window);
    for (index, (tick, label)) in ticks.iter().zip(&layout.labels).enumerate() {
        paint_path(tick_line(plot, side, tick.position, tick_length), style.stroke, window);
        let slot = fixed_label_width(style);
        let mut origin = label_origin(
            plot,
            side,
            tick,
            slot.unwrap_or_else(|| label.width()),
            positive(style.tick_labels.line_height),
            tick_length + gap,
        );
        let mut align = if slot.is_some() {
            style.label_align
        } else {
            TextAlign::Left
        };
        if style.inset_end_labels
            && side.is_horizontal()
            && let Some(width) = slot
        {
            let forward = ticks.first().zip(ticks.last()).is_none_or(|(first, last)| first.position <= last.position);
            let edge = if ticks.len() <= 1 {
                0
            } else if index == 0 {
                if forward { -1 } else { 1 }
            } else if index + 1 == ticks.len() {
                if forward { 1 } else { -1 }
            } else {
                0
            };
            let (x, alignment) = horizontal_label_slot(tick.position, width, gap, edge);
            origin.x = x;
            align = alignment;
        }
        label.paint(origin, style.tick_labels.line_height, align, slot, window, cx).ok();
    }
    if let Some(title) = &layout.title {
        let height = positive(style.title.line_height);
        let labels_height = positive(style.tick_labels.line_height);
        let origin = match side {
            AxisSide::Bottom => point(
                plot.origin.x + (plot.size.width - title.width()) / 2.0,
                plot.bottom() + tick_length + gap + labels_height + gap,
            ),
            AxisSide::Top => point(
                plot.origin.x + (plot.size.width - title.width()) / 2.0,
                plot.origin.y - tick_length - gap - labels_height - gap - height,
            ),
            AxisSide::Left => point(plot.origin.x - tick_length - gap - title.width(), bounds.origin.y),
            AxisSide::Right => point(plot.right() + tick_length + gap, bounds.origin.y),
        };
        title.paint(origin, height, TextAlign::Left, None, window, cx).ok();
    }
}

fn horizontal_label_slot(position: Pixels, width: Pixels, gap: Pixels, edge: i8) -> (Pixels, TextAlign) {
    match edge {
        -1 => (position + gap, TextAlign::Left),
        1 => (position - gap - width, TextAlign::Right),
        _ => (position - width / 2.0, TextAlign::Center),
    }
}

fn fixed_label_width(style: &crate::AxisPresentation) -> Option<Pixels> {
    style.label_width.filter(|width| f32::from(*width).is_finite() && *width > px(0.0))
}

fn label_origin(
    plot: Bounds<Pixels>,
    side: AxisSide,
    tick: &TickGeometry,
    width: Pixels,
    height: Pixels,
    offset: Pixels,
) -> Point<Pixels> {
    match side {
        AxisSide::Bottom => point(tick.position - width / 2.0, plot.bottom() + offset),
        AxisSide::Top => point(tick.position - width / 2.0, plot.origin.y - offset - height),
        AxisSide::Left => point(plot.origin.x - offset - width, tick.position - height / 2.0),
        AxisSide::Right => point(plot.right() + offset, tick.position - height / 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn axis_edges_and_ticks_point_outward() {
        let bounds = Bounds::new(point(px(10.0), px(20.0)), size(px(100.0), px(80.0)));
        for (side, expected) in [
            (AxisSide::Top, [point(px(50.0), px(20.0)), point(px(50.0), px(16.0))]),
            (AxisSide::Bottom, [point(px(50.0), px(100.0)), point(px(50.0), px(104.0))]),
            (AxisSide::Left, [point(px(10.0), px(50.0)), point(px(6.0), px(50.0))]),
            (AxisSide::Right, [point(px(110.0), px(50.0)), point(px(114.0), px(50.0))]),
        ] {
            assert_eq!(tick_line(bounds, side, px(50.0), px(4.0)), expected);
            let edge = axis_line(bounds, side);
            let expected_edge = match side {
                AxisSide::Top => [point(px(10.0), px(20.0)), point(px(110.0), px(20.0))],
                AxisSide::Bottom => [point(px(10.0), px(100.0)), point(px(110.0), px(100.0))],
                AxisSide::Left => [point(px(10.0), px(20.0)), point(px(10.0), px(100.0))],
                AxisSide::Right => [point(px(110.0), px(20.0)), point(px(110.0), px(100.0))],
            };
            assert_eq!(edge, expected_edge);
        }
    }

    #[test]
    fn fixed_horizontal_slots_align_inward_at_endpoints() {
        let (x, align) = horizontal_label_slot(px(50.0), px(68.0), px(6.0), -1);
        assert_eq!(x, px(56.0));
        assert!(matches!(align, TextAlign::Left));
        let (x, align) = horizontal_label_slot(px(450.0), px(68.0), px(6.0), 1);
        assert_eq!(x + px(68.0), px(444.0));
        assert!(matches!(align, TextAlign::Right));
        let (x, align) = horizontal_label_slot(px(250.0), px(68.0), px(6.0), 0);
        assert_eq!(x + px(34.0), px(250.0));
        assert!(matches!(align, TextAlign::Center));
    }

    #[test]
    fn labels_follow_each_edge() {
        let bounds = Bounds::new(point(px(10.0), px(20.0)), size(px(100.0), px(80.0)));
        let tick = TickGeometry { position: px(50.0), label: "2".into() };
        for (side, expected) in [
            (AxisSide::Top, point(px(40.0), px(-4.0))),
            (AxisSide::Bottom, point(px(40.0), px(108.0))),
            (AxisSide::Left, point(px(-18.0), px(42.0))),
            (AxisSide::Right, point(px(118.0), px(42.0))),
        ] {
            assert_eq!(label_origin(bounds, side, &tick, px(20.0), px(16.0), px(8.0)), expected);
        }
    }
}

#[cfg(test)]
mod fixed_label_tests {
    use super::*;
    #[gpui::test]
    fn fixed_slots_keep_plot_insets_identical_as_proportional_text_changes(cx: &mut gpui::TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| TestView);
        cx.update(|window, _| {
            for side in [AxisSide::Bottom, AxisSide::Left] {
                let mut style = crate::AxisPresentation::new(gpui::black());
                style.label_width = Some(px(68.0));
                style.label_align = TextAlign::Right;
                let axis = Axis::new(side, style);
                let short = [TickGeometry { position: px(0.0), label: "1".into() }];
                let long = [TickGeometry { position: px(0.0), label: "100000".into() }];
                let a = measure(&axis, &short, &short, window);
                let b = measure(&axis, &long, &long, window);
                assert_ne!(a.labels[0].width(), b.labels[0].width());
                assert_eq!(insets(&[a], px(4.0)), insets(&[b], px(4.0)));
            }
        });
    }
    struct TestView;
    impl gpui::Render for TestView {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
            gpui::div()
        }
    }
}
