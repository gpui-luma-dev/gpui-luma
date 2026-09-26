//! Headless GPUI TestWindow tests; no platform application is launched.
use std::{cell::RefCell, rc::Rc};

use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, point};

use super::*;

#[derive(Clone)]
struct Row {
    id: usize,
    enabled: bool,
}

fn rows(count: usize) -> Vec<Row> {
    (0..count).map(|id| Row { id, enabled: id != 3 }).collect()
}

fn setup(
    app: &mut TestAppContext,
    count: usize,
    mode: TableSelectionMode,
) -> (Entity<TableControl<Row>>, &mut VisualTestContext) {
    let (table, cx) = app.add_window_view(|_, cx| {
        let mut table = TableControl::from_builder(
            TableBuilder::new_typed("test-table")
                .items(rows(count))
                .row_label(|row| row.id.to_string())
                .row_enabled(|row| row.enabled)
                .selection_mode(mode)
                .visible_rows(5)
                .visible_row_height(30.0_f32),
            cx,
        );
        table.set_row_key(|row| row.id.to_string(), cx).unwrap();
        table
    });
    cx.update(|window, app| {
        window.activate_window();
        table.read(app).focus_handle.clone().focus(window, app);
    });
    cx.run_until_parked();
    (table, cx)
}

#[test]
fn pointer_ranges_toggle_and_disabled_rows() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 12, TableSelectionMode::Extended);
    let click = |cx: &mut VisualTestContext, index: usize, modifiers| {
        cx.simulate_click(point(px(20.0), px(index as f32 * 30.0 + 15.0)), modifiers);
    };
    click(cx, 1, Modifiers::default());
    click(cx, 4, Modifiers { shift: true, ..Default::default() });
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[1, 2, 4]));
    // Shrinking a range retains its original anchor and excludes disabled rows.
    click(cx, 2, Modifiers { shift: true, ..Default::default() });
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[1, 2]));
    click(cx, 0, Modifiers { control: true, ..Default::default() });
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[0, 1, 2]));
    click(cx, 1, Modifiers { platform: true, ..Default::default() });
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[0, 2]));
    click(cx, 3, Modifiers::default());
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[0, 2]));
    click(cx, 4, Modifiers::default());
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[4]));
}

#[test]
fn keyboard_ranges_select_all_and_virtualized_navigation() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 100, TableSelectionMode::Extended);
    // Install application defaults too: Ctrl+A must not be stolen by Selector's
    // legacy first-row binding.
    cx.update(|_, app| crate::key_handling::bind_default_control_keys(app));
    cx.simulate_keystrokes("down space shift-down shift-down");
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[1, 2, 4]));
    cx.simulate_keystrokes("shift-up");
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[1, 2]));
    cx.simulate_keystrokes("ctrl-end ctrl-space");
    cx.update(|_, app| {
        assert_eq!(table.read(app).selected_indices(), &[1, 2, 99]);
        assert!(table.read(app).list_state.logical_scroll_top().item_ix > 0);
    });
    cx.simulate_keystrokes("ctrl-shift-home");
    cx.update(|_, app| {
        assert_eq!(table.read(app).selected_indices().len(), 99);
        assert_eq!(table.read(app).active_index(), Some(0));
    });
    cx.simulate_keystrokes("ctrl-shift-a");
    cx.update(|_, app| assert!(table.read(app).selected_indices().is_empty()));
    cx.simulate_keystrokes("cmd-a");
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices().len(), 99));
    cx.simulate_keystrokes("cmd-shift-a");
    cx.update(|_, app| assert!(table.read(app).selected_indices().is_empty()));
}

#[test]
fn keyed_reorder_removal_and_mode_changes_emit_final_state() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 10, TableSelectionMode::Extended);
    table.update(cx, |table, cx| {
        table.set_selected_keys(["1", "4"], cx).unwrap();
        table.set_active_index(Some(4), cx);
        table.selection_anchor = Some(1);
    });
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(&table, move |_, event: &TableEvent, _| events.borrow_mut().push(event.clone()))
            .detach();
    });
    table.update(cx, |table, cx| {
        table.set_items(rows(10).into_iter().rev(), cx).unwrap();
        assert_eq!(table.selected_keys(), vec![SharedString::from("4"), SharedString::from("1")]);
        assert_eq!(table.active_index(), Some(5));
        assert_eq!(table.selection_anchor, Some(8));
    });
    cx.run_until_parked();
    assert_eq!(
        events.borrow().iter().filter(|event| matches!(event, TableEvent::SelectionChanged { .. })).count(),
        1
    );
    assert!(!events.borrow().iter().any(|event| matches!(event, TableEvent::SelectedKeysChanged { .. })));
    events.borrow_mut().clear();
    table.update(cx, |table, cx| {
        table.set_items(rows(4), cx).unwrap();
        assert_eq!(table.selected_keys(), vec![SharedString::from("1")]);
        table.set_selection_mode(TableSelectionMode::None, cx);
        assert!(table.selected_indices().is_empty());
    });
    cx.run_until_parked();
    let keyed: Vec<_> = events
        .borrow()
        .iter()
        .filter_map(|event| match event {
            TableEvent::SelectedKeysChanged { selected_keys } => Some(selected_keys.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(keyed, vec![vec![SharedString::from("1")], vec![]]);
}

#[test]
fn owner_updates_preserve_focus_scroll_active_and_reject_invalid_keys() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 100, TableSelectionMode::Extended);
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    cx.update(|window, app| {
        table.update(app, |table, cx| {
            let offset = table.list_state.logical_scroll_top();
            table.set_selected_keys(["1", "2", "2"], cx).unwrap();
            assert_eq!(table.selected_indices(), &[1, 2]);
            assert_eq!(table.active_index(), Some(99));
            assert!(table.focus_handle.is_focused(window));
            assert_eq!(table.list_state.logical_scroll_top().item_ix, offset.item_ix);
            assert_eq!(table.list_state.logical_scroll_top().offset_in_item, offset.offset_in_item);
            assert_eq!(table.set_selected_keys(["1", "missing"], cx), Err(TableSelectionError::UnknownKey));
            assert_eq!(table.set_selected_keys(["3"], cx), Err(TableSelectionError::DisabledRow));
            assert_eq!(table.set_items(vec![rows(1).remove(0); 2], cx), Err(TableSelectionError::DuplicateKey));
            assert_eq!(table.items().len(), 100);
            assert_eq!(table.selected_indices(), &[1, 2]);
            assert_eq!(table.list_state.logical_scroll_top().item_ix, offset.item_ix);
            assert_eq!(table.list_state.logical_scroll_top().offset_in_item, offset.offset_in_item);
            table.set_selection_mode(TableSelectionMode::Single, cx);
            assert_eq!(table.selected_indices(), &[1]);
            assert_eq!(table.set_selected_keys(["1", "2"], cx), Err(TableSelectionError::TooManySelected));
            table.set_selection_mode(TableSelectionMode::None, cx);
            assert_eq!(table.set_selected_keys(["1"], cx), Err(TableSelectionError::SelectionDisabled));
        });
    });
}

#[test]
fn empty_single_multiple_and_disabled_tables() {
    for count in [0, 1, 5] {
        for mode in [TableSelectionMode::None, TableSelectionMode::Single, TableSelectionMode::Multiple] {
            let mut app = TestAppContext::single();
            let (table, cx) = setup(&mut app, count, mode);
            cx.simulate_keystrokes("space down space");
            table.update(cx, |table, cx| {
                let expected: &[usize] = match (count, mode) {
                    (0, _) | (_, TableSelectionMode::None) => &[],
                    (1, TableSelectionMode::Multiple) => &[],
                    (1, _) => &[0],
                    (_, TableSelectionMode::Multiple) => &[0, 1],
                    _ => &[1],
                };
                assert_eq!(table.selected_indices(), expected);
                table.set_enabled(false, cx);
            });
            let before =
                cx.update(|_, app| (table.read(app).selected_indices().to_vec(), table.read(app).active_index()));
            cx.simulate_keystrokes("down space ctrl-a shift-end");
            cx.update(|_, app| {
                assert_eq!(table.read(app).selected_indices(), before.0);
                assert_eq!(table.read(app).active_index(), before.1);
            });
        }
    }
}

#[test]
fn range_crosses_pages_and_owner_feedback_is_a_noop() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 12, TableSelectionMode::Extended);
    table.update(cx, |table, cx| {
        table.set_scroll_mode(TableScrollMode::Paged { page_size: 5 }, cx);
        table.select_index(1, false, false, cx);
    });
    cx.simulate_keystrokes("shift-end");
    table.update(cx, |table, cx| {
        assert_eq!(table.current_page(), 2);
        assert_eq!(table.selected_indices(), &[1, 2, 4, 5, 6, 7, 8, 9, 10, 11]);
        table.set_items(rows(12).into_iter().rev(), cx).unwrap();
        // The anchor follows key 1 to index 10; the active key 11 moves to 0.
        assert_eq!(table.selection_anchor, Some(10));
        table.select_index(9, false, true, cx);
        assert_eq!(table.selected_keys(), vec![SharedString::from("2"), SharedString::from("1")]);
        assert_eq!(table.current_page(), 1);
    });
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(&table, move |_, event: &TableEvent, _| events.borrow_mut().push(event.clone()))
            .detach();
    });
    table.update(cx, |table, cx| {
        let keys = table.selected_keys();
        table.set_selected_keys(keys, cx).unwrap();
        table.set_selected_indices([10, 9, 10], cx);
        assert_eq!(table.selection_anchor, Some(10));
        assert_eq!(table.current_page(), 1);
    });
    cx.run_until_parked();
    assert!(
        !events
            .borrow()
            .iter()
            .any(|event| matches!(event, TableEvent::SelectionChanged { .. } | TableEvent::SelectedKeysChanged { .. }))
    );
}

#[test]
fn replacement_disables_selected_rows_and_validates_key_configuration() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 5, TableSelectionMode::Multiple);
    table.update(cx, |table, cx| {
        table.set_selected_keys(["1", "4"], cx).unwrap();
        assert_eq!(table.set_row_key(|_| "duplicate", cx), Err(TableSelectionError::DuplicateKey));
        assert_eq!(table.selected_keys(), vec![SharedString::from("1"), SharedString::from("4")]);
        let mut replacement = rows(5);
        replacement[1].enabled = false;
        table.set_items(replacement, cx).unwrap();
        assert_eq!(table.selected_keys(), vec![SharedString::from("4")]);
    });
}

#[test]
fn checkbox_style_table_keeps_keyboard_selection() {
    let mut app = TestAppContext::single();
    let (table, cx) = setup(&mut app, 5, TableSelectionMode::Multiple);
    table.update(cx, |table, _| table.model.select_on_row_click = false);
    cx.simulate_click(point(px(20.0), px(45.0)), Modifiers::default());
    cx.update(|_, app| {
        assert!(table.read(app).selected_indices().is_empty());
        assert_eq!(table.read(app).active_index(), Some(1));
    });
    cx.simulate_keystrokes("space down space");
    cx.update(|_, app| assert_eq!(table.read(app).selected_indices(), &[1, 2]));
}
