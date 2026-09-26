//! Large-tree TestWindow checks count row construction, not source traversal.
use super::*;
use gpui::{Entity, TestAppContext, VisualTestContext};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Counts {
    calls: usize,
    ids: HashSet<SharedString>,
}
struct Measured {
    base: Arc<dyn super::super::TreeViewTemplate<()>>,
    counts: Arc<Mutex<Counts>>,
}
impl super::super::TreeViewTemplate<()> for Measured {
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: gpui::AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        self.base.render(model, body, window, cx)
    }
    fn render_node(
        &self,
        node: &FlatTreeNode<'_, ()>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let mut counts = self.counts.lock().unwrap();
        counts.calls += 1;
        counts.ids.insert(node.id.clone());
        drop(counts);
        self.base.render_node(node, handlers, window, cx)
    }
}
struct Page {
    tree: Entity<TreeViewControl<()>>,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(360.0)).h(px(240.0)).child(self.tree.clone())
    }
}
fn items() -> Vec<TreeNode<()>> {
    let mut rows: Vec<_> = (0..100)
        .map(|folder| {
            TreeNode::new(format!("folder-{folder}"), format!("Folder {folder:02}"), ())
                .expanded(true)
                .children((0..100).map(|file| {
                    TreeNode::new(format!("file-{folder}-{file}"), format!("Document {folder:02}/{file:02}"), ())
                }))
        })
        .collect();
    let mut deep = TreeNode::new("deep-file", "Deep document", ());
    for depth in (0..32).rev() {
        deep = TreeNode::new(format!("deep-{depth}"), format!("Level {depth}"), ()).expanded(true).children([deep]);
    }
    rows.push(deep);
    rows
}
fn setup(app: &mut TestAppContext) -> (Entity<TreeViewControl<()>>, &mut VisualTestContext, Arc<Mutex<Counts>>) {
    let counts = Arc::new(Mutex::new(Counts::default()));
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let tree = TreeViewBuilder::new("scale-tree")
            .items(items())
            .animated(false)
            .wrap_navigation(false)
            .template(Arc::new(Measured { base: super::super::default_tree_view_template(), counts: counts.clone() }))
            .spawn(cx);
        Page { tree }
    });
    cx.run_until_parked();
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    (tree, cx, counts)
}
fn frame(cx: &mut VisualTestContext) {
    cx.executor().advance_clock(std::time::Duration::from_millis(25));
    cx.update(|window, app| window.simulate_next_frame(app));
    cx.run_until_parked();
}
fn report(counts: &Arc<Mutex<Counts>>, step: &str) {
    let counts = counts.lock().unwrap();
    eprintln!("TreeView scale {step}: unique rows={} template calls={}", counts.ids.len(), counts.calls);
    assert!(counts.ids.len() < 200, "{step}: constructed {} distinct rows", counts.ids.len());
    assert!(counts.calls < 1000, "{step}: {} template calls", counts.calls);
}
fn reset(counts: &Arc<Mutex<Counts>>) {
    *counts.lock().unwrap() = Counts::default();
}

#[test]
fn large_tree_constructs_only_viewport_rows_and_replaces_same_count_while_scrolled() {
    let mut app = TestAppContext::single();
    let (tree, cx, counts) = setup(&mut app);
    cx.update(|_, app| {
        assert_eq!(tree.read(app).model.index.nodes.len(), 10133);
        assert_eq!(tree.read(app).visible_ids().len(), 10133);
    });
    report(&counts, "initial");
    reset(&counts);
    tree.update(cx, |tree, cx| {
        assert!(tree.scroll_to_center("file-75-50", cx));
    });
    for _ in 0..4 {
        frame(cx);
    }
    report(&counts, "distant jump");
    reset(&counts);
    let before = cx.update(|_, app| {
        let tree = tree.read(app);
        let top = tree.list_state.logical_scroll_top();
        (tree.flat_cache[top.item_ix].id.clone(), top.offset_in_item)
    });
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("file-75-50", cx);
        let mut data = items();
        data.reverse();
        data.iter_mut().for_each(|folder| folder.children.reverse());
        data.iter_mut()
            .find(|node| node.id == "folder-75")
            .unwrap()
            .children
            .iter_mut()
            .find(|node| node.id == "file-75-50")
            .unwrap()
            .label = "Updated document".into();
        tree.replace_items(data, cx).unwrap();
        let top = tree.list_state.logical_scroll_top();
        assert_eq!((&tree.flat_cache[top.item_ix].id, top.offset_in_item), (&before.0, before.1));
    });
    for _ in 0..4 {
        frame(cx);
    }
    report(&counts, "same-count reorder");
    cx.update(|_, app| {
        let tree = tree.read(app);
        let row = &tree.flat_cache[tree.flat_index_for_id(&"file-75-50".into()).unwrap()];
        assert_eq!(row.label.as_ref(), "Updated document");
        assert_eq!(tree.selected_ids(), &HashSet::from(["file-75-50".into()]));
    });
}

#[test]
fn large_tree_filter_sort_deep_reveal_and_animation_preserve_keyed_state() {
    let mut app = TestAppContext::single();
    let (tree, cx, counts) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("file-75-50", cx);
        tree.scroll_to_center("file-75-50", cx);
    });
    for _ in 0..4 {
        frame(cx);
    }
    let before = cx.update(|_, app| {
        let tree = tree.read(app);
        let top = tree.list_state.logical_scroll_top();
        tree.flat_cache[top.item_ix].id.clone()
    });
    reset(&counts);
    tree.update(cx, |tree, cx| {
        tree.set_animated(true, cx);
        tree.collapse("folder-0", cx);
        tree.collapse("folder-50", cx);
    });
    tree.update(cx, |tree, cx| {
        tree.expand_transitions.insert("folder-0".into(), DisclosureMotion::new(0.0, true));
        tree.expand_transitions.insert("folder-50".into(), DisclosureMotion::new(0.0, true));
        cx.notify();
    });
    frame(cx);
    report(&counts, "collapse above viewport");
    cx.update(|_, app| {
        let tree = tree.read(app);
        assert_eq!(tree.flat_cache.len(), 9933);
        assert_eq!(tree.flat_cache[tree.list_state.logical_scroll_top().item_ix].id, before);
    });
    reset(&counts);
    tree.update(cx, |tree, cx| {
        tree.set_filter(|node| node.id == "deep-file", cx);
        tree.set_sort(|a, b| b.label.cmp(&a.label), cx);
    });
    for _ in 0..4 {
        frame(cx);
    }
    report(&counts, "deep filter");
    reset(&counts);
    tree.update(cx, |tree, cx| {
        assert_eq!(tree.visible_ids().len(), 33);
        assert!(tree.selected_ids().contains("file-75-50"));
        tree.clear_filter(cx);
        tree.clear_sort(cx);
        tree.collapse("deep-0", cx);
        tree.reveal_node("deep-file", cx);
    });
    for _ in 0..4 {
        frame(cx);
    }
    report(&counts, "deep reveal");
    assert!(cx.debug_bounds("scale-tree/node/deep-file").is_some());
}

#[test]
fn very_wide_branch_collapse_keeps_row_construction_bounded() {
    let mut app = TestAppContext::single();
    let (tree, cx, counts) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.set_items(
            [
                TreeNode::new("wide", "Wide", ())
                    .expanded(true)
                    .children((0..10000).map(|i| TreeNode::new(format!("wide-{i}"), "Document", ()))),
                TreeNode::new("tail", "Tail", ()),
            ],
            cx,
        );
        tree.set_animated(true, cx);
    });
    cx.run_until_parked();
    reset(&counts);
    tree.update(cx, |tree, cx| tree.collapse("wide", cx));
    frame(cx);
    report(&counts, "wide collapse settles before layout");
    cx.update(|_, app| assert!(!tree.read(app).expand_transitions[&SharedString::from("wide")].is_animating()));
    reset(&counts);
    tree.update(cx, |tree, cx| tree.expand("wide", cx));
    frame(cx);
    report(&counts, "wide expand settles before layout");
    cx.update(|_, app| assert_eq!(tree.read(app).flat_cache.len(), 10002));
    tree.update(cx, |tree, cx| tree.collapse("wide", cx));
    frame(cx);
    cx.update(|_, app| assert_eq!(tree.read(app).flat_cache.len(), 2));
}

#[test]
fn simultaneous_branch_motion_obeys_one_row_budget_and_small_branches_still_animate() {
    let mut app = TestAppContext::single();
    let (tree, cx, counts) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.set_animated(true, cx);
        tree.collapse("folder-0", cx);
    });
    frame(cx);
    cx.update(|_, app| assert!(tree.read(app).expand_transitions[&SharedString::from("folder-0")].is_animating()));
    reset(&counts);
    tree.update(cx, |tree, cx| {
        tree.collapse("folder-1", cx);
        tree.collapse("folder-2", cx);
    });
    frame(cx);
    report(&counts, "simultaneous collapse settles");
    cx.update(|_, app| assert_eq!(tree.read(app).flat_cache.len(), 9833));
}
