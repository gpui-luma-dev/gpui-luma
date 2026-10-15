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
    pager_forwards_callbacks,
    crate::Pager::new("callbacks"),
    on_page_changed(_),
    gpui_luma::controls::pager::PagerEvent::PageChanged { page: 3 }
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
    menu_choice_group_forwards_callbacks,
    crate::MenuChoiceGroup::<gpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_, _, _),
    gpui_luma::controls::control_group::ControlGroupEvent::Change {
        changed_id: "payload".into(),
        selected: true,
        selected_ids: Vec::new()
    }
);
check_look!(
    search_selector_forwards_callbacks,
    crate::SearchSelector::new("callbacks", []),
    on_value_changed(_),
    gpui_luma::controls::search_selector::SearchSelectorEvent::Change { query: "payload".into() }
);
check_look!(
    table_forwards_callbacks,
    crate::Table::<String>::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::table::TableEvent::FocusChanged { focused: true }
);
check_look!(
    autocomplete_forwards_callbacks,
    crate::Autocomplete::new("callbacks", []),
    on_value_changed(_),
    gpui_luma::controls::autocomplete::AutocompleteEvent::Change { query: "payload".into() }
);
check_look!(
    popup_menu_forwards_callbacks,
    crate::PopupMenu::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::popup_menu::PopupMenuEvent::FocusChanged { focused: true }
);
check_look!(
    split_button_forwards_callbacks,
    crate::SplitButton::new("callbacks"),
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
    scrollbar_forwards_callbacks,
    crate::Scrollbar::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::scrollbar::ScrollbarEvent::Change { value: 0.75 }
);
check_look!(
    selector_forwards_callbacks,
    crate::Selector::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::selector::SelectorEvent::Change { item_id: "payload".into(), label: "payload".into() }
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
    accordion_forwards_callbacks,
    crate::Accordion::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::accordion::AccordionEvent::FocusChanged { focused: true }
);
check_look!(
    combo_box_forwards_callbacks,
    crate::ComboBox::new("callbacks", []),
    on_value_changed(_),
    gpui_luma::controls::combobox::ComboBoxEvent::Change { query: "payload".into() }
);
check_look!(
    tree_view_forwards_callbacks,
    crate::TreeView::<()>::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::tree_view::TreeViewEvent::<()>::FocusChanged { focused: true }
);
check_look!(
    tabs_forwards_callbacks,
    crate::Tabs::new("callbacks"),
    on_value_changed(_, _),
    gpui_luma::controls::tabs::TabsEvent::Change { tab_id: "payload".into(), label: "payload".into() }
);
check_look!(
    selection_panel_forwards_callbacks,
    crate::SelectionPanel::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::selection_panel::SelectionPanelEvent::FocusChanged { focused: true }
);
check_look!(
    radio_group_forwards_callbacks,
    crate::RadioGroup::<gpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_),
    gpui_luma::controls::radio_group::RadioGroupEvent::Change { value: None }
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
    sidebar_forwards_callbacks,
    crate::Sidebar::new("callbacks"),
    on_presentation_changed(_),
    gpui_luma::controls::sidebar::SidebarEvent::PresentationChanged { presentation: Default::default() }
);
check_look!(
    context_menu_forwards_callbacks,
    crate::ContextMenu::new("callbacks"),
    on_focus_changed(_),
    gpui_luma::controls::context_menu::ContextMenuEvent::FocusChanged { focused: true }
);
check_look!(
    icon_group_forwards_callbacks,
    crate::IconGroup::<gpui_luma::controls::control_group::ControlGroupItem>::new("callbacks"),
    on_value_changed(_, _, _),
    gpui_luma::controls::control_group::ControlGroupEvent::Change {
        changed_id: "payload".into(),
        selected: true,
        selected_ids: Vec::new()
    }
);
