use std::{cell::RefCell, rc::Rc};

use gpui::{AppContext, TestAppContext, div, px, size};

use super::*;
use crate::controls::resizable_panels::{
    ResizablePanelSpec, ResizeCollapseBehavior, ResizeCollapseDirection, ResizeCollapseMode,
};

fn builder(animated: bool) -> ResizablePanelsBuilder {
    ResizablePanels::horizontal("shell-regression")
        .animated(animated)
        .double_click_collapse(Some(ResizeCollapseBehavior::new(
            ResizeCollapseMode::Completely,
            ResizeCollapseDirection::Left,
        )))
        .panel(ResizablePanelSpec::new_render(div).size(px(260.0)).min(px(200.0)).max(px(480.0)))
        .panel(ResizablePanelSpec::new_render(div).weight(1.0).min(px(380.0)))
}

#[test]
fn toolbar_toggle_restores_double_click_collapse_and_can_hide_again() {
    for animated in [false, true] {
        let app = TestAppContext::single();
        app.update(|cx| {
            let panels = cx.new(|cx| ResizablePanels::from_builder(builder(animated), cx));
            panels.update(cx, |panels, cx| {
                panels.set_measured_size(size(px(1000.0), px(600.0)), cx);
                assert!(panels.apply_pair_delta(0, 80.0, cx));
                panels.refresh_panel_sizes_px();
                let original = panels.panel_sizes_px();
                assert_eq!(original[0], 340.0);
                assert!(panels.apply_double_click_collapse(0, cx));
                if animated {
                    assert_eq!(panels.panel_sizes_px(), original, "collapse must start at the visible width");
                    assert!(panels.transitions[0].is_animating());
                    panels.transitions[0].snap_to(0.0);
                    panels.refresh_panel_sizes_px();
                }
                assert_eq!(panels.panel_sizes_px()[0], 0.0);
                panels.toggle_panel_hidden(0, PanelHideMode::Completely, cx);
                if animated {
                    assert_eq!(panels.panel_sizes_px()[0], 0.0, "restore must start at the collapsed width");
                    assert!(panels.transitions[0].is_animating());
                    panels.transitions[0].snap_to(1.0);
                    panels.refresh_panel_sizes_px();
                }
                assert_eq!(panels.panel_sizes_px(), original);
                assert!(!panels.is_panel_hidden(0));
                panels.toggle_panel_hidden(0, PanelHideMode::Completely, cx);
                assert!(panels.is_panel_hidden(0));
                panels.toggle_panel_hidden(0, PanelHideMode::Completely, cx);
                assert!(!panels.is_panel_hidden(0));
            });
        });
    }
}

#[test]
fn reversing_an_in_flight_panel_transition_preserves_its_visible_width() {
    let app = TestAppContext::single();
    app.update(|cx| {
        let panels = cx.new(|cx| ResizablePanels::from_builder(builder(true), cx));
        panels.update(cx, |panels, cx| {
            panels.set_measured_size(size(px(1000.0), px(600.0)), cx);
            panels.hide_panel(0, PanelHideMode::Completely, cx);
            panels.transitions[0].snap_to(0.5);
            panels.refresh_panel_sizes_px();
            assert_eq!(panels.panel_sizes_px()[0], 130.0);
            panels.show_panel(0, cx);
            assert_eq!(panels.panel_sizes_px()[0], 130.0);
            panels.hide_panel(0, PanelHideMode::Completely, cx);
            assert_eq!(panels.panel_sizes_px()[0], 130.0);
        });
    });
}

#[test]
fn double_click_collapse_and_restore_preserve_the_resized_width() {
    let app = TestAppContext::single();
    app.update(|cx| {
        let panels = cx.new(|cx| ResizablePanels::from_builder(builder(false), cx));
        panels.update(cx, |panels, cx| {
            panels.set_measured_size(size(px(1000.0), px(600.0)), cx);
            assert!(panels.apply_pair_delta(0, 110.0, cx));
            panels.refresh_panel_sizes_px();
            assert!(panels.apply_double_click_collapse(0, cx));
            assert_eq!(panels.panel_sizes_px()[0], 0.0);
            assert!(panels.apply_double_click_collapse(0, cx));
            assert_eq!(panels.panel_sizes_px()[0], 370.0);
        });
    });
}

#[test]
fn parent_resize_publishes_solved_sizes_to_dependent_chrome() {
    let app = TestAppContext::single();
    let sizes = Rc::new(RefCell::new(Vec::new()));
    let (panels, _subscription) = app.update(|cx| {
        let panels = cx.new(|cx| ResizablePanels::from_builder(builder(false), cx));
        let sizes = sizes.clone();
        let subscription = cx.subscribe(&panels, move |_, event: &ResizablePanelsEvent, _| {
            if let ResizablePanelsEvent::SizesChanged { sizes_px } = event {
                sizes.borrow_mut().push(sizes_px.clone());
            }
        });
        (panels, subscription)
    });
    for width in [1000.0, 800.0, 1200.0] {
        app.update(|cx| {
            panels.update(cx, |panels, cx| {
                panels.set_measured_size(size(px(width), px(600.0)), cx);
            })
        });
        let last = sizes.borrow().last().cloned().expect("measurement must publish solved sizes");
        assert_eq!(last, vec![260.0, width - 260.0]);
    }
    let count = sizes.borrow().len();
    app.update(|cx| {
        panels.update(cx, |panels, cx| {
            panels.set_measured_size(size(px(1200.0), px(700.0)), cx);
        })
    });
    assert_eq!(sizes.borrow().len(), count, "cross-axis resizing must not publish unchanged widths");
}
