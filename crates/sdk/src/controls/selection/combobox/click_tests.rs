//! Dispatch real popup row clicks, including disabled rows and filtered source indices.
use super::*;
use std::{cell::RefCell, rc::Rc};
use gpui::{Focusable, TestAppContext, div, prelude::*};
use super::super::behavior::SelectionItem;

#[test]
fn disabled_rows_preserve_click_targets_after_filtering() {
    for query in ["", "Match"] {
        for (target, selector_id) in [("a", "option-a"), ("c", "option-c")] {
            let mut app = TestAppContext::single();
            let (selector, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                let builder = ComboBoxBuilder::new(
                    "row-clicks",
                    [
                        SelectionItem::new("other", "Other").enabled(false),
                        SelectionItem::new("a", "Match A"),
                        SelectionItem::new("b", "Match B").enabled(false),
                        SelectionItem::new("d", "Match D").enabled(false),
                        SelectionItem::new("c", "Match C"),
                    ],
                )
                .motion_enabled(false)
                .with_item_template(|model, _| {
                    let id = model.item.id.clone();
                    div()
                        .id(id.clone())
                        .debug_selector(move || format!("option-{id}"))
                        .w_full()
                        .h(px(24.0))
                        .child(model.item.label.clone())
                });
                ComboBoxControl::from_builder(builder, cx)
            });
            cx.run_until_parked();
            cx.update(|window, app| {
                selector.read(app).textfield.read(app).focus_handle(app).focus(window, app);
            });
            selector.update(cx, |selector, cx| selector.open_popup_with_all_items(cx));
            cx.run_until_parked();
            if !query.is_empty() {
                cx.simulate_input(query);
                cx.run_until_parked();
            }
            cx.update(|window, app| {
                let selector = selector.read(app);
                assert!(selector.behavior.state.open);
                assert_eq!(
                    selector.behavior.state.filtered,
                    if query.is_empty() {
                        vec![0, 1, 2, 3, 4]
                    } else {
                        vec![1, 2, 3, 4]
                    }
                );
                assert!(selector.textfield.read(app).focus_handle(app).is_focused(window));
            });

            let events = Rc::new(RefCell::new(Vec::new()));
            let capture = events.clone();
            let _subscription = cx.update(|_, app| {
                app.subscribe(&selector, move |_, event: &ComboBoxEvent, _| {
                    capture.borrow_mut().push(match event {
                        ComboBoxEvent::Select { item_id, label } => format!("select:{item_id}:{label}"),
                        ComboBoxEvent::Complete { .. } => "complete".into(),
                        ComboBoxEvent::Clear => "clear".into(),
                        ComboBoxEvent::OpenChanged { open } => format!("open:{open}"),
                        ComboBoxEvent::Dismiss => "dismiss".into(),
                        _ => return,
                    });
                })
            });
            for disabled in ["option-b", "option-d"] {
                let bounds = cx.debug_bounds(disabled).unwrap();
                cx.simulate_click(bounds.center(), Default::default());
                cx.run_until_parked();
                cx.update(|_, app| {
                    assert!(selector.read(app).behavior.state.open);
                    assert_eq!(selector.read(app).committed_selection, None);
                });
                assert!(events.borrow().is_empty());
            }
            let bounds = cx.debug_bounds(selector_id).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
            cx.update(|window, app| {
                let selector = selector.read(app);
                assert!(!selector.behavior.state.open, "{query:?}, target={target}");
                let selected = selector.committed_selection.unwrap();
                assert_eq!(selector.model.items[selected].id.as_ref(), target);
                assert!(selector.textfield.read(app).focus_handle(app).is_focused(window));
            });
            let label = if target == "a" { "Match A" } else { "Match C" };
            assert_eq!(*events.borrow(), ["open:false".to_string(), format!("select:{target}:{label}")]);
        }
    }
}
