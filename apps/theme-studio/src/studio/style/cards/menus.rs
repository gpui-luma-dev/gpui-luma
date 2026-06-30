use super::super::*;
use super::inputs::{
    input_noop_bounds, input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
    render_input_section_heading,
};

pub(in crate::studio::style::style_guide) fn render_menu_template_state_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Menu Template State Preview",
        "Combines the Gallery floating-menu and popup-menu template state previews into a single style-guide panel.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(render_popup_menu_template_preview(&look, window, cx))
            .child(render_floating_menu_template_preview(&look))
            .into_any_element(),
    )
}

pub(in crate::studio::style::style_guide) fn render_tabs_navigation_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template = look.tabs_navigation_template();
    let samples = tabs_navigation_state_samples();

    section_shell_with_width(
        960.0,
        "Tabs Navigation Template State Preview",
        "Copied from Gallery. It previews inactive, active, hover, focus, pressed, and disabled tabs using the shared tabs-navigation template.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_tabs_navigation_state_sample(&template, sample, chrome.muted_text, window, cx)
                }),
            ))
            .into_any_element(),
    )
}

fn render_popup_menu_template_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let template = look.popup_menu_template();

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_input_section_heading("Popup Menu", chrome.muted_text))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(14.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.muted_text)
                        .child("Outline trigger state preview"),
                )
                .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    popup_menu_outline_samples().into_iter().map(|sample| {
                        render_popup_menu_trigger_sample(&template, sample, chrome.muted_text, window, cx)
                    }),
                ))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.muted_text)
                        .child("Ghost trigger state preview"),
                )
                .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    popup_menu_ghost_samples().into_iter().map(|sample| {
                        render_popup_menu_trigger_sample(&template, sample, chrome.muted_text, window, cx)
                    }),
                )),
        )
        .into_any_element()
}

fn render_floating_menu_template_preview(look: &Arc<ShadcnLook>) -> AnyElement {
    let chrome = look.chrome();
    let menu_look = look.floating_menu_theme().resolve();

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_input_section_heading("Floating Menu", chrome.muted_text))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_start()
                .justify_center()
                .gap(px(16.0))
                .child(render_floating_menu_state_sample(
                    "Standard",
                    &menu_look,
                    chrome.muted_text,
                    &floating_menu_default_items(),
                    None,
                ))
                .child(render_floating_menu_state_sample(
                    "Hover / active item",
                    &menu_look,
                    chrome.muted_text,
                    &floating_menu_default_items(),
                    Some(MenuPath::Root(1)),
                ))
                .child(render_floating_menu_state_sample(
                    "Disabled item",
                    &menu_look,
                    chrome.muted_text,
                    &floating_menu_disabled_items(),
                    None,
                ))
                .child(render_floating_menu_state_sample(
                    "Submenu affordance",
                    &menu_look,
                    chrome.muted_text,
                    &floating_menu_submenu_items(),
                    None,
                )),
        )
        .child(div().text_sm().text_color(chrome.muted_text).child(floating_menu_state_machine_snapshot()))
        .into_any_element()
}

fn render_popup_menu_trigger_sample(
    template: &Arc<dyn PopupMenuTemplate>,
    sample: PopupMenuStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-popup-menu-preview-trigger-{}", sample.id));
    let label = SharedString::from("Popup");
    let items = popup_menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style: sample.trigger_style,
        trigger_size: ControlSize::Md,
        trigger_icon: None,
        without_elevation: false,
        open_submenu: None,
        active_path: None,
        enabled: !sample.state.disabled,
        focus: sample.focus,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .child(div().text_xs().line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn render_floating_menu_state_sample(
    label: &'static str,
    look: &FloatingMenuLook,
    label_color: gpui::Hsla,
    items: &[MenuItem],
    active_path: Option<MenuPath>,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-floating-menu-sample-{}", menu_sample_id(label)));
    let root_count = items.len();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(7.0))
        .child(render_floating_menu(
            &id,
            items,
            None,
            active_path,
            look.clone(),
            floating_menu_noop_hovers(root_count),
            floating_menu_noop_clicks(root_count),
        ))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}

fn render_tabs_navigation_state_sample(
    template: &Arc<dyn TabsNavigationTemplate>,
    sample: TabsNavigationStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-tabs-navigation-preview-{}", sample.id));
    let items = tabs_navigation_preview_tabs();
    let active_id = items.get(sample.active_index).map(TabsNavigationItem::id);
    let render_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_enabled = sample.enabled && item.is_enabled();
            let active = active_id.is_some_and(|active_id| active_id == item.id());
            let mut state = TabsNavigationItemState { selected: active, ..TabsNavigationItemState::default() };

            if index == sample.target_index {
                state = sample.target_state;
                state.selected = active;
            }

            if !item_enabled {
                state.disabled = true;
                state.hovered = false;
                state.pressed = false;
                state.active = false;
                state.focus_visible = false;
            }

            TabsNavigationRenderItem { id: item.id(), label: item.label_text(), active, enabled: item_enabled, state }
        })
        .collect::<Vec<_>>();
    let model = TabsNavigationRenderModel {
        id: &id,
        size: ControlSize::Md,
        width_mode: TabsNavigationWidthMode::Intrinsic,
        items: render_items,
        active_id,
        enabled: sample.enabled,
        focus: TabsControlFocusState {
            focused: sample.target_state.active,
            focus_visible: sample.target_state.focus_visible,
        },
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, tabs_navigation_preview_handlers(items.len()), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn popup_menu_outline_samples() -> [PopupMenuStateSample; 5] {
    [
        PopupMenuStateSample {
            id: "default",
            label: "Standard",
            trigger_style: PopupMenuTriggerStyle::Outline,
            state: InteractionState::default(),
            focus: PopupMenuControlFocusState::default(),
        },
        PopupMenuStateSample {
            id: "hover",
            label: "Hover",
            trigger_style: PopupMenuTriggerStyle::Outline,
            state: InteractionState { hovered: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState::default(),
        },
        PopupMenuStateSample {
            id: "focus",
            label: "Focus",
            trigger_style: PopupMenuTriggerStyle::Outline,
            state: InteractionState { focused: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState { focused: true, focus_visible: true },
        },
        PopupMenuStateSample {
            id: "active",
            label: "Active",
            trigger_style: PopupMenuTriggerStyle::Outline,
            state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState { focused: true, focus_visible: true },
        },
        PopupMenuStateSample {
            id: "disabled",
            label: "Disabled",
            trigger_style: PopupMenuTriggerStyle::Outline,
            state: InteractionState { disabled: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState::default(),
        },
    ]
}

fn popup_menu_ghost_samples() -> [PopupMenuStateSample; 3] {
    [
        PopupMenuStateSample {
            id: "ghost-default",
            label: "Standard",
            trigger_style: PopupMenuTriggerStyle::Ghost,
            state: InteractionState::default(),
            focus: PopupMenuControlFocusState::default(),
        },
        PopupMenuStateSample {
            id: "ghost-hover",
            label: "Hover",
            trigger_style: PopupMenuTriggerStyle::Ghost,
            state: InteractionState { hovered: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState::default(),
        },
        PopupMenuStateSample {
            id: "ghost-disabled",
            label: "Disabled",
            trigger_style: PopupMenuTriggerStyle::Ghost,
            state: InteractionState { disabled: true, ..InteractionState::default() },
            focus: PopupMenuControlFocusState::default(),
        },
    ]
}

fn tabs_navigation_state_samples() -> [TabsNavigationStateSample; 7] {
    [
        TabsNavigationStateSample {
            id: "inactive",
            label: "Inactive",
            active_index: 1,
            target_index: 0,
            target_state: TabsNavigationItemState::default(),
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "active",
            label: "Active",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState { selected: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "hover",
            label: "Hover",
            active_index: 1,
            target_index: 0,
            target_state: TabsNavigationItemState { hovered: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "focus",
            label: "Focus",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                active: true,
                focus_visible: true,
                ..TabsNavigationItemState::default()
            },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "pressed",
            label: "Pressed",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                active: true,
                hovered: true,
                pressed: true,
                focus_visible: true,
                ..TabsNavigationItemState::default()
            },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "disabled-item",
            label: "Disabled item",
            active_index: 1,
            target_index: 2,
            target_state: TabsNavigationItemState { disabled: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "disabled-list",
            label: "Disabled list",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                disabled: true,
                ..TabsNavigationItemState::default()
            },
            enabled: false,
        },
    ]
}

fn popup_menu_preview_handlers(root_count: usize, submenu_click_count: usize) -> PopupMenuTemplateHandlers {
    let click_count = root_count + submenu_click_count;

    PopupMenuTemplateHandlers {
        trigger_bounds: Box::new(input_noop_bounds),
        trigger_click: Box::new(input_noop_click),
        trigger_hover: Box::new(input_noop_hover),
        trigger_mouse_down: Box::new(input_noop_mouse_down),
        trigger_mouse_up: Box::new(input_noop_mouse_up),
        trigger_mouse_up_out: Box::new(input_noop_mouse_up),
        root_mouse_down_out: Box::new(input_noop_mouse_down),
        item_hovers: (0..root_count).map(|_| Box::new(input_noop_hover) as _).collect(),
        item_clicks: (0..click_count).map(|_| Box::new(input_noop_click) as _).collect(),
    }
}

fn tabs_navigation_preview_handlers(count: usize) -> TabsNavigationTemplateHandlers {
    TabsNavigationTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(input_noop_hover) as TabsNavigationHoverHandler).collect(),
        item_mouse_downs: (0..count)
            .map(|_| Box::new(input_noop_mouse_down) as TabsNavigationMouseDownHandler)
            .collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(input_noop_click) as TabsNavigationClickHandler).collect(),
    }
}

fn popup_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn floating_menu_default_items() -> [MenuItem; 3] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
    ]
}

fn floating_menu_disabled_items() -> [MenuItem; 3] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download).enabled(false),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2),
    ]
}

fn floating_menu_submenu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("inspect").label("Inspect"),
    ]
}

fn floating_menu_state_machine_snapshot() -> String {
    let items = floating_menu_submenu_items().into_iter().collect::<Vec<_>>();
    let mut state = FloatingMenuState::default();
    let _ = state.open_with(Some(MenuPath::Root(0)));
    let _ = state.step(&items, FloatingMenuStepDirection::Next);
    let _ = state.open_active_submenu(&items);
    format!("State machine demo: active_path={:?}, open_submenu={:?}", state.active_path(), state.open_submenu())
}

fn menu_sample_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn floating_menu_noop_hovers(count: usize) -> Vec<FloatingMenuHoverHandler> {
    (0..count).map(|_| Box::new(input_noop_hover) as FloatingMenuHoverHandler).collect()
}

fn floating_menu_noop_clicks(count: usize) -> Vec<FloatingMenuClickHandler> {
    (0..count).map(|_| Box::new(input_noop_click) as FloatingMenuClickHandler).collect()
}

fn tabs_navigation_preview_tabs() -> [TabsNavigationItem; 3] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}
