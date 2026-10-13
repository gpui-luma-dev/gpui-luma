use crate::*;
use gpui::{Context, Render, Window, IntoElement, TestAppContext, div, px, point, prelude::*};
use std::{rc::Rc, cell::RefCell};
struct Harness {
    events: Rc<RefCell<Vec<PieSliceEvent>>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, 1.0), Sample::new(1.0, 1.0)]))
            .unwrap()
            .with_series(Series::new("b", "B", [Sample::new(0.0, 1.0), Sample::new(1.0, 1.0)]))
            .unwrap();
        let events = self.events.clone();
        div().w(px(400.0)).h(px(400.0)).child(
            PieChartBuilder::new(data, "a")
                .outer_radius(0.5)
                .ring("b", PieRingPresentation::new(0.7, 1.0))
                .ring_active_slice("b", Some(0))
                .on_slice_event(move |event, _| events.borrow_mut().push(event))
                .build(),
        )
    }
}
#[gpui::test]
fn primary_and_added_ring_clicks_report_distinct_identity(cx: &mut TestAppContext) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| Harness { events: events.clone() });
    for (pointer, series, ring) in
        [(point(px(230.0), px(170.0)), "a", None), (point(px(230.0), px(50.0)), "b", Some(0))]
    {
        events.borrow_mut().clear();
        cx.update(|window, cx| {
            window.simulate_mouse_move(pointer, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_click(pointer, gpui::Modifiers::default());
        let selected: Vec<_> = events
            .borrow()
            .iter()
            .filter_map(|event| {
                if let PieSliceEvent::Select { slice } = event {
                    Some(slice.clone())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].series, SeriesId::from(series));
        assert_eq!(selected[0].ring_index, ring);
    }
}
