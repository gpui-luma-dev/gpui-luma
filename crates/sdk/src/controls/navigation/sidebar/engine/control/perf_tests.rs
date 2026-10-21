//! Real handler binding and projection work through GPUI's headless test platform.
use super::*;
use crate::perf_measure::measure;
use gpui::prelude::*;

#[test]
#[ignore = "manual headless performance measurement"]
fn measure_sidebar_hot_paths() {
    for count in [100, 10_000] {
        let mut app = gpui::TestAppContext::single();
        let (engine, cx) = app.add_window_view(|_, cx| {
            SidebarPanelEngine::from_builder(
                super::super::super::SidebarPanelEngineBuilder::new("perf-sidebar")
                    .motion_enabled(false)
                    .items((0..count).map(|index| NavNode::new(format!("row-{index}")).label("Row"))),
                cx,
            )
        });
        cx.run_until_parked();
        cx.update(|window, app| {
            engine.update(app, |view, cx| {
                let mut model = view.render_model(window, cx);
                let mut signature = Vec::new();
                handler_signature(&model.nodes, &mut signature);
                measure(&format!("Sidebar/signature hit/{count}"), 100, || {
                    assert!(signature_matches(&[&model.nodes], &signature));
                });
                model.nodes.last_mut().unwrap().id = "replacement".into();
                measure(&format!("Sidebar/signature late miss/{count}"), 100, || {
                    assert!(!signature_matches(&[&model.nodes], &signature));
                });
                measure(&format!("Sidebar/handlers cold/{count}"), 5, || {
                    *view.row_handlers.borrow_mut() = None;
                    view.template_handlers(&model, cx)
                });
                measure(&format!("Sidebar/handlers warm/{count}"), 100, || view.template_handlers(&model, cx));
                measure(&format!("Sidebar/model+handlers warm/{count}"), 5, || {
                    let model = view.render_model(window, cx);
                    view.template_handlers(&model, cx)
                });
            });
        });
        measure(&format!("Sidebar/headless redraw/{count}"), 3, || {
            engine.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
        });
    }

    for count in [10, 100] {
        let mut app = gpui::TestAppContext::single();
        let (engine, cx) = app.add_window_view(|_, cx| {
            SidebarPanelEngine::from_builder(
                super::super::super::SidebarPanelEngineBuilder::new("perf-rail")
                    .motion_enabled(false)
                    .collapsed(true)
                    .item(
                        NavNode::new("branch")
                            .icon(lucide_svg_static::Icon::Search)
                            .children((0..count).map(|index| NavNode::new(format!("child-{index}")).label("Child"))),
                    ),
                cx,
            )
        });
        cx.run_until_parked();
        cx.update(|window, app| {
            engine.update(app, |view, cx| {
                view.set_open_rail_submenu("branch".into(), cx);
                let model = view.render_model(window, cx);
                assert_eq!(model.rail_submenu.as_ref().unwrap().items.len(), count);
                measure(&format!("Sidebar/rail submenu handlers/{count}"), 100, || view.template_handlers(&model, cx));
                measure(&format!("Sidebar/rail model+handlers/{count}"), 100, || {
                    let model = view.render_model(window, cx);
                    view.template_handlers(&model, cx)
                });
            });
        });
    }
}
