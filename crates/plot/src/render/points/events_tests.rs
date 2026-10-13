use super::*;
use crate::{
    AreaLayer, CartesianChartBuilder, ChartMetrics, Dataset, LegendPresentation, LineLayer, LinePresentation,
    MarkerPresentation, MarkerVisibility, NumericScale, PlotPointEvent, Sample, Series,
};
use gpui::{Context, IntoElement, Render, TestAppContext};
use std::{cell::RefCell, rc::Rc};

struct EventHarness {
    area: bool,
    mode: MarkerVisibility,
    events: Rc<RefCell<Vec<PlotPointEvent>>>,
}

impl Render for EventHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, 0.0), Sample::new(1.0, 1.0), Sample::new(2.0, 0.0)]))
            .expect("A")
            .with_series(Series::new("b", "B", [Sample::new(0.0, 0.5), Sample::new(1.0, 1.5), Sample::new(2.0, 0.5)]))
            .expect("B");
        let mut line = LinePresentation::new(gpui::black());
        line.markers = Some(MarkerPresentation::new(gpui::black(), px(4.0)).visibility(self.mode));
        let events = self.events.clone();
        let mut chart = CartesianChartBuilder::new(data)
            .presentation(CartesianPresentation::new(crate::AxisPresentation::new(gpui::black()), line))
            .axes([])
            .legend(LegendPresentation { visible: false, text_color: gpui::black() })
            .metrics(ChartMetrics { plot_padding: px(0.0), ..ChartMetrics::default() })
            .x_scale(NumericScale::new().domain(0.0..=2.0))
            .y_scale(NumericScale::new().domain(0.0..=2.0))
            .on_point_event(move |event, _| events.borrow_mut().push(event));
        for id in ["a", "b"] {
            chart = if self.area {
                chart.area(AreaLayer::new(id))
            } else {
                chart.line(LineLayer::new(id))
            };
        }
        // Intentionally no tooltip: control input events must be independent of it.
        div().id("event-test").w(px(300.0)).h(px(200.0)).child(chart.build())
    }
}

#[gpui::test]
fn controls_emit_hover_changes_leave_and_repeated_picks_without_tooltips(cx: &mut TestAppContext) {
    for area in [false, true] {
        for mode in [MarkerVisibility::Always, MarkerVisibility::Hover, MarkerVisibility::HoverSlice] {
            let events = Rc::new(RefCell::new(Vec::new()));
            let (_, cx) = cx.add_window_view(|_, _| EventHarness { area, mode, events: events.clone() });
            cx.update(|window, _| window.activate_window());
            let draw_at = |cx: &mut gpui::VisualTestContext, position| {
                cx.update(|window, cx| {
                    // The mock platform resets the pointer during bounds updates.
                    window.simulate_mouse_move(position, cx);
                    window.draw(cx).clear(cx);
                });
                cx.run_until_parked();
            };
            draw_at(cx, point(px(400.0), px(250.0)));
            events.borrow_mut().clear();
            draw_at(cx, point(px(150.0), px(100.0)));
            {
                let events = events.borrow();
                assert_eq!(events.len(), 1);
                let PlotPointEvent::HoverChanged { points } = &events[0] else {
                    panic!("hover event")
                };
                assert_eq!(points.len(), if mode == MarkerVisibility::HoverSlice { 2 } else { 1 });
                assert_eq!(points[0].series, "a".into());
                assert_eq!(points[0].sample, Sample::new(1.0, 1.0));
                if mode == MarkerVisibility::HoverSlice {
                    assert_eq!(points[1].sample, Sample::new(1.0, 1.5));
                }
            }
            draw_at(cx, point(px(150.0), px(100.0)));
            assert_eq!(events.borrow().len(), 1, "redrawing an unchanged target must not emit");
            draw_at(cx, point(px(160.0), px(100.0)));
            assert_eq!(events.borrow().len(), 1, "moving inside the same target must not emit");
            for _ in 0..2 {
                cx.simulate_click(point(px(150.0), px(100.0)), gpui::Modifiers::default());
            }
            {
                let events = events.borrow();
                let picks: Vec<_> = events
                    .iter()
                    .filter_map(|event| match event {
                        PlotPointEvent::Select { points } => Some(points),
                        _ => None,
                    })
                    .collect();
                assert_eq!(picks.len(), 2, "each click emits a pick");
                for points in picks {
                    assert_eq!(points.len(), if mode == MarkerVisibility::HoverSlice { 2 } else { 1 });
                    assert_eq!(points[0].sample, Sample::new(1.0, 1.0));
                }
            }
            // Re-establish hover after the mock's click/bounds updates, then leave.
            draw_at(cx, point(px(150.0), px(100.0)));
            events.borrow_mut().clear();
            draw_at(cx, point(px(400.0), px(250.0)));
            assert!(
                matches!(events.borrow().as_slice(), [PlotPointEvent::HoverChanged { points }] if points.is_empty())
            );
            draw_at(cx, point(px(400.0), px(250.0)));
            assert_eq!(events.borrow().len(), 1);
            events.borrow_mut().clear();
            if mode != MarkerVisibility::HoverSlice {
                cx.simulate_click(point(px(20.0), px(10.0)), gpui::Modifiers::default());
                assert!(!events.borrow().iter().any(|event| matches!(event, PlotPointEvent::Select { .. })));
            } else {
                draw_at(cx, point(px(160.0), px(10.0)));
                assert!(
                    matches!(events.borrow().as_slice(), [PlotPointEvent::HoverChanged { points }] if points.len() == 2)
                );
                cx.simulate_click(point(px(160.0), px(10.0)), gpui::Modifiers::default());
                assert!(events.borrow().iter().any(|event| matches!(event, PlotPointEvent::Select { points } if points.len() == 2 && points[0].sample.x == 1.0)));
            }
        }
    }
}
