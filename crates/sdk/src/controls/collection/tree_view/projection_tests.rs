//! Projection transactions and dispatch in GPUI TestWindow.
use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use gpui::{Entity, TestAppContext, VisualTestContext};

fn leaf(id: &str) -> TreeNode<()> {
    TreeNode::new(id.to_owned(), id.to_owned(), ())
}
fn items() -> Vec<TreeNode<()>> {
    vec![
        leaf("z").expanded(true).children([
            leaf("docs").expanded(true).children([leaf("readme"), leaf("guide")]),
            leaf("empty").branch(true),
        ]),
        leaf("a"),
    ]
}
fn setup(app: &mut TestAppContext) -> (Entity<TreeViewControl<()>>, &mut VisualTestContext) {
    app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        TreeViewControl::from_builder(TreeViewBuilder::new("projection-tree").items(items()).animated(false), cx)
    })
}
fn ids(tree: &TreeViewControl<()>) -> Vec<String> {
    tree.visible_ids().iter().map(ToString::to_string).collect()
}

#[test]
fn filter_opens_only_match_paths_and_restores_saved_expansion_and_hidden_selection() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("readme", cx);
        tree.collapse("z", cx);
    });
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(&tree, move |_, event, _| events.borrow_mut().push(event.clone())).detach();
    });
    let calls = Rc::new(Cell::new(0));
    tree.update(cx, |tree, cx| {
        let calls = calls.clone();
        tree.set_filter(
            move |node| {
                calls.set(calls.get() + 1);
                node.id == "guide"
            },
            cx,
        );
        assert_eq!(ids(tree), ["z", "docs", "guide"]);
        assert!(tree.is_expanded(&"z".into()));
        tree.toggle_expand("z", cx); // required ancestor stays open; saved state is unchanged
        tree.expand("z", cx);
        assert!(tree.is_expanded(&"z".into()));
        assert!(tree.selected_ids().contains("readme"));
        tree.clear_filter(cx);
        assert_eq!(ids(tree), ["z", "a"]);
        assert!(!tree.is_expanded(&"z".into()));
        assert!(tree.is_expanded(&"docs".into()));
        tree.set_filter(|node| node.id == "docs", cx);
        assert_eq!(ids(tree), ["z", "docs"], "matching branches do not include unmatched children");
        tree.set_filter(|_| false, cx);
        assert!(ids(tree).is_empty());
        assert!(tree.active_node_id().is_none());
        assert!(tree.selected_ids().contains("readme"));
        tree.clear_filter(cx);
        assert!(tree.active_node_id().is_none(), "clearing does not synthesize activation");
        tree.expand("z", cx);
        assert!(tree.selected_ids().contains("readme"));
    });
    assert_eq!(calls.get(), 6, "evaluate every loaded node, including collapsed descendants, once");
    cx.run_until_parked();
    let events = events.borrow();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, TreeViewEvent::SelectionChanged { .. } | TreeViewEvent::NodeCollapsed { .. }))
    );
    assert_eq!(
        events.iter().filter(|e| matches!(e, TreeViewEvent::NodeExpanded { .. })).count(),
        1,
        "only explicit expansion after clearing emits"
    );
}

#[test]
fn sibling_sort_is_stable_and_keyboard_navigation_uses_projected_preorder() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.set_sort(|a, b| a.label.cmp(&b.label), cx);
        assert_eq!(ids(tree), ["a", "z", "docs", "guide", "readme", "empty"]);
        assert_eq!(tree.items()[0].id.as_ref(), "z");
        assert_eq!(tree.items()[0].children[0].children[0].id.as_ref(), "readme");
        tree.select_node_by_id("guide", cx);
    });
    cx.update(|window, app| tree.read(app).focus_handle.clone().focus(window, app));
    cx.simulate_keystrokes("down");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().unwrap().as_ref(), "readme"));
    tree.update(cx, |tree, cx| {
        tree.set_filter(|node| node.id == "guide" || node.id == "readme", cx);
        tree.set_sort(|a, b| b.label.cmp(&a.label), cx);
        assert_eq!(ids(tree), ["z", "docs", "readme", "guide"]);
        tree.set_sort(|_, _| std::cmp::Ordering::Equal, cx);
        assert_eq!(ids(tree), ["z", "docs", "readme", "guide"], "stable ties retain source order");
        tree.clear_sort(cx);
        assert!(tree.has_projection(), "filter remains active");
        tree.clear_filter(cx);
        assert!(!tree.has_projection());
        assert_eq!(ids(tree), ["z", "docs", "readme", "guide", "empty", "a"]);
        assert!(tree.selected_ids().contains("guide"));
    });
}

#[test]
fn replacement_reprojects_and_reconciles_active_once_without_hidden_selection_loss() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("guide", cx);
        tree.set_filter(|node| node.label.contains("guide"), cx);
    });
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(&tree, move |_, event, _| events.borrow_mut().push(event.clone())).detach();
    });
    tree.update(cx, |tree, cx| {
        let mut data = items();
        data[0].children[0].children[1].label = "renamed".into();
        data[0].children[0].children.push(leaf("new-guide"));
        tree.replace_items(data, cx).unwrap();
        assert_eq!(ids(tree), ["z", "docs", "new-guide"]);
        assert_eq!(tree.active_node_id().unwrap().as_ref(), "docs");
        assert!(tree.selected_ids().contains("guide"));
        let before = ids(tree);
        assert!(tree.replace_items([leaf("duplicate"), leaf("duplicate")], cx).is_err());
        assert_eq!(ids(tree), before);
    });
    cx.run_until_parked();
    let events = events.borrow();
    assert_eq!(events.iter().filter(|e| matches!(e, TreeViewEvent::ActiveNodeChanged { .. })).count(), 1);
    assert!(!events.iter().any(|e| matches!(e, TreeViewEvent::SelectionChanged { .. })));
}

#[test]
fn sorting_preserves_scroll_anchor_and_filter_change_settles_old_animation() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.set_items((0..100).map(|i| leaf(&format!("{i:03}"))), cx);
        tree.list_state.scroll_to(gpui::ListOffset { item_ix: 30, offset_in_item: px(5.0) });
        tree.set_sort(|a, b| b.label.cmp(&a.label), cx);
        let top = tree.list_state.logical_scroll_top();
        assert_eq!(tree.flat_cache[top.item_ix].id.as_ref(), "030");
        assert_eq!(top.offset_in_item, px(5.0));
        tree.set_items(items(), cx);
        tree.set_animated(true, cx);
        tree.collapse("z", cx);
        assert!(tree.flat_cache.iter().filter(|n| n.depth > 0).all(|n| !n.enabled));
        tree.set_filter(|node| node.id == "guide", cx);
        assert_eq!(ids(tree), ["z", "docs", "guide"]);
        assert!(tree.flat_cache.iter().all(|n| n.enabled && n.row_height_factor == 1.0));
        tree.clear_filter(cx);
        assert_eq!(ids(tree), ["z", "a"]);
    });
}

#[test]
fn builder_projection_accepts_ui_captures_and_filters_domain_data() {
    let mut app = TestAppContext::single();
    let query = Rc::new(String::from("keep"));
    let (tree, cx) = app.add_window_view(|_, cx| {
        TreeViewControl::from_builder(
            TreeViewBuilder::new("builder-projection")
                .items([TreeNode::new("parent", "Parent", "other").children([
                    TreeNode::new("b", "B", "keep"),
                    TreeNode::new("a", "A", "keep"),
                    TreeNode::new("c", "C", "other"),
                ])])
                .filter(move |node| node.data == query.as_str())
                .sort(|a, b| a.label.cmp(&b.label)),
            cx,
        )
    });
    cx.update(|_, app| {
        assert_eq!(tree.read(app).visible_ids(), vec![SharedString::from("parent"), "a".into(), "b".into()]);
        assert!(tree.read(app).expanded_ids.is_empty());
    });
}
