//! Headless checks for forwarding callbacks through Look-owned builders.
use gpui::{AppContext, TestAppContext};
#[derive(Default)]
struct Owner {
    all: usize,
    specific: usize,
}
macro_rules! check_look {
    ($test:ident, $builder:expr, $method:ident($($arg:tt),*), $event:expr) => {
        #[test]
        fn $test() {
            let app = TestAppContext::single();
            let owner = app.update(|cx| cx.new(|_| Owner::default()));
            let control = app.update(|cx| owner.update(cx, |_, cx| {
                $builder
                    .on_event(cx, |owner: &mut Owner, _, _| owner.all += 1)
                    .$method(cx, |owner, $($arg,)* _| owner.specific += 1)
                    .spawn(cx)
            }));
            app.update(|cx| control.update(cx, |_, cx| cx.emit($event)));
            app.read(|cx| {
                assert_eq!(owner.read(cx).all, 1);
                assert_eq!(owner.read(cx).specific, 1);
            });
        }
    };
}
check_look!(
    checkbox_group_forwards_callbacks,
    crate::CheckboxGroup::<gpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_, _, _),
    gpui_luma::controls::control_group::ControlGroupEvent::Change {
        changed_id: "payload".into(),
        selected: true,
        selected_ids: Vec::new()
    }
);
check_look!(
    toolbar_forwards_callbacks,
    crate::Toolbar::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::toolbar::ToolbarEvent::Change {
        id: "payload".into(),
        value: gpui_luma::controls::toolbar::ToolbarValue::Bool(true)
    }
);
check_look!(
    tree_view_forwards_callbacks,
    crate::TreeView::<()>::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::FocusChanged { focused: true }
);
check_look!(
    selector_forwards_callbacks,
    crate::Selector::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::selector::SelectorEvent::Change { item_id: "payload".into(), label: "payload".into() }
);
check_look!(
    radio_group_forwards_callbacks,
    crate::RadioGroup::<gpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::radio_group::RadioGroupEvent::Change { value: None }
);
check_look!(
    popup_menu_forwards_callbacks,
    crate::PopupMenu::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::popup_menu::PopupMenuEvent::FocusChanged { focused: true }
);
check_look!(
    text_area_forwards_callbacks,
    crate::TextArea::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::textarea::TextAreaEvent::Change { value: "payload".into() }
);
check_look!(
    radio_forwards_callbacks,
    crate::Radio::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::radio_button::RadioButtonEvent::Change { selected: true }
);
check_look!(
    text_field_forwards_callbacks,
    crate::TextField::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::textfield::TextFieldEvent::Change { value: "payload".into() }
);
check_look!(
    tabs_forwards_callbacks,
    crate::Tabs::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::tabs::TabsEvent::Change { tab_id: "payload".into(), label: "payload".into() }
);
check_look!(
    slider_forwards_callbacks,
    crate::Slider::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::slider::SliderEvent::Change {
        thumb_id: gpui_luma::controls::slider::ThumbId::next(),
        value: 0.75
    }
);
check_look!(
    context_menu_forwards_callbacks,
    crate::ContextMenu::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::context_menu::ContextMenuEvent::FocusChanged { focused: true }
);

check_look!(
    tree_drag_started_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drag_started(_, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::Started {
            source: gpui::EntityId::from(1),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()]
        }
    )
);

check_look!(
    tree_dropped_forwards,
    crate::TreeView::<()>::new("drag"),
    on_dropped(_, _, _, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::Dropped {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: gpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: gpui_luma::controls::tree_view::TreeDropPosition::Before
            },
            changed: false
        }
    )
);

check_look!(
    tree_drop_rejected_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drop_rejected(_, _, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::Rejected {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            error: "denied".into()
        }
    )
);

check_look!(
    tree_drag_ended_forwards,
    crate::TreeView::<()>::new("drag"),
    on_drag_ended(_, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::Ended {
            source: gpui::EntityId::from(1),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            reason: gpui_luma::controls::tree_view::TreeDragEndReason::Unchanged
        }
    )
);

check_look!(
    tree_items_removed_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_removed(_, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::ItemsRemoved {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()]
        }
    )
);

check_look!(
    tree_items_added_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_added(_, _, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::ItemsAdded {
            source: gpui::EntityId::from(1),
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: gpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: gpui_luma::controls::tree_view::TreeDropPosition::Before
            }
        }
    )
);

check_look!(
    tree_items_reordered_forwards,
    crate::TreeView::<()>::new("drag"),
    on_items_reordered(_, _, _, _),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::DragDrop(
        gpui_luma::controls::tree_view::TreeViewDragEvent::ItemsReordered {
            target: gpui::EntityId::from(2),
            node_id: "a".into(),
            node_ids: vec!["a".into(), "b".into()],
            location: gpui_luma::controls::tree_view::TreeDropLocation {
                parent_id: None,
                anchor_id: Some("c".into()),
                position: gpui_luma::controls::tree_view::TreeDropPosition::Before
            }
        }
    )
);
