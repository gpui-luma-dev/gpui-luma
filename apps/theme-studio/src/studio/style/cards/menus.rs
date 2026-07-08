use super::super::*;
use super::buttons::render_lucide_icon;
use super::inputs::{
    input_noop_bounds, input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};

pub(in crate::studio::style::style_guide) fn render_menu_template_state_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("menu-trigger"));

    section_shell_with_width(
        960.0,
        "Menus",
        "Menu triggers and floating menu panels.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        gpui::hsla(0.0, 0.0, 0.0, 0.0),
        render_menus_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, window, cx),
    )
}

fn render_menus_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "trigger-sizes" => render_menu_trigger_size_matrix(&look, window, cx),
        "floating-menu" => render_floating_menu_template_preview(&look),
        "sizes" => render_floating_menu_size_matrix(&look),
        _ => render_menu_trigger_template_matrix(&look, window, cx),
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

const MENU_TRIGGER_TABLE_STATE_COLUMN_WIDTH: f32 = 152.0;
const MENU_TRIGGER_TABLE_RADIUS_COLUMN_WIDTH: f32 = 136.0;
const MENU_TRIGGER_TABLE_SIZE_RADIUS_ROW_HEIGHT: f32 = 55.0;
const MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH: f32 = 120.0;
const MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT: f32 = 40.0;

const MENU_TRIGGER_SIZE_PREVIEW_STYLE: PopupMenuTriggerStyle = PopupMenuTriggerStyle::Outline;

const MENU_TRIGGER_SIZES: [(ButtonSize, &'static str); 3] =
    [(ButtonSize::Sm, "Small"), (ButtonSize::Md, "Medium"), (ButtonSize::Lg, "Large")];

struct MenuTriggerStyleDef {
    label: &'static str,
    description: &'static str,
    style: PopupMenuTriggerStyle,
}

const MENU_TRIGGER_STYLE_VARIANTS: [MenuTriggerStyleDef; 2] = [
    MenuTriggerStyleDef {
        label: "Outline",
        description: "Bordered menu trigger",
        style: PopupMenuTriggerStyle::Outline,
    },
    MenuTriggerStyleDef { label: "Ghost", description: "Quiet menu trigger", style: PopupMenuTriggerStyle::Ghost },
];

fn menu_trigger_state_samples() -> [ButtonStateSample; 5] {
    [
        ButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
        ButtonStateSample {
            id: "hover",
            header: "hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "focused",
            header: "focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ]
}

fn render_menu_trigger_template_matrix(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.popup_menu_template();
    let samples = menu_trigger_state_samples();
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .state_column_width(MENU_TRIGGER_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_menu_trigger_state_header_cell(sample, chrome.muted_text)))
    .rows(MENU_TRIGGER_STYLE_VARIANTS.iter().map(|row| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_menu_trigger_state_cell(&template, look, row.style, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_menu_trigger_size_matrix(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.popup_menu_template();
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(MENU_TRIGGER_TABLE_RADIUS_COLUMN_WIDTH)
            .header_height(MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(MENU_TRIGGER_TABLE_SIZE_RADIUS_ROW_HEIGHT)
            .variant_column_align_center(true),
    )
    .row_group_label("SIZE")
    .column_headers(
        ButtonRadiusPreset::ALL
            .iter()
            .map(|preset| render_menu_trigger_radius_header_cell(preset.label(), chrome.muted_text)),
    )
    .rows(MENU_TRIGGER_SIZES.iter().map(|(size, label)| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: ButtonRadiusPreset::ALL
                .iter()
                .map(|radius| {
                    render_menu_trigger_size_radius_cell(
                        &template,
                        look,
                        MENU_TRIGGER_SIZE_PREVIEW_STYLE,
                        *size,
                        *radius,
                        window,
                        cx,
                    )
                })
                .collect(),
        }
    }))
    .build()
}

fn render_floating_menu_size_matrix(look: &Arc<ShadcnLook>) -> AnyElement {
    let chrome = look.chrome();

    super::super::variant_state_table::VariantStateTable::new(
        super::super::variant_state_table::VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(MENU_TRIGGER_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
            .state_column_width(280.0)
            .header_height(MENU_TRIGGER_TABLE_SIZE_HEADER_HEIGHT)
            .row_height(160.0)
            .variant_column_align_center(false),
    )
    .row_group_label("SIZE")
    .column_headers([render_floating_menu_size_header_cell("Menu items", chrome.muted_text)])
    .rows(MENU_TRIGGER_SIZES.iter().map(|(size, label)| {
        super::super::variant_state_table::VariantStateTableRow {
            label: SharedString::from(*label),
            description: SharedString::from(""),
            cells: vec![render_floating_menu_size_cell(look, *size)],
        }
    }))
    .build()
}

fn render_floating_menu_size_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(label)
        .into_any_element()
}

fn render_floating_menu_size_cell(look: &Arc<ShadcnLook>, size: ButtonSize) -> AnyElement {
    let menu_look = floating_menu_look_for_size(look, size);
    let id = SharedString::from(format!("theme-studio-floating-menu-size-{}", menu_trigger_size_id(size)));
    let items = floating_menu_default_items();

    div()
        .w_full()
        .h_full()
        .flex()
        .items_start()
        .justify_center()
        .py(px(8.0))
        .child(render_floating_menu(
            &id,
            &items,
            None,
            None,
            menu_look,
            floating_menu_noop_hovers(items.len()),
            floating_menu_noop_clicks(items.len()),
        ))
        .into_any_element()
}

fn floating_menu_look_for_size(look: &Arc<ShadcnLook>, size: ButtonSize) -> FloatingMenuLook {
    gpui_luma_look_shadcn::paint::floating_menu_look(look.mode_tokens().as_ref(), look.mode(), size)
}

fn render_menu_trigger_state_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    _look: &ShadcnLook,
    trigger_style: PopupMenuTriggerStyle,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "theme-studio-menu-trigger-preview-{}-{}",
        menu_trigger_style_id(trigger_style),
        sample.id
    ));
    let label = SharedString::from("Menu");
    let items = popup_menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style,
        trigger_size: ButtonSize::Md,
        trigger_icon: None,
        without_elevation: false,
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        enabled: !sample.state.disabled,
        focus: menu_trigger_focus_for_sample(sample),
        state: sample.state,
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .into_any_element()
}

fn render_menu_trigger_size_radius_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    look: &ShadcnLook,
    trigger_style: PopupMenuTriggerStyle,
    size: ButtonSize,
    radius: ButtonRadiusPreset,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "theme-studio-menu-trigger-size-preview-{}-{}-{}",
        menu_trigger_style_id(trigger_style),
        menu_trigger_size_id(size),
        menu_trigger_radius_id(radius),
    ));
    let label = SharedString::from("Menu");
    let items = popup_menu_items().into_iter().collect::<Vec<_>>();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style,
        trigger_size: size,
        trigger_icon: None,
        without_elevation: false,
        trigger_radius_override: Some(menu_trigger_radius_px(look, size, radius)),
        open_submenu: None,
        active_path: None,
        enabled: true,
        focus: PopupMenuControlFocusState::default(),
        state: InteractionState::default(),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, popup_menu_preview_handlers(items.len(), 0), window, cx))
        .into_any_element()
}

fn menu_trigger_focus_for_sample(sample: &ButtonStateSample) -> PopupMenuControlFocusState {
    match sample.id {
        "focused" | "pressed" => PopupMenuControlFocusState { focused: true, focus_visible: true },
        _ => PopupMenuControlFocusState::default(),
    }
}

fn menu_trigger_radius_px(look: &ShadcnLook, size: ButtonSize, radius: ButtonRadiusPreset) -> f32 {
    gpui_luma_look_shadcn::paint::button_look_semantic(
        look.mode_tokens().as_ref(),
        look.mode(),
        ShadcnButtonStyle::Outline,
        ButtonFamilyRole::Text,
        size,
        Some(radius),
        InteractionState::default(),
    )
    .radius
}

fn menu_trigger_style_id(style: PopupMenuTriggerStyle) -> &'static str {
    match style {
        PopupMenuTriggerStyle::Outline => "outline",
        PopupMenuTriggerStyle::Ghost => "ghost",
    }
}

fn menu_trigger_size_id(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Sm => "sm",
        ButtonSize::Md => "md",
        ButtonSize::Lg => "lg",
    }
}

fn menu_trigger_radius_id(radius: ButtonRadiusPreset) -> &'static str {
    match radius {
        ButtonRadiusPreset::None => "none",
        ButtonRadiusPreset::Small => "small",
        ButtonRadiusPreset::Medium => "medium",
        ButtonRadiusPreset::Large => "large",
        ButtonRadiusPreset::Full => "full",
    }
}

fn render_menu_trigger_state_header_cell(sample: &ButtonStateSample, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(
            div()
                .text_color(muted_text)
                .child(render_lucide_icon(menu_trigger_state_header_icon(sample.id), 16.0)),
        )
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted_text)
                .child(menu_trigger_state_display_label(sample.header)),
        )
        .into_any_element()
}

fn render_menu_trigger_radius_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(label)
        .into_any_element()
}

fn menu_trigger_state_header_icon(state_id: &'static str) -> LucideIcon {
    match state_id {
        "default" => LucideIcon::House,
        "hover" => LucideIcon::MousePointer2,
        "focused" => LucideIcon::SquareDashed,
        "pressed" => LucideIcon::ArrowDown,
        "disabled" => LucideIcon::CircleMinus,
        _ => LucideIcon::House,
    }
}

fn menu_trigger_state_display_label(header: &'static str) -> &'static str {
    match header {
        "default" => "Default",
        "hover" => "Hover",
        "focused" => "Focused",
        "pressed" => "Pressed",
        "disabled" => "Disabled",
        _ => header,
    }
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
        "Tabs",
        "Inactive, active, hover, focus, pressed, and disabled.",
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

fn render_floating_menu_template_preview(look: &Arc<ShadcnLook>) -> AnyElement {
    let chrome = look.chrome();
    let menu_look = floating_menu_look_for_size(look, ButtonSize::Md);

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
        ))
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
