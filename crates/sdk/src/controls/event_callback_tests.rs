//! Headless coverage for SDK builder and factory callback dispatch.
use gpui::{AppContext, TestAppContext};
use crate::prelude::*;
#[derive(Default)]
struct Owner {
    log: Vec<String>,
}

macro_rules! check_callbacks {
    ($test:ident, $builder:expr, [$($method:ident($($arg:ident),*) => [$($event:expr),*]),* $(,)?]) => {
        #[test]
        fn $test() {
            let app = TestAppContext::single();
            let owner = app.update(|cx| cx.new(|_| Owner::default()));
            let control = app.update(|cx| owner.update(cx, |_, cx| {
                $builder
                    .on_event(cx, |owner: &mut Owner, _, _| owner.log.push("all".into()))
                    $(.$method(cx, |owner, $($arg,)* _| {
                        owner.log.push(format!("{}:{:?}", stringify!($method), ($($arg,)*)));
                    }))*
                    .spawn(cx)
                    .on_event(cx, |owner, _, _| owner.log.push("factory".into()))
            }));
            $($(
                app.update(|cx| {
                    owner.update(cx, |owner, _| owner.log.clear());
                    control.update(cx, |_, cx| cx.emit($event));
                });
                app.read(|cx| {
                    let log = &owner.read(cx).log;
                    assert_eq!(log.len(), 3, "{}: {log:?}", stringify!($method));
                    assert!(log.iter().any(|s| s == "all"));
                    assert!(log.iter().any(|s| s == "factory"));
                    assert!(log.iter().any(|s| s.starts_with(concat!(stringify!($method), ":"))));
                });
            )*)*
        }
    };
}
macro_rules! check_drag_entity_callbacks {
    ($test:ident, $builder:expr, [$($method:ident($($arg:ident),*) => [$($event:expr),*]),* $(,)?]) => {
        #[test]
        fn $test() {
            let app = TestAppContext::single();
            let owner = app.update(|cx| cx.new(|_| Owner::default()));
            let control = app.update(|cx| owner.update(cx, |_, cx| {
                $builder
                    .on_event(cx, |owner: &mut Owner, _, _| owner.log.push("all".into()))
                    .spawn(cx)
                    $(.$method(cx, |owner, $($arg,)* _| {
                        owner.log.push(format!("{}:{:?}", stringify!($method), ($($arg,)*)));
                    }))*
                    .on_event(cx, |owner, _, _| owner.log.push("factory".into()))
            }));
            $($(
                app.update(|cx| {
                    owner.update(cx, |owner, _| owner.log.clear());
                    control.update(cx, |_, cx| cx.emit($event));
                });
                app.read(|cx| {
                    let log = &owner.read(cx).log;
                    assert_eq!(log.len(), 3, "{}: {log:?}", stringify!($method));
                    assert!(log.iter().any(|s| s == "all"));
                    assert!(log.iter().any(|s| s == "factory"));
                    assert!(log.iter().any(|s| s.starts_with(concat!(stringify!($method), ":"))));
                });
            )*)*
        }
    };
}
check_callbacks!(
overlay_window_callbacks,
crate::controls::overlay_window::OverlayWindowBuilder::new("callbacks"),
[
on_opened() => [crate::controls::overlay_window::OverlayWindowEvent::Opened],
on_dismissed() => [crate::controls::overlay_window::OverlayWindowEvent::Dismissed]
]);

check_callbacks!(
popover_button_callbacks,
crate::controls::popover_button::PopoverButtonBuilder::<()>::new("callbacks"),
[
on_open_changed(open) => [crate::controls::popover_button::PopoverButtonEvent::OpenChanged { open: true }]
]);

check_callbacks!(
context_menu_callbacks,
crate::controls::context_menu::ContextMenuBuilder::new("callbacks"),
[
on_select(item_id, label) => [crate::controls::context_menu::ContextMenuEvent::Select { item_id: "payload".into(), label: "payload".into() }],
on_open_changed(open) => [crate::controls::context_menu::ContextMenuEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::context_menu::ContextMenuEvent::Dismiss],
on_focus_changed(focused) => [crate::controls::context_menu::ContextMenuEvent::FocusChanged { focused: true }],
on_hover_changed(hovered) => [crate::controls::context_menu::ContextMenuEvent::HoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::context_menu::ContextMenuEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
popup_menu_callbacks,
crate::controls::popup_menu::PopupMenuBuilder::new("callbacks"),
[
on_action_click() => [crate::controls::popup_menu::PopupMenuEvent::ActionClick],
on_select(item_id, label) => [crate::controls::popup_menu::PopupMenuEvent::Select { item_id: "payload".into(), label: "payload".into() }],
on_open_changed(open) => [crate::controls::popup_menu::PopupMenuEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::popup_menu::PopupMenuEvent::Dismiss],
on_focus_changed(focused) => [crate::controls::popup_menu::PopupMenuEvent::FocusChanged { focused: true }],
on_hover_changed(hovered) => [crate::controls::popup_menu::PopupMenuEvent::HoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::popup_menu::PopupMenuEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
slider_callbacks,
crate::controls::slider::SliderBuilder::new("callbacks"),
[
on_value_changed(thumb_id, value) => [crate::controls::slider::SliderEvent::Change { thumb_id: crate::controls::slider::ThumbId::next(), value: 0.75 }],
on_release(thumb_id, value) => [crate::controls::slider::SliderEvent::Release { thumb_id: crate::controls::slider::ThumbId::next(), value: 0.75 }],
on_drag_start(thumb_id) => [crate::controls::slider::SliderEvent::DragStart { thumb_id: crate::controls::slider::ThumbId::next() }],
on_drag_end(thumb_id, value) => [crate::controls::slider::SliderEvent::DragEnd { thumb_id: crate::controls::slider::ThumbId::next(), value: 0.75 }],
on_thumb_added(thumb_id, value) => [crate::controls::slider::SliderEvent::ThumbAdded { thumb_id: crate::controls::slider::ThumbId::next(), value: 0.75 }],
on_thumb_removed(thumb_id) => [crate::controls::slider::SliderEvent::ThumbRemoved { thumb_id: crate::controls::slider::ThumbId::next() }],
on_thumb_selected(thumb_id) => [crate::controls::slider::SliderEvent::ThumbSelected { thumb_id: crate::controls::slider::ThumbId::next() }],
on_focus_changed(focused) => [crate::controls::slider::SliderEvent::FocusChanged { focused: true }],
on_hover_changed(hovered) => [crate::controls::slider::SliderEvent::HoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::slider::SliderEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
text_field_callbacks,
crate::controls::textfield::TextFieldBuilder::new("callbacks"),
[
on_value_changed(value) => [crate::controls::textfield::TextFieldEvent::Change { value: "payload".into() }],
on_submit(value) => [crate::controls::textfield::TextFieldEvent::Submit { value: "payload".into() }],
on_focus_changed(focused) => [crate::controls::textfield::TextFieldEvent::FocusChanged { focused: true }],
on_enabled_changed(enabled) => [crate::controls::textfield::TextFieldEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
text_area_callbacks,
crate::controls::textarea::TextAreaBuilder::new("callbacks"),
[
on_value_changed(value) => [crate::controls::textarea::TextAreaEvent::Change { value: "payload".into() }],
on_focus_changed(focused) => [crate::controls::textarea::TextAreaEvent::FocusChanged { focused: true }],
on_enabled_changed(enabled) => [crate::controls::textarea::TextAreaEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
selector_callbacks,
crate::controls::selector::SelectorBuilder::<crate::controls::selector::SelectorItem>::new("callbacks"),
[
on_value_changed(item_id, label) => [crate::controls::selector::SelectorEvent::Change { item_id: "payload".into(), label: "payload".into() }],
on_focus_changed(focused) => [crate::controls::selector::SelectorEvent::FocusChanged { focused: true }],
on_open_changed(open) => [crate::controls::selector::SelectorEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::selector::SelectorEvent::Dismiss]
]);

check_callbacks!(
autocomplete_callbacks,
crate::controls::autocomplete::AutocompleteBuilder::new("callbacks", []),
[
on_value_changed(query) => [crate::controls::autocomplete::AutocompleteEvent::Change { query: "payload".into() }],
on_select(item_id, label) => [crate::controls::autocomplete::AutocompleteEvent::Select { item_id: "payload".into(), label: "payload".into() }],
on_complete(item_id, label) => [crate::controls::autocomplete::AutocompleteEvent::Complete { item_id: "payload".into(), label: "payload".into() }],
on_clear() => [crate::controls::autocomplete::AutocompleteEvent::Clear],
on_focus_changed(focused) => [crate::controls::autocomplete::AutocompleteEvent::FocusChanged { focused: true }],
on_open_changed(open) => [crate::controls::autocomplete::AutocompleteEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::autocomplete::AutocompleteEvent::Dismiss]
]);

check_callbacks!(
search_selector_callbacks,
crate::controls::search_selector::SearchSelectorBuilder::new("callbacks", []),
[
on_value_changed(query) => [crate::controls::search_selector::SearchSelectorEvent::Change { query: "payload".into() }],
on_select(item_id, label) => [crate::controls::search_selector::SearchSelectorEvent::Select { item_id: "payload".into(), label: "payload".into() }],
on_complete(item_id, label) => [crate::controls::search_selector::SearchSelectorEvent::Complete { item_id: "payload".into(), label: "payload".into() }],
on_clear() => [crate::controls::search_selector::SearchSelectorEvent::Clear],
on_focus_changed(focused) => [crate::controls::search_selector::SearchSelectorEvent::FocusChanged { focused: true }],
on_open_changed(open) => [crate::controls::search_selector::SearchSelectorEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::search_selector::SearchSelectorEvent::Dismiss]
]);

check_callbacks!(
combo_box_callbacks,
crate::controls::combobox::ComboBoxBuilder::new("callbacks", []),
[
on_value_changed(query) => [crate::controls::combobox::ComboBoxEvent::Change { query: "payload".into() }],
on_select(item_id, label) => [crate::controls::combobox::ComboBoxEvent::Select { item_id: "payload".into(), label: "payload".into() }],
on_complete(item_id, label) => [crate::controls::combobox::ComboBoxEvent::Complete { item_id: "payload".into(), label: "payload".into() }],
on_clear() => [crate::controls::combobox::ComboBoxEvent::Clear],
on_focus_changed(focused) => [crate::controls::combobox::ComboBoxEvent::FocusChanged { focused: true }],
on_open_changed(open) => [crate::controls::combobox::ComboBoxEvent::OpenChanged { open: true }],
on_dismiss() => [crate::controls::combobox::ComboBoxEvent::Dismiss]
]);

check_callbacks!(
selection_panel_callbacks,
crate::controls::selection_panel::SelectionPanelBuilder::<crate::controls::selection_panel::SelectionPanelItem>::new("callbacks"),
[
on_hover_changed(visible_index) => [crate::controls::selection_panel::SelectionPanelEvent::HoverChanged { visible_index: None }],
on_activate_row(source_index, visible_index, item_id) => [crate::controls::selection_panel::SelectionPanelEvent::ActivateRow { source_index: 3, visible_index: 3, item_id: "payload".into() }],
on_active_index_changed(visible_index) => [crate::controls::selection_panel::SelectionPanelEvent::ActiveIndexChanged { visible_index: None }],
on_focus_changed(focused) => [crate::controls::selection_panel::SelectionPanelEvent::FocusChanged { focused: true }],
on_open_changed(open) => [crate::controls::selection_panel::SelectionPanelEvent::OpenChanged { open: true }]
]);

check_callbacks!(
toolbar_callbacks,
crate::controls::toolbar::ToolbarBuilder::new("callbacks"),
[
on_click(id) => [crate::controls::toolbar::ToolbarEvent::Click { id: "payload".into() }],
on_value_changed(id, value) => [crate::controls::toolbar::ToolbarEvent::Change { id: "payload".into(), value: crate::controls::toolbar::ToolbarValue::Bool(true) }]
]);

check_callbacks!(
radio_group_callbacks,
crate::controls::radio_group::new::<crate::controls::control_group::ControlGroupItem>("callbacks"),
[
on_value_changed(value) => [crate::controls::radio_group::RadioGroupEvent::Change { value: None }],
on_activate(value) => [crate::controls::radio_group::RadioGroupEvent::Activate { value: "payload".into() }],
on_focus_changed(focused) => [crate::controls::radio_group::RadioGroupEvent::FocusChanged { focused: true }],
on_item_focused(item_id) => [crate::controls::radio_group::RadioGroupEvent::ItemFocused { item_id: "payload".into() }]
]);

check_callbacks!(
radio_button_callbacks,
crate::controls::radio_button::new("callbacks"),
[
on_value_changed(selected) => [crate::controls::radio_button::RadioButtonEvent::Change { selected: true }],
on_focus_changed(focused) => [crate::controls::radio_button::RadioButtonEvent::FocusChanged { focused: true }],
on_enabled_changed(enabled) => [crate::controls::radio_button::RadioButtonEvent::EnabledChanged { enabled: true }],
on_hover_changed(hovered) => [crate::controls::radio_button::RadioButtonEvent::HoverChanged { hovered: true }]
]);

check_callbacks!(
control_group_callbacks,
crate::controls::control_group::ControlGroupBuilder::<crate::controls::control_group::ControlGroupItem>::new("callbacks"),
[
on_activate(activated_id) => [crate::controls::control_group::ControlGroupEvent::Activate { activated_id: "payload".into() }],
on_value_changed(changed_id, selected, selected_ids) => [crate::controls::control_group::ControlGroupEvent::Change { changed_id: "payload".into(), selected: true, selected_ids: Vec::new() }],
on_item_bounds_changed(item_id, bounds) => [crate::controls::control_group::ControlGroupEvent::ItemBoundsChanged { item_id: "payload".into(), bounds: Default::default() }],
on_focus_changed(focused) => [crate::controls::control_group::ControlGroupEvent::FocusChanged { focused: true }],
on_item_focused(item_id) => [crate::controls::control_group::ControlGroupEvent::ItemFocused { item_id: "payload".into() }]
]);

check_callbacks!(
tabs_callbacks,
crate::controls::tabs::TabsBuilder::new("callbacks"),
[
on_value_changed(tab_id, label) => [crate::controls::tabs::TabsEvent::Change { tab_id: "payload".into(), label: "payload".into() }],
on_activate(tab_id, label) => [crate::controls::tabs::TabsEvent::Activate { tab_id: "payload".into(), label: "payload".into() }],
on_reactivate(tab_id, label) => [crate::controls::tabs::TabsEvent::Reactivate { tab_id: "payload".into(), label: "payload".into() }],
on_dropdown_requested(tab_id, label, bounds) => [crate::controls::tabs::TabsEvent::DropdownRequested { tab_id: "payload".into(), label: "payload".into(), bounds: None }],
on_item_bounds_changed(tab_id, bounds) => [crate::controls::tabs::TabsEvent::ItemBoundsChanged { tab_id: "payload".into(), bounds: Default::default() }],
on_focus_changed(focused) => [crate::controls::tabs::TabsEvent::FocusChanged { focused: true }],
on_item_focused(tab_id, label) => [crate::controls::tabs::TabsEvent::ItemFocused { tab_id: "payload".into(), label: "payload".into() }]
]);

check_callbacks!(
sidebar_callbacks,
crate::controls::sidebar::SidebarControlBuilder::new("callbacks"),
[
on_presentation_changed(presentation) => [crate::controls::sidebar::SidebarEvent::PresentationChanged { presentation: Default::default() }],
on_select(id) => [crate::controls::sidebar::SidebarEvent::Select { id: "payload".into() }],
on_item_focused(id) => [crate::controls::sidebar::SidebarEvent::ItemFocused { id: "payload".into() }],
on_hover_changed(id) => [crate::controls::sidebar::SidebarEvent::HoverChanged { id: None }],
on_sub_menu_toggle(id, open) => [crate::controls::sidebar::SidebarEvent::SubMenuToggle { id: "payload".into(), open: true }],
on_enabled_changed(enabled) => [crate::controls::sidebar::SidebarEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
accordion_callbacks,
crate::controls::accordion::AccordionBuilder::new("callbacks"),
[
on_expanded_changed(item_id, expanded) => [crate::controls::accordion::AccordionEvent::ExpandedChanged { item_id: "payload".into(), expanded: true }],
on_focus_changed(focused) => [crate::controls::accordion::AccordionEvent::FocusChanged { focused: true }],
on_item_focused(item_id) => [crate::controls::accordion::AccordionEvent::ItemFocused { item_id: "payload".into() }],
on_item_hover_changed(item_id, hovered) => [crate::controls::accordion::AccordionEvent::ItemHoverChanged { item_id: "payload".into(), hovered: true }],
on_enabled_changed(enabled) => [crate::controls::accordion::AccordionEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
pager_callbacks,
crate::controls::pager::PagerBuilder::new("callbacks"),
[
on_page_changed(page) => [crate::controls::pager::PagerEvent::PageChanged { page: 3 }],
on_page_size_changed(page_size) => [crate::controls::pager::PagerEvent::PageSizeChanged { page_size: 3 }],
on_page_size_open_changed(open) => [crate::controls::pager::PagerEvent::PageSizeOpenChanged { open: true }],
on_enabled_changed(enabled) => [crate::controls::pager::PagerEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
dock_splitter_callbacks,
crate::controls::dock_splitter::DockSplitterBuilder::new("callbacks", crate::controls::dock_splitter::SplitterOrientation::Horizontal),
[
on_resize_start() => [crate::controls::dock_splitter::DockSplitterEvent::ResizeStart],
on_resize_delta(total_delta) => [crate::controls::dock_splitter::DockSplitterEvent::Resize { total_delta: 0.75 }],
on_resize_end() => [crate::controls::dock_splitter::DockSplitterEvent::ResizeEnd],
on_focus_changed(focused) => [crate::controls::dock_splitter::DockSplitterEvent::FocusChanged { focused: true }],
on_hover_changed(hovered) => [crate::controls::dock_splitter::DockSplitterEvent::HoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::dock_splitter::DockSplitterEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
split_view_callbacks,
crate::controls::split_view::SplitViewBuilder::new("callbacks"),
[
on_resize_start() => [crate::controls::split_view::SplitViewEvent::ResizeStart],
on_sidebar_width_changed(width) => [crate::controls::split_view::SplitViewEvent::SidebarWidthChanged { width: gpui::px(42.0) }],
on_resize_end(width) => [crate::controls::split_view::SplitViewEvent::ResizeEnd { width: gpui::px(42.0) }],
on_collapsed_changed(collapsed) => [crate::controls::split_view::SplitViewEvent::CollapsedChanged { collapsed: true }],
on_separator_hover_changed(hovered) => [crate::controls::split_view::SplitViewEvent::SeparatorHoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::split_view::SplitViewEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
scrollbar_callbacks,
crate::controls::scrollbar::ScrollbarBuilder::new("callbacks"),
[
on_value_changed(value) => [crate::controls::scrollbar::ScrollbarEvent::Change { value: 0.75 }],
on_drag_start() => [crate::controls::scrollbar::ScrollbarEvent::DragStart],
on_drag_end(value) => [crate::controls::scrollbar::ScrollbarEvent::DragEnd { value: 0.75 }],
on_focus_changed(focused) => [crate::controls::scrollbar::ScrollbarEvent::FocusChanged { focused: true }],
on_hover_changed(hovered) => [crate::controls::scrollbar::ScrollbarEvent::HoverChanged { hovered: true }],
on_enabled_changed(enabled) => [crate::controls::scrollbar::ScrollbarEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
resizable_panels_callbacks,
crate::controls::resizable_panels::ResizablePanelsBuilder::new("callbacks"),
[
on_resize_start() => [crate::controls::resizable_panels::ResizablePanelsEvent::ResizeStart],
on_sizes_changed(sizes_px) => [crate::controls::resizable_panels::ResizablePanelsEvent::SizesChanged { sizes_px: Vec::new() }],
on_resize_end(sizes_px) => [crate::controls::resizable_panels::ResizablePanelsEvent::ResizeEnd { sizes_px: Vec::new() }],
on_panel_hidden_changed(panel_index, hidden) => [crate::controls::resizable_panels::ResizablePanelsEvent::PanelHiddenChanged { panel_index: 3, hidden: true }],
on_panel_region_hidden_changed(region, panel_index, hidden) => [crate::controls::resizable_panels::ResizablePanelsEvent::PanelRegionHiddenChanged { region: crate::controls::resizable_panels::PanelId::new("region"), panel_index: 3, hidden: true }],
on_handle_focus_changed(handle_index, focused) => [crate::controls::resizable_panels::ResizablePanelsEvent::HandleFocusChanged { handle_index: 3, focused: true }],
on_handle_hover_changed(handle_index, hovered) => [crate::controls::resizable_panels::ResizablePanelsEvent::HandleHoverChanged { handle_index: 3, hovered: true }],
on_enabled_changed(enabled) => [crate::controls::resizable_panels::ResizablePanelsEvent::EnabledChanged { enabled: true }]
]);

check_callbacks!(
tree_view_callbacks,
crate::controls::tree_view::TreeViewBuilder::<()>::new("callbacks"),
[
on_drag_drop(event) => [],
on_node_activated(node_id, data) => [crate::controls::tree_view::TreeViewEvent::<()>::NodeActivated { node_id: "payload".into(), data: () }],
on_selection_policy_changed(policy) => [crate::controls::tree_view::TreeViewEvent::<()>::SelectionPolicyChanged { policy: crate::controls::tree_view::TreeViewSelectionPolicy::new(crate::controls::tree_view::TreeViewSelectionMode::Single) }],
on_node_expanded(node_id, data) => [crate::controls::tree_view::TreeViewEvent::<()>::NodeExpanded { node_id: "payload".into(), data: () }],
on_node_collapsed(node_id, data) => [crate::controls::tree_view::TreeViewEvent::<()>::NodeCollapsed { node_id: "payload".into(), data: () }],
on_selection_changed(selected_ids) => [crate::controls::tree_view::TreeViewEvent::<()>::SelectionChanged { selected_ids: std::collections::HashSet::new() }],
on_active_node_changed(node_id) => [crate::controls::tree_view::TreeViewEvent::<()>::ActiveNodeChanged { node_id: None }],
on_scroll_changed(top_index) => [crate::controls::tree_view::TreeViewEvent::<()>::ScrollChanged { top_index: 3 }],
on_focus_changed(focused) => [crate::controls::tree_view::TreeViewEvent::<()>::FocusChanged { focused: true }],
on_row_hover_changed(node_id, index, hovered) => [crate::controls::tree_view::TreeViewEvent::<()>::RowHoverChanged { node_id: "payload".into(), index: 3, hovered: true }],
on_enabled_changed(enabled) => [crate::controls::tree_view::TreeViewEvent::<()>::EnabledChanged { enabled: true }]
]);

check_callbacks!(
table_callbacks,
crate::controls::table::TableBuilder::<()>::new_typed("callbacks"),
[
on_row_drag(event) => [],
on_selection_changed(selected_indices) => [crate::controls::table::TableEvent::SelectionChanged { selected_indices: Vec::new() }],
on_selected_keys_changed(selected_keys) => [crate::controls::table::TableEvent::SelectedKeysChanged { selected_keys: Vec::new() }],
on_active_index_changed(active_index) => [crate::controls::table::TableEvent::ActiveIndexChanged { active_index: None }],
on_page_changed(page) => [crate::controls::table::TableEvent::PageChanged { page: 3 }],
on_page_size_changed(page_size) => [crate::controls::table::TableEvent::PageSizeChanged { page_size: 3 }],
on_scroll_changed(top_index) => [crate::controls::table::TableEvent::ScrollChanged { top_index: 3 }],
on_focus_changed(focused) => [crate::controls::table::TableEvent::FocusChanged { focused: true }],
on_row_hover_changed(index, hovered) => [crate::controls::table::TableEvent::RowHoverChanged { index: 3, hovered: true }],
on_enabled_changed(enabled) => [crate::controls::table::TableEvent::EnabledChanged { enabled: true }]
]);

#[test]
fn paging_table_preserves_inner_and_wrapper_callbacks() {
    use crate::controls::{
        pager::PagerBuilder,
        table::{PagingTableBuilder, TableBuilder},
    };
    let app = TestAppContext::single();
    let owner = app.update(|cx| cx.new(|_| Owner::default()));
    let table = app.update(|cx| {
        owner.update(cx, |_, cx| {
            let inner = TableBuilder::<()>::new_typed("inner")
                .on_selection_changed(cx, |owner: &mut Owner, _, _| owner.log.push("inner".into()));
            PagingTableBuilder::new(inner, PagerBuilder::new("pager"))
                .on_selection_changed(cx, |owner, _, _| owner.log.push("wrapper".into()))
                .spawn(cx)
        })
    });
    let inner = app.read(|cx| table.read(cx).list().clone());
    app.update(|cx| {
        inner.update(cx, |_, cx| {
            cx.emit(crate::controls::table::TableEvent::SelectionChanged { selected_indices: vec![] })
        })
    });
    app.read(|cx| {
        let log = &owner.read(cx).log;
        assert_eq!(log.len(), 2);
        assert!(log.iter().any(|s| s == "inner"));
        assert!(log.iter().any(|s| s == "wrapper"));
    });
}

check_callbacks!(table_drag_builder_callbacks, crate::controls::table::TableBuilder::<()>::new_typed("drag"), [
on_drag_started(source, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);

check_drag_entity_callbacks!(table_drag_entity_callbacks, crate::controls::table::TableBuilder::<()>::new_typed("drag"), [
on_drag_started(source, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);

check_callbacks!(paging_table_drag_builder_callbacks, crate::controls::table::PagingTableBuilder::new(crate::controls::table::TableBuilder::<()>::new_typed("drag"), crate::controls::pager::PagerBuilder::new("pager")), [
on_drag_started(source, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);

check_drag_entity_callbacks!(paging_table_drag_entity_callbacks, crate::controls::table::PagingTableBuilder::new(crate::controls::table::TableBuilder::<()>::new_typed("drag"), crate::controls::pager::PagerBuilder::new("pager")), [
on_drag_started(source, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::table::TableEvent::RowDrag(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);

check_callbacks!(tree_drag_builder_callbacks, crate::controls::tree_view::TreeViewBuilder::<()>::new("drag"), [
on_items_removed(source, target, node_id, node_ids) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()] })],
on_items_added(source, target, node_id, node_ids, location) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before } })],
on_items_reordered(target, node_id, node_ids, location) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsReordered { target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before } })],
on_drag_started(source, node_id, node_ids) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Started { source: gpui::EntityId::from(1), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()] })],
on_dropped(source, target, node_id, node_ids, location, changed) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before }, changed: false })],
on_drop_rejected(source, target, node_id, node_ids, error) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Rejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], error: "denied".into() })],
on_drag_ended(source, node_id, node_ids, reason) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Ended { source: gpui::EntityId::from(1), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], reason: crate::controls::tree_view::TreeDragEndReason::Unchanged })]
]);

check_drag_entity_callbacks!(tree_drag_entity_callbacks, crate::controls::tree_view::TreeViewBuilder::<()>::new("drag"), [
on_items_removed(source, target, node_id, node_ids) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()] })],
on_items_added(source, target, node_id, node_ids, location) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before } })],
on_items_reordered(target, node_id, node_ids, location) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::ItemsReordered { target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before } })],
on_drag_started(source, node_id, node_ids) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Started { source: gpui::EntityId::from(1), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()] })],
on_dropped(source, target, node_id, node_ids, location, changed) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], location: crate::controls::tree_view::TreeDropLocation { parent_id: None, anchor_id: Some("c".into()), position: crate::controls::tree_view::TreeDropPosition::Before }, changed: false })],
on_drop_rejected(source, target, node_id, node_ids, error) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Rejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], error: "denied".into() })],
on_drag_ended(source, node_id, node_ids, reason) => [crate::controls::tree_view::TreeViewEvent::<()>::DragDrop(crate::controls::tree_view::TreeViewDragEvent::Ended { source: gpui::EntityId::from(1), node_id: "a".into(), node_ids: vec!["a".into(), "b".into()], reason: crate::controls::tree_view::TreeDragEndReason::Unchanged })]
]);

#[test]
fn drag_aliases_and_named_handlers_coexist_and_can_update_emitting_control() {
    use crate::controls::table::{TableBuilder, TableControl, TableEvent};
    use crate::infra::drag_drop::DragDropEvent;
    #[derive(Default)]
    struct DragOwner {
        table: Option<gpui::Entity<TableControl<()>>>,
        log: Vec<&'static str>,
    }
    let app = TestAppContext::single();
    let capture = std::rc::Rc::new(());
    let owner = app.update(|cx| {
        cx.new(|cx| {
            let retained = capture.clone();
            let table = TableBuilder::<()>::new_typed("alias")
                .on_row_drag(cx, |owner: &mut DragOwner, _, _| owner.log.push("legacy"))
                .on_drag_drop(cx, |owner, _, _| owner.log.push("aggregate"))
                .on_dropped(cx, move |owner, source, target, keys, before, changed, cx| {
                    let _keep = &retained;
                    assert_eq!(source, target);
                    assert_eq!(keys, &[gpui::SharedString::from("row")]);
                    assert_eq!(before, &None);
                    assert!(!changed);
                    owner.log.push("completed");
                    owner.table.as_ref().unwrap().update(cx, |table, cx| table.set_enabled(false, cx));
                })
                .spawn(cx);
            DragOwner { table: Some(table), log: vec![] }
        })
    });
    let table = app.read(|cx| owner.read(cx).table.as_ref().unwrap().clone());
    app.update(|cx| {
        table.update(cx, |_, cx| {
            cx.emit(TableEvent::RowDrag(DragDropEvent::Dropped {
                source: cx.entity_id(),
                target: cx.entity_id(),
                keys: vec!["row".into()],
                before: None,
                changed: false,
            }))
        })
    });
    app.read(|cx| assert_eq!(owner.read(cx).log, ["legacy", "aggregate", "completed"]));
    let weak_owner = owner.downgrade();
    drop(owner);
    app.update(|_| {});
    assert!(weak_owner.upgrade().is_none());
    assert_eq!(std::rc::Rc::strong_count(&capture), 2);
    let weak_table = table.downgrade();
    drop(table);
    app.update(|_| {});
    assert!(weak_table.upgrade().is_none());
    assert_eq!(std::rc::Rc::strong_count(&capture), 1);
}

check_callbacks!(accordion_drag_builder_callbacks, crate::controls::accordion::AccordionBuilder::new("drag"), [
on_drag_started(source, keys) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);

check_drag_entity_callbacks!(accordion_drag_entity_callbacks, crate::controls::accordion::AccordionBuilder::new("drag"), [
on_drag_started(source, keys) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DragStarted { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()] })],
on_items_removed(source, target, keys) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsRemoved { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()] })],
on_items_added(source, target, keys, before) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsAdded { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_items_reordered(target, keys, before) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::ItemsReordered { target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()) })],
on_dropped(source, target, keys, before, changed) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::Dropped { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), changed: false })],
on_drop_rejected(source, target, keys, before, error) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DropRejected { source: gpui::EntityId::from(1), target: gpui::EntityId::from(2), keys: vec!["a".into(), "b".into()], before: Some("c".into()), error: "denied".into() })],
on_drag_ended(source, keys, reason) => [crate::controls::accordion::AccordionEvent::DragDrop(crate::infra::drag_drop::DragDropEvent::DragEnded { source: gpui::EntityId::from(1), keys: vec!["a".into(), "b".into()], reason: crate::infra::drag_drop::DragEndReason::Unchanged })]
]);
