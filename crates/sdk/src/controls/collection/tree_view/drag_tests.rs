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
    named: Vec<(usize, &'static str)>,
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
                .motion_enabled(false)
                .require_focus_for_scroll(true)
                .expand_on_row_click(false)
                .drag_drop(config)
                .on_drag_started(cx, move |owner: &mut Pair, _, _, _, _| owner.named.push((i, "start")))
                .on_items_removed(cx, move |owner, _, _, _, _, _| owner.named.push((i, "remove")))
                .on_items_added(cx, move |owner, _, _, _, _, _, _| owner.named.push((i, "add")))
                .on_items_reordered(cx, move |owner, _, _, _, _, _| owner.named.push((i, "reorder")))
                .on_dropped(cx, move |owner, _, _, _, _, _, _, cx| {
                    owner.named.push((i, "drop"));
                    cx.notify();
                })
                .on_drop_rejected(cx, move |owner, _, _, _, _, _, _| owner.named.push((i, "reject")))
                .on_drag_ended(cx, move |owner, _, _, _, reason, _| {
                    owner.named.push((
                        i,
                        if reason == TreeDragEndReason::Cancelled {
                            "cancel"
                        } else {
                            "end"
                        },
                    ))
                })
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
        Pair { trees, events: Vec::new(), named: Vec::new(), _subscriptions: subscriptions }
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

fn apply_root_move(proposal: &TreeDropProposal<()>, app: &mut App) -> bool {
    let source = proposal.source().unwrap();
    let target = proposal.target().unwrap();
    let old = source.read(app).items().to_vec();
    let destination = target.read(app).items().to_vec();
    let moving: Vec<_> = old.iter().filter(|node| proposal.node_ids().contains(&node.id)).cloned().collect();
    let location = proposal.location();
    assert!(location.parent_id.is_none());
    let gap = match location.position {
        super::super::TreeDropPosition::Before | super::super::TreeDropPosition::After => {
            let anchor = destination.iter().position(|node| Some(&node.id) == location.anchor_id.as_ref()).unwrap();
            anchor + usize::from(location.position == super::super::TreeDropPosition::After)
        }
        super::super::TreeDropPosition::Into => destination.len(),
    };
    let index = destination[..gap]
        .iter()
        .filter(|node| source != target || !proposal.node_ids().contains(&node.id))
        .count();
    let mut next: Vec<_> = destination
        .iter()
        .filter(|node| source != target || !proposal.node_ids().contains(&node.id))
        .cloned()
        .collect();
    next.splice(index..index, moving);
    let changed = next.iter().map(|node| &node.id).collect::<Vec<_>>()
        != destination.iter().map(|node| &node.id).collect::<Vec<_>>();
    if changed {
        let state = source.read(app).capture_subtrees_state(proposal.node_ids());
        if source != target {
            source.update(app, |tree, cx| {
                tree.replace_items(old.into_iter().filter(|node| !proposal.node_ids().contains(&node.id)), cx)
                    .unwrap()
            });
        }
        target.update(app, |tree, cx| {
            tree.replace_items(next, cx).unwrap();
            tree.restore_subtree_state(&state, cx);
        });
    }
    changed
}

#[test]
fn tree_native_transactions_emit_all_shared_phases_with_root_payloads_and_locations() {
    for (destination, decision, expected) in [
        ((450.0, 50.0), "commit", vec![(0, "start"), (0, "remove"), (1, "add"), (1, "drop"), (0, "end")]),
        ((150.0, 110.0), "commit", vec![(0, "start"), (0, "reorder"), (0, "drop"), (0, "end")]),
        ((150.0, 2.0), "commit", vec![(0, "start"), (0, "drop"), (0, "end")]),
        ((450.0, 50.0), "reject", vec![(0, "start"), (1, "reject"), (0, "end")]),
        ((450.0, 50.0), "unanswered", vec![(0, "start"), (1, "reject"), (0, "end")]),
        ((750.0, 300.0), "commit", vec![(0, "start"), (0, "cancel")]),
    ] {
        let mut app = TestAppContext::single();
        let (pair, cx, captured, _) = setup(&mut app, false);
        let trees = cx.update(|_, app| pair.read(app).trees.clone());
        for tree in &trees {
            tree.update(cx, |tree, _| {
                let config = tree.model.drag_drop.as_mut().unwrap();
                config.drag_selected = true;
                config.on_drop = Rc::new(move |proposal, _, app| match decision {
                    "reject" => proposal.rejected("Locked", app),
                    "unanswered" => {}
                    _ => {
                        proposal.validate(app).unwrap();
                        let changed = apply_root_move(&proposal, app);
                        proposal.committed(changed, app);
                        proposal.committed(changed, app);
                        proposal.rejected("late", app);
                    }
                });
            });
        }
        trees[0].update(cx, |tree, cx| {
            tree.set_items(
                [
                    TreeNode::new("source", "Source", ()).children([TreeNode::new("child", "Child", ())]),
                    TreeNode::new("second", "Second", ()),
                    TreeNode::new("third", "Third", ()),
                ],
                cx,
            );
            tree.selected_ids = HashSet::from(["source".into(), "child".into(), "second".into()]);
            cx.notify();
        });
        cx.run_until_parked();
        begin(cx);
        cx.simulate_mouse_move(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
        cx.simulate_mouse_up(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        let location = captured.borrow().as_ref().map(|proposal| proposal.location().clone());
        cx.update(|_, app| {
            let owner = pair.read(app);
            assert_eq!(owner.named, expected, "{destination:?} / {decision}");
            assert_eq!(owner.events.len(), expected.len());
            for event in &owner.events {
                match event {
                    TreeViewDragEvent::ItemsRemoved { source, target, node_id, node_ids } => {
                        assert_eq!(*source, trees[0].entity_id());
                        assert_eq!(*target, trees[1].entity_id());
                        assert_eq!(node_id.as_ref(), "source");
                        assert_eq!(node_ids, &["source", "second"]);
                        assert_eq!(trees[0].read(app).items().len(), 1);
                        assert!(trees[1].read(app).model.index.nodes.contains_key(&SharedString::from("child")));
                    }
                    TreeViewDragEvent::ItemsAdded { node_ids, location: actual, .. }
                    | TreeViewDragEvent::ItemsReordered { node_ids, location: actual, .. } => {
                        assert_eq!(node_ids, &["source", "second"]);
                        assert_eq!(Some(actual), location.as_ref());
                    }
                    TreeViewDragEvent::Dropped { location: actual, changed, .. } => {
                        assert_eq!(Some(actual), location.as_ref());
                        assert_eq!(*changed, destination != (150.0, 2.0));
                    }
                    _ => {}
                }
            }
            if decision != "commit" || destination == (750.0, 300.0) {
                assert_eq!(
                    trees[0].read(app).items().iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(),
                    ["source", "second", "third"]
                );
            }
        });
    }
}

#[test]
fn same_tree_commit_animates_visible_keyed_rows_after_ended_event() {
    use std::time::Duration;
    let mut app = TestAppContext::single();
    let ended = Rc::new(Cell::new(false));
    let report = ended.clone();
    let (tree, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let config = TreeViewDragDrop::new(cx.entity_id(), move |p, _, cx| {
            p.validate(cx).unwrap();
            let source = p.source().unwrap();
            source.update(cx, |tree, cx| {
                tree.replace_items(
                    [TreeNode::new("b", "B", ()), TreeNode::new("c", "C", ()), TreeNode::new("a", "A", ())],
                    cx,
                )
                .unwrap()
            });
            p.committed(true, cx);
        });
        let builder = TreeViewBuilder::new("motion-tree")
            .items([TreeNode::new("a", "A", ()), TreeNode::new("b", "B", ()), TreeNode::new("c", "C", ())])
            .drag_drop(config);
        let mut tree = TreeViewControl::from_builder(builder, cx);
        tree.model.motion.configure(|m| {
            m.drop_layout =
                crate::motion::Transition::timed(Duration::from_millis(1000), crate::motion::Easing::OutCubic)
        });
        tree.model.motion.freeze();
        tree
    });
    cx.update(|_, cx| {
        cx.subscribe(&tree, move |_, event: &TreeViewEvent<()>, _| {
            if matches!(event, TreeViewEvent::DragDrop(TreeViewDragEvent::Ended { .. })) {
                report.set(true)
            }
        })
        .detach();
    });
    cx.run_until_parked();
    let old = cx.debug_bounds("motion-tree/node/a").unwrap();
    let end = cx.debug_bounds("motion-tree/node/c").unwrap();
    let start = old.center();
    let target = point(end.center().x, end.bottom() - px(2.));
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(start + point(px(12.), px(0.)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_move(target, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(target, MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert!(ended.get());
    assert!((cx.debug_bounds("motion-tree/node/a").unwrap().top() - old.top()).abs() < px(1.));
    cx.update(|_, cx| {
        let tree = tree.read(cx);
        assert_eq!(tree.items()[2].id.as_ref(), "a");
        assert!(tree.drop_layout_motion().is_animating());
    });
    tree.update(cx, |tree, _| tree.drop_layout_motion().elapse_for_test(Duration::from_millis(1100)));
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("motion-tree/node/a").unwrap().top() > old.top());
}
