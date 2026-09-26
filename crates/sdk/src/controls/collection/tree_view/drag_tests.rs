//! Real GPUI dispatch against scoped tree targets; no native application launch.
use super::*;
use super::super::{TreeViewDragDrop, TreeDropProposal, TreeViewDragEvent, TreeDragEndReason};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use gpui::{Entity, TestAppContext, VisualTestContext, MouseButton, point};

struct Pair {
    trees: [Entity<TreeViewControl<()>>; 2],
    events: Vec<TreeViewDragEvent>,
    _subscriptions: Vec<Subscription>,
}
impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .children(self.trees.iter().map(|tree| div().w(px(300.0)).h(px(120.0)).child(tree.clone())))
    }
}
type Captured = Rc<RefCell<Option<TreeDropProposal<()>>>>;
fn setup(app: &mut TestAppContext, foreign: bool) -> (Entity<Pair>, &mut VisualTestContext, Captured, Rc<Cell<usize>>) {
    let captured: Captured = Rc::default();
    let drops = Rc::new(Cell::new(0));
    let (view, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let owner_scope = cx.entity_id();
        let other_scope = cx.new(|_| ()).entity_id();
        let trees = std::array::from_fn(|i| {
            let captured = captured.clone();
            let drops = drops.clone();
            let config = TreeViewDragDrop::new(
                if foreign && i == 1 { other_scope } else { owner_scope },
                move |proposal, _, app| {
                    drops.set(drops.get() + 1);
                    proposal.committed(false, app);
                    proposal.committed(false, app); // completion guard must prevent duplicates
                },
            )
            .can_drop(move |proposal, _| {
                *captured.borrow_mut() = Some(proposal.clone());
                true
            });
            let items = if i == 0 {
                vec![TreeNode::new("source", "Source", ())]
            } else {
                (0..100).map(|i| TreeNode::new(format!("target-{i}"), format!("Target {i}"), ())).collect()
            };
            TreeViewBuilder::new(if i == 0 { "drag-source" } else { "drag-target" })
                .items(items)
                .animated(false)
                .require_focus_for_scroll(true)
                .expand_on_row_click(false)
                .drag_drop(config)
                .spawn(cx)
        });
        let subscriptions = trees
            .iter()
            .map(|tree| {
                cx.subscribe(tree, |this: &mut Pair, _, event, _| {
                    if let TreeViewEvent::DragDrop(event) = event {
                        this.events.push(event.clone());
                    }
                })
            })
            .collect();
        Pair { trees, events: Vec::new(), _subscriptions: subscriptions }
    });
    cx.run_until_parked();
    (view, cx, captured, drops)
}
fn begin(cx: &mut VisualTestContext) {
    cx.simulate_mouse_down(point(px(150.0), px(12.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(165.0), px(12.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(450.0), px(50.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
}

#[test]
fn scoped_proposals_revalidate_data_and_ended_sessions_cannot_commit_twice() {
    let mut app = TestAppContext::single();
    let (pair, cx, captured, drops) = setup(&mut app, false);
    begin(cx);
    let proposal = captured.borrow().clone().expect("hover proposal");
    cx.update(|_, app| assert!(proposal.validate(app).is_ok()));
    let source = cx.update(|_, app| pair.read(app).trees[0].clone());
    source.update(cx, |tree, cx| tree.replace_items(tree.items().to_vec(), cx).unwrap());
    cx.update(|_, app| assert_eq!(proposal.validate(app).unwrap_err().as_ref(), "Tree changed during drag"));
    cx.simulate_mouse_up(point(px(450.0), px(50.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert_eq!(drops.get(), 0);
    cx.update(|_, app| {
        assert!(!proposal.is_active());
        proposal.committed(true, app);
        assert_eq!(pair.read(app).events.iter().filter(|e| matches!(e, TreeViewDragEvent::Ended { .. })).count(), 1);
    });
}

#[test]
fn foreign_scope_cannot_hover_drop_or_auto_scroll() {
    let mut app = TestAppContext::single();
    let (pair, cx, captured, drops) = setup(&mut app, true);
    begin(cx);
    captured.borrow_mut().take(); // ignore the initial hover over the source itself
    cx.simulate_mouse_move(point(px(450.0), px(118.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(point(px(450.0), px(118.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    let target_id = cx.update(|_, app| pair.read(app).trees[1].entity_id());
    assert!(
        !captured
            .borrow()
            .as_ref()
            .is_some_and(|proposal| proposal.target().is_some_and(|target| target.entity_id() == target_id))
    );
    assert_eq!(drops.get(), 0);
    cx.update(|_, app| {
        assert_eq!(pair.read(app).trees[1].read(app).list_state.logical_scroll_top().item_ix, 0);
        assert_eq!(pair.read(app).trees[1].read(app).list_state.logical_scroll_top().offset_in_item, px(0.0));
    });
}

#[test]
fn accepted_drag_auto_scrolls_unfocused_destination_and_stops_on_cancel() {
    let mut app = TestAppContext::single();
    let (pair, cx, _, _) = setup(&mut app, false);
    begin(cx);
    let target = cx.update(|_, app| pair.read(app).trees[1].clone());
    cx.update(|window, app| assert!(!target.read(app).focus_handle.contains_focused(window, app)));
    cx.simulate_mouse_move(point(px(450.0), px(118.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert!(target.read(app).list_state.scroll_px_offset_for_scrollbar().y < px(0.0)));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    let top = cx.update(|_, app| target.read(app).list_state.logical_scroll_top());
    cx.executor().advance_clock(std::time::Duration::from_millis(100));
    cx.run_until_parked();
    cx.update(|_, app| {
        assert!(!app.has_active_drag());
        assert_eq!(target.read(app).list_state.logical_scroll_top().item_ix, top.item_ix);
        assert_eq!(target.read(app).list_state.logical_scroll_top().offset_in_item, top.offset_in_item);
    });
}

#[test]
fn many_target_updates_keep_one_drag_lifecycle_and_report_dispatch_latency() {
    let mut app = TestAppContext::single();
    let (pair, cx, _, drops) = setup(&mut app, false);
    begin(cx);
    let preview_size = cx.debug_bounds("tree-drag-preview").expect("native preview").size;
    let mut times = Vec::new();
    for i in 0..50 {
        let start = std::time::Instant::now();
        cx.simulate_mouse_move(
            point(px(450.0), px(45.0 + (i % 3) as f32 * 10.0)),
            MouseButton::Left,
            Default::default(),
        );
        cx.run_until_parked();
        times.push(start.elapsed().as_micros());
        assert_eq!(cx.debug_bounds("tree-drag-preview").unwrap().size, preview_size);
    }
    times.sort();
    eprintln!("TreeView headless target dispatch (50 moves): median={}us max={}us", times[25], times[49]);
    cx.simulate_mouse_up(point(px(450.0), px(50.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert_eq!(drops.get(), 1);
    cx.update(|_, app| {
        let events = &pair.read(app).events;
        assert_eq!(events.iter().filter(|e| matches!(e, TreeViewDragEvent::Started { .. })).count(), 1);
        assert_eq!(events.iter().filter(|e| matches!(e, TreeViewDragEvent::Dropped { .. })).count(), 1);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, TreeViewDragEvent::Ended { reason: TreeDragEndReason::Unchanged, .. }))
                .count(),
            1
        );
    });
}

#[test]
fn changing_source_filter_or_destination_sort_invalidates_retained_proposals() {
    for change_source in [true, false] {
        let mut app = TestAppContext::single();
        let (pair, cx, captured, drops) = setup(&mut app, false);
        begin(cx);
        let proposal = captured.borrow().clone().expect("hover proposal");
        let trees = cx.update(|_, app| pair.read(app).trees.clone());
        if change_source {
            trees[0].update(cx, |tree, cx| tree.set_filter(|_| true, cx));
        } else {
            trees[1].update(cx, |tree, cx| tree.set_sort(|a, b| a.label.cmp(&b.label), cx));
        }
        cx.update(|_, app| assert_eq!(proposal.validate(app).unwrap_err().as_ref(), "Tree changed during drag"));
        cx.simulate_keystrokes("escape");
        assert_eq!(drops.get(), 0);
    }
}

#[test]
fn extended_drag_does_not_replace_existing_selection() {
    let mut app = TestAppContext::single();
    let (pair, cx, _, drops) = setup(&mut app, false);
    let source = cx.update(|_, app| pair.read(app).trees[0].clone());
    source.update(cx, |tree, cx| {
        tree.set_selection_policy(super::super::TreeViewSelectionPolicy::new(TreeViewSelectionMode::Extended), cx);
        tree.set_items([TreeNode::new("source", "Source", ()), TreeNode::new("selected", "Selected", ())], cx);
        tree.select_node_by_id("selected", cx);
    });
    cx.run_until_parked();
    begin(cx);
    cx.simulate_mouse_up(point(px(450.0), px(50.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert_eq!(drops.get(), 1);
    cx.update(|_, app| assert_eq!(source.read(app).selected_ids(), &HashSet::from(["selected".into()])));
}

#[test]
fn grouped_drag_is_opt_in_captures_ordered_keys_and_deduplicates_completion() {
    for grouped in [false, true] {
        let mut app = TestAppContext::single();
        let (pair, cx, captured, _) = setup(&mut app, false);
        let source = cx.update(|_, app| pair.read(app).trees[0].clone());
        source.update(cx, |tree, cx| {
            tree.model.drag_drop.as_mut().unwrap().drag_selected = grouped;
            tree.set_items([TreeNode::new("source", "Source", ()), TreeNode::new("second", "Second", ())], cx);
            tree.set_selection_policy(super::super::TreeViewSelectionPolicy::new(TreeViewSelectionMode::Extended), cx);
            tree.selected_ids = HashSet::from(["second".into(), "source".into()]);
            cx.notify();
        });
        cx.run_until_parked();
        begin(cx);
        let proposal = captured.borrow().clone().unwrap();
        let expected: Vec<SharedString> = if grouped {
            vec!["source".into(), "second".into()]
        } else {
            vec!["source".into()]
        };
        assert_eq!(proposal.node_ids(), expected);
        cx.update(|_, app| {
            assert!(proposal.validate(app).is_ok());
            proposal.rejected("Host refused group", app);
            proposal.committed(true, app); // retained proposal must not complete twice
        });
        cx.simulate_mouse_up(point(px(450.0), px(50.0)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.update(|_, app| {
            let events = &pair.read(app).events;
            assert_eq!(events.iter().filter(|event| matches!(event, TreeViewDragEvent::Ended {..})).count(),1);
            assert!(events.iter().any(|event| matches!(event, TreeViewDragEvent::Rejected {node_ids,..} if node_ids == &expected)));
            assert!(events.iter().any(|event| matches!(event, TreeViewDragEvent::Ended {node_ids,reason:TreeDragEndReason::Rejected,..} if node_ids == &expected)));
            assert_eq!(source.read(app).selected_ids(), &HashSet::from(["source".into(),"second".into()]));
        });
    }
}

#[test]
fn grouped_proposal_revalidates_every_root_and_stale_snapshot() {
    let mut app = TestAppContext::single();
    let (pair, cx, captured, _) = setup(&mut app, false);
    let [source, target] = cx.update(|_, app| pair.read(app).trees.clone());
    source.update(cx, |tree, cx| {
        tree.model.drag_drop.as_mut().unwrap().drag_selected = true;
        tree.set_items([TreeNode::new("source", "Source", ()), TreeNode::new("second", "Second", ())], cx);
        tree.selected_ids = HashSet::from(["source".into(), "second".into()]);
        cx.notify();
    });
    cx.run_until_parked();
    begin(cx);
    let proposal = captured.borrow().clone().unwrap();
    assert_eq!(proposal.node_ids().len(), 2);
    source.update(cx, |tree, cx| {
        let mut items = tree.items().to_vec();
        items[1].enabled = false;
        tree.replace_items(items, cx).unwrap();
    });
    cx.update(|_, app| assert!(proposal.validate(app).is_err()));
    cx.simulate_keystrokes("escape");
    // A later session captures just the eligible first row; target updates still invalidate it.
    cx.run_until_parked();
    begin(cx);
    let proposal = captured.borrow().clone().unwrap();
    target.update(cx, |tree, cx| tree.replace_items(tree.items().to_vec(), cx).unwrap());
    cx.update(|_, app| assert!(proposal.validate(app).is_err()));
    cx.simulate_keystrokes("escape");
}
