use super::*;
use gpui::{Focusable, TestAppContext, VisualTestContext, point};

fn setup(app: &mut TestAppContext) -> (Entity<SdkControlsApp>, &mut VisualTestContext) {
    let (view, cx) = app.add_window_view(|window, cx| {
        gpui_luma::init(cx).unwrap();
        gpui_luma::focus::bind_default_focus_keys(cx);
        gpui_luma::key_handling::bind_default_control_keys(cx);
        window.activate_window();
        SdkControlsApp::new(window, cx)
    });
    cx.run_until_parked();
    (view, cx)
}
fn page(view: &Entity<SdkControlsApp>, cx: &mut VisualTestContext) -> Entity<WorkPane> {
    cx.update(|_, cx| view.read(cx).page.clone())
}
fn value(page: &Entity<WorkPane>, cx: &mut VisualTestContext) -> bool {
    cx.update(|_, cx| *page.read(cx).switch.read(cx).data())
}
fn row(page: &Entity<WorkPane>, key: &str, cx: &mut VisualTestContext) -> String {
    cx.update(|_, cx| page.read(cx).inspector.read(cx).rows.iter().find(|row| row.label == key).unwrap().value.clone())
}

#[test]
fn pointer_keyboard_disabled_and_reset_report_real_events_and_inspection() {
    let mut app = TestAppContext::single();
    let (view, cx) = setup(&mut app);
    let page = page(&view, cx);
    assert!(!value(&page, cx));
    let bounds = cx.debug_bounds("sdk-switch-bounds").unwrap();
    cx.simulate_click(point(bounds.left() + px(10.0), bounds.center().y), Default::default());
    cx.run_until_parked();
    assert!(value(&page, cx));
    assert_eq!(row(&page, "switch.value", cx), "true");
    cx.update(|window, cx| assert!(page.read(cx).switch.read(cx).focus_handle(cx).is_focused(window)));
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    assert!(!value(&page, cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(value(&page, cx));
    let sample = cx.update(|_, cx| page.read(cx).sample.clone());
    // Use the real enabled scenario control, rather than injecting a semantic event.
    let enabled = cx.update(|_, cx| sample.read(cx).enabled.clone());
    cx.update(|window, cx| enabled.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    cx.simulate_click(point(bounds.left() + px(10.0), bounds.center().y), Default::default());
    cx.run_until_parked();
    assert!(value(&page, cx), "disabled Switch must ignore activation");
    assert!(row(&page, "switch.state", cx).contains("disabled: true"));
    cx.update(|window, cx| sample.read(cx).reset.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(!value(&page, cx));
    cx.update(|_, cx| {
        let log = page.read(cx).events.read(cx).text();
        assert!(log.contains("SwitchEvent::Change { on: true }"));
        assert!(log.contains("SwitchEvent::Change { on: false }"));
        assert!(log.contains("SwitchEvent::EnabledChanged { enabled: false }"));
        assert!(log.contains("SwitchEvent::FocusChanged"));
        assert!(log.contains("Reset value to false (programmatic; no Change event)"));
    });
}

#[test]
fn mode_change_preserves_entities_focus_scroll_selection_events_and_motion_profile() {
    let mut app = TestAppContext::single();
    let (view, cx) = setup(&mut app);
    let page = page(&view, cx);
    cx.update(|window, _| window.resize(gpui::size(px(1100.0), px(480.0))));
    cx.run_until_parked();
    let (switch, inspector, sample, events, split, sidebar, panels, motion) = cx.update(|window, cx| {
        page.read(cx).switch.read(cx).focus_handle(cx).focus(window, cx);
        let app = view.read(cx);
        let page = page.read(cx);
        (
            page.switch.clone(),
            page.inspector.clone(),
            page.sample.clone(),
            page.events.clone(),
            page.split.clone(),
            app.sidebar.clone(),
            app.panels.clone(),
            format!("{:?}", page.switch.read(cx).motion_profile()),
        )
    });
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    sample.update(cx, |sample, cx| {
        sample.scroll.set_offset(point(px(0.0), px(-25.0)));
        cx.notify();
    });
    split.update(cx, |split, cx| split.set_weights(vec![0.6, 0.4], cx));
    inspector.update(cx, |inspector, cx| {
        inspector.scroll.set_offset(point(px(0.0), px(-50.0)));
        cx.notify();
    });
    cx.run_until_parked();
    let before_color = row(&page, "switch.track.background", cx);
    let before = cx.update(|_, cx| {
        (
            events.read(cx).text().to_owned(),
            sample.read(cx).scroll.offset(),
            split.read(cx).panel_sizes_px(),
            panels.read(cx).panel_sizes_px(),
            inspector.read(cx).scroll.offset(),
        )
    });
    assert!(before.1.y < px(0.0), "exercise nonzero page scroll retention");
    assert!(before.4.y < px(0.0), "exercise nonzero inspector scroll retention");
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        view.update(cx, |app, cx| app.toggle_mode(cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            let app = view.read(cx);
            let page = page.read(cx);
            assert_eq!(app.theme.mode, mode);
            assert_eq!(app.sidebar, sidebar);
            assert_eq!(app.panels, panels);
            assert_eq!(page.switch, switch);
            assert_eq!(page.inspector, inspector);
            assert_eq!(page.sample, sample);
            assert_eq!(page.events, events);
            assert_eq!(page.split, split);
            assert_eq!(page.selected, "switch");
            assert!(*switch.read(cx).data());
            assert!(switch.read(cx).focus_handle(cx).is_focused(window));
            assert_eq!(events.read(cx).text(), before.0);
            assert_eq!(sample.read(cx).scroll.offset(), before.1);
            assert_eq!(split.read(cx).panel_sizes_px(), before.2);
            assert_eq!(panels.read(cx).panel_sizes_px(), before.3);
            assert_eq!(inspector.read(cx).scroll.offset(), before.4);
            assert_eq!(format!("{:?}", switch.read(cx).motion_profile()), motion);
        });
        let expected = NativeTheme::new(mode).switch_theme().resolve(
            true,
            gpui_luma::theme::InteractionState::default(),
            gpui_luma::theme::ControlSize::Md,
        );
        assert_eq!(row(&page, "switch.track.background", cx), format!("{:?}", expected.track_background));
        assert_eq!(row(&page, "switch.mode", cx), format!("{mode:?}"));
    }
    assert_eq!(row(&page, "switch.track.background", cx), before_color);
    // Exercise the actual mode button as well as in-place updates under sample focus.
    let mode_button = cx.update(|_, cx| view.read(cx).mode_toggle.clone());
    cx.update(|window, cx| mode_button.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(view.read(cx).theme.mode, ThemeMode::Dark));
    cx.update(|window, cx| switch.read(cx).focus_handle(cx).focus(window, cx));
    // Tab follows the SDK's focus order; no recreated focus handles.
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.update(|window, cx| assert!(!switch.read(cx).focus_handle(cx).is_focused(window)));
}

#[test]
fn navigation_to_deferred_entry_and_back_retains_switch_and_sidebar_toggle_works() {
    let mut app = TestAppContext::single();
    let (view, cx) = setup(&mut app);
    let page = page(&view, cx);
    let switch = cx.update(|window, cx| {
        let switch = page.read(cx).switch.clone();
        switch.read(cx).focus_handle(cx).focus(window, cx);
        switch
    });
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    // Exercise app navigation through the sidebar semantic event boundary.
    let sidebar = cx.update(|_, cx| view.read(cx).sidebar.clone());
    sidebar.update(cx, |_, cx| cx.emit(SidebarEvent::Select { id: "textfield".into() }));
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(page.read(cx).selected, "textfield"));
    view.update(cx, |app, cx| app.toggle_mode(cx));
    sidebar.update(cx, |_, cx| cx.emit(SidebarEvent::Select { id: "switch".into() }));
    cx.run_until_parked();
    assert!(value(&page, cx));
    cx.update(|_, cx| assert_eq!(page.read(cx).switch, switch));
    let inspector_toggle = cx.update(|_, cx| page.read(cx).sample.read(cx).inspector_toggle.clone());
    cx.update(|window, cx| inspector_toggle.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(page.read(cx).split.read(cx).is_panel_hidden(1)));
    view.update(cx, |app, cx| app.toggle_mode(cx));
    cx.update(|_, cx| assert!(page.read(cx).split.read(cx).is_panel_hidden(1)));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!page.read(cx).split.read(cx).is_panel_hidden(1)));
    let toggle = cx.update(|_, cx| view.read(cx).sidebar_toggle.clone());
    cx.update(|window, cx| toggle.read(cx).focus_handle(cx).focus(window, cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(view.read(cx).panels.read(cx).is_panel_hidden(0)));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!view.read(cx).panels.read(cx).is_panel_hidden(0)));
}

#[test]
fn catalog_matches_documentation_groups_labels_and_order() {
    let navigation = include_str!("../../../../docs/navigation.js");
    let mut position = 0;
    for (group, entries) in catalog::GROUPS {
        let offset = navigation[position..].find(&format!("\"title\": \"{group}\"")).unwrap();
        position += offset;
        for (id, label) in *entries {
            let offset = navigation[position..].find(&format!("\"href\": \"{id}.html\"")).unwrap();
            position += offset;
            let end = navigation[position..].find('}').unwrap() + position;
            assert!(navigation[position..end].contains(&format!("\"label\": \"{label}\"")));
        }
    }
}

#[test]
fn dropping_work_pane_releases_sample_and_split_without_callback_cycles() {
    let app = TestAppContext::single();
    let owner = app.update(|cx| cx.new(|cx| WorkPane::new(NativeTheme::light(), cx)));
    let (sample, split, switch) = app.read(|cx| {
        let owner = owner.read(cx);
        (owner.sample.downgrade(), owner.split.downgrade(), owner.switch.downgrade())
    });
    drop(owner);
    app.update(|_| {});
    assert!(sample.upgrade().is_none());
    assert!(split.upgrade().is_none());
    assert!(switch.upgrade().is_none());
}
