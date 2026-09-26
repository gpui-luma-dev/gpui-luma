//! Positioning assertions use measured TestWindow geometry, without a GUI launch.
use super::*;
use gpui::{Entity, TestAppContext, VisualTestContext, point, ScrollDelta};
use std::sync::Arc;

struct Page {
    tree: Entity<TreeViewControl<usize>>,
    height: f32,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.0)).h(px(self.height)).child(self.tree.clone())
    }
}
struct Heights;
impl super::super::TreeViewTemplate<usize> for Heights {
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: gpui::AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        super::super::default_tree_view_template::<usize>().render(model, body, window, cx)
    }
    fn render_node(
        &self,
        node: &FlatTreeNode<'_, usize>,
        _: TreeViewTemplateHandlers,
        _: &mut Window,
        _: &mut App,
    ) -> gpui::AnyElement {
        div().h(px(*node.data as f32 * node.row_height_factor)).child(node.label.clone()).into_any_element()
    }
}
fn node(i: usize, height: usize) -> TreeNode<usize> {
    TreeNode::new(i.to_string(), i.to_string(), height)
}
fn setup(
    app: &mut TestAppContext,
    varied: bool,
) -> (Entity<Page>, &mut VisualTestContext, Entity<TreeViewControl<usize>>) {
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let tree = TreeViewBuilder::new("position-tree")
            .items((0..100).map(|i| node(i, if varied { 20 + (i % 4) * 17 } else { 24 })))
            .template(Arc::new(Heights))
            .animated(false)
            .spawn(cx);
        Page { tree, height: 120.0 }
    });
    cx.run_until_parked();
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    (page, cx, tree)
}
fn frame(cx: &mut VisualTestContext, ms: u64) {
    cx.executor().advance_clock(std::time::Duration::from_millis(ms));
    cx.update(|window, app| {
        window.simulate_next_frame(app);
    });
    cx.run_until_parked();
}
fn settle(cx: &mut VisualTestContext) {
    for _ in 0..16 {
        frame(cx, 25);
    }
}
fn bounds(tree: &TreeViewControl<usize>, id: &str) -> gpui::Bounds<gpui::Pixels> {
    tree.list_state
        .bounds_for_item(tree.flat_index_for_id(&id.into()).unwrap())
        .expect("measured target")
}
fn centered(tree: &TreeViewControl<usize>, id: &str) {
    assert!((bounds(tree, id).center().y - tree.list_state.viewport_bounds().center().y).abs() <= px(0.6));
    assert!(tree.position.is_none(), "request must finish");
}

#[test]
fn immediate_positioning_uses_measured_bounds_and_clamps_without_selection_or_focus() {
    for varied in [false, true] {
        let mut app = TestAppContext::single();
        let (_, cx, tree) = setup(&mut app, varied);
        tree.update(cx, |tree, cx| {
            tree.select_node_by_id("1", cx);
            assert!(tree.scroll_to("80", cx));
        });
        settle(cx);
        cx.update(|window, app| {
            let tree = tree.read(app);
            assert!((bounds(tree, "80").bottom() - tree.list_state.viewport_bounds().bottom()).abs() <= px(0.6));
            assert!(tree.selected_ids().contains("1"));
            assert_eq!(tree.active_node_id().unwrap().as_ref(), "1");
            assert!(!tree.focus_handle.contains_focused(window, app));
            assert!(tree.position.is_none());
        });
        let before = cx.update(|_, app| tree.read(app).list_state.logical_scroll_top());
        tree.update(cx, |tree, cx| {
            tree.scroll_to("80", cx);
        });
        settle(cx);
        cx.update(|_, app| {
            let after = tree.read(app).list_state.logical_scroll_top();
            assert_eq!(before.item_ix, after.item_ix);
            assert_eq!(before.offset_in_item, after.offset_in_item);
        });
        tree.update(cx, |tree, cx| {
            tree.scroll_to_center("50", cx);
        });
        settle(cx);
        cx.update(|_, app| centered(tree.read(app), "50"));
        tree.update(cx, |tree, cx| {
            tree.scroll_to_center("0", cx);
        });
        settle(cx);
        cx.update(|_, app| assert_eq!(tree.read(app).list_state.logical_scroll_top().item_ix, 0));
        tree.update(cx, |tree, cx| {
            tree.scroll_to_center("99", cx);
        });
        settle(cx);
        cx.update(|_, app| {
            let tree = tree.read(app);
            assert!((bounds(tree, "99").bottom() - tree.list_state.viewport_bounds().bottom()).abs() <= px(0.6));
            assert!(tree.position.is_none());
        });
    }
}

#[test]
fn hidden_nodes_need_explicit_reveal_and_filtered_or_missing_nodes_remain_untouched() {
    let mut app = TestAppContext::single();
    let (_, cx, tree) = setup(&mut app, false);
    tree.update(cx, |tree, cx| {
        let mut items = tree.items().to_vec();
        items.push(node(100, 24).children([node(101, 24).children([node(102, 24)])]));
        tree.replace_items(items, cx).unwrap();
        assert!(!tree.scroll_to("102", cx));
        assert!(!tree.scroll_to_center_smooth("missing", cx));
        tree.set_filter(|node| node.id == "0", cx);
        assert!(!tree.reveal_node("102", cx));
        assert!(!tree.is_expanded(&"100".into()));
        tree.clear_filter(cx);
        assert!(tree.reveal_node("102", cx));
        assert!(tree.is_expanded(&"100".into()) && tree.is_expanded(&"101".into()));
    });
    settle(cx);
    cx.update(|window, app| {
        let tree = tree.read(app);
        assert!(bounds(tree, "102").bottom() <= tree.list_state.viewport_bounds().bottom() + px(0.6));
        assert!(tree.selected_ids().is_empty() && tree.active_node_id().is_none());
        assert!(!tree.focus_handle.contains_focused(window, app));
        assert!(tree.position.is_none());
    });
}

#[test]
fn smooth_positioning_has_intermediate_frames_and_latest_request_wins() {
    let mut app = TestAppContext::single();
    let (_, cx, tree) = setup(&mut app, false);
    tree.update(cx, |tree, cx| {
        tree.scroll_to_center_smooth("80", cx);
    });
    cx.run_until_parked();
    frame(cx, 40);
    cx.update(|_, app| {
        let top = tree.read(app).list_state.logical_scroll_top().item_ix;
        assert!(top > 0 && top < 78, "intermediate top {top}");
    });
    settle(cx);
    cx.update(|_, app| centered(tree.read(app), "80"));
    tree.update(cx, |tree, cx| {
        tree.scroll_to_center_smooth("10", cx);
    });
    cx.run_until_parked();
    frame(cx, 40);
    tree.update(cx, |tree, cx| {
        tree.scroll_to_center("60", cx);
    });
    settle(cx);
    cx.update(|_, app| centered(tree.read(app), "60"));
    tree.update(cx, |tree, cx| {
        tree.scroll_to_smooth("0", cx);
        assert!(!tree.scroll_to("missing", cx));
        assert!(tree.position.is_none());
    });
    settle(cx);
    cx.update(|_, app| centered(tree.read(app), "60"));
}

#[test]
fn wheel_keyboard_scrollbar_and_projection_changes_interrupt_motion() {
    for input in 0..5 {
        let mut app = TestAppContext::single();
        let (_, cx, tree) = setup(&mut app, false);
        tree.update(cx, |tree, cx| {
            tree.scroll_to_center_smooth("80", cx);
        });
        cx.run_until_parked();
        frame(cx, 40);
        match input {
            0 => cx.simulate_event(ScrollWheelEvent {
                position: point(px(30.0), px(30.0)),
                delta: ScrollDelta::Pixels(point(px(0.0), px(-25.0))),
                ..Default::default()
            }),
            1 => {
                cx.update(|window, app| tree.read(app).focus_handle.clone().focus(window, app));
                cx.simulate_keystrokes("down");
            }
            2 => tree.update(cx, |tree, cx| {
                tree.handle_scrollbar(&crate::controls::scrollbar::ScrollbarEvent::DragStart, cx)
            }),
            3 => tree.update(cx, |tree, cx| tree.set_filter(|_| true, cx)),
            _ => tree.update(cx, |tree, cx| tree.replace_items(tree.items().to_vec(), cx).unwrap()),
        }
        cx.run_until_parked();
        let before = cx.update(|_, app| {
            assert!(tree.read(app).position.is_none(), "input {input}");
            tree.read(app).list_state.logical_scroll_top()
        });
        settle(cx);
        cx.update(|_, app| {
            let after = tree.read(app).list_state.logical_scroll_top();
            assert_eq!(before.item_ix, after.item_ix);
            assert_eq!(before.offset_in_item, after.offset_in_item);
        });
    }
}

#[test]
fn pending_position_waits_for_layout_and_centering_tracks_resize() {
    let mut app = TestAppContext::single();
    let (page, cx, tree) = setup(&mut app, true);
    page.update(cx, |page, cx| {
        page.height = 0.0;
        cx.notify();
    });
    cx.run_until_parked();
    tree.update(cx, |tree, cx| {
        tree.scroll_to_center_smooth("70", cx);
    });
    frame(cx, 40);
    page.update(cx, |page, cx| {
        page.height = 220.0;
        cx.notify();
    });
    settle(cx);
    cx.update(|_, app| centered(tree.read(app), "70"));
}

#[test]
fn unmeasured_oversized_row_reveals_its_leading_edge_and_sorted_keys_resolve_at_request() {
    let mut app = TestAppContext::single();
    let (_, cx, tree) = setup(&mut app, false);
    tree.update(cx, |tree, cx| {
        let mut items = tree.items().to_vec();
        items[80].data = 300;
        tree.replace_items(items, cx).unwrap();
        tree.scroll_to("80", cx);
    });
    settle(cx);
    cx.update(|_, app| {
        let tree = tree.read(app);
        assert!((bounds(tree, "80").top() - tree.list_state.viewport_bounds().top()).abs() <= px(0.6));
        assert!(tree.position.is_none());
    });
    tree.update(cx, |tree, cx| {
        tree.set_sort(|a, b| b.label.cmp(&a.label), cx);
        tree.set_filter(|node| node.id != "70", cx);
        assert!(!tree.scroll_to("70", cx));
        assert!(tree.scroll_to_center("50", cx));
    });
    settle(cx);
    cx.update(|_, app| centered(tree.read(app), "50"));
}
