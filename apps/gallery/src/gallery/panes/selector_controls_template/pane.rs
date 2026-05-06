use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, FontWeight, IntoElement, MouseDownEvent, MouseUpEvent, Pixels,
    Render, SharedString, TextRun, Window, div, font, prelude::*, px, svg,
};
use gpui_luma::controls::floating_menu::{
    DefaultFloatingMenuTheme, FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuTheme,
    render_floating_menu,
};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::search_selector;
use gpui_luma::controls::selector::{
    ControlFocusState, SelectorItem, SelectorPlacement, SelectorRenderModel, SelectorTemplate, SelectorTemplateHandlers,
};
use gpui_luma::controls::textfield::{
    TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTemplateHandlers, TextFieldTheme,
};
use gpui_luma::theme::InteractionState;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_description, notify_entity};

const SELECTOR_TEMPLATES_DESCRIPTION: &str = concat!(
    "Template matrix for selection controls. ",
    "Rows are interaction states; columns are selection controls."
);

#[derive(Clone)]
pub(in crate::gallery) struct SelectorControlsTemplatePane {
    state_preview: gpui::Entity<SelectorControlsTemplatePreview>,
}

impl SelectorControlsTemplatePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self { state_preview: cx.new(|_| SelectorControlsTemplatePreview::new(theme)) }
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_description(
            "Selector Templates",
            Some(SELECTOR_TEMPLATES_DESCRIPTION),
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.state_preview, cx);
    }
}

#[derive(Clone)]
struct SelectorControlsTemplatePreview {
    theme: GalleryThemePack,
    textfield_template: Arc<dyn TextFieldTemplate>,
    textfield_theme: Arc<dyn TextFieldTheme>,
    selector_template: Arc<dyn SelectorTemplate>,
}

#[derive(Clone, Copy)]
struct SelectorTemplateStateSample {
    id: &'static str,
    label: &'static str,
    textfield_state: TextFieldState,
    textfield_enabled: bool,
    selector_state: InteractionState,
    selector_focus: ControlFocusState,
    selector_enabled: bool,
}

#[derive(Clone, Copy)]
enum SelectorTemplateControl {
    AutocompleteTextBox,
    ComboBox,
    Selector,
    SearchSelector,
}

impl SelectorTemplateControl {
    fn header(self) -> &'static str {
        match self {
            Self::AutocompleteTextBox => "AutocompleteTextBox",
            Self::ComboBox => "ComboBox",
            Self::Selector => "Selector",
            Self::SearchSelector => "SearchSelector",
        }
    }
}

impl SelectorControlsTemplatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self {
            theme: theme.clone(),
            textfield_template: theme.textfield_template(),
            textfield_theme: theme.textfield_theme(),
            selector_template: theme.selector_template(),
        }
    }
}

impl Render for SelectorControlsTemplatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let states = [
            SelectorTemplateStateSample {
                id: "default",
                label: "Default",
                textfield_state: TextFieldState::default(),
                textfield_enabled: true,
                selector_state: InteractionState::default(),
                selector_focus: ControlFocusState::default(),
                selector_enabled: true,
            },
            SelectorTemplateStateSample {
                id: "hover",
                label: "Hover",
                textfield_state: TextFieldState { hovered: true, ..TextFieldState::default() },
                textfield_enabled: true,
                selector_state: InteractionState { hovered: true, ..InteractionState::default() },
                selector_focus: ControlFocusState::default(),
                selector_enabled: true,
            },
            SelectorTemplateStateSample {
                id: "focused",
                label: "Focused",
                textfield_state: TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
                textfield_enabled: true,
                selector_state: InteractionState { focused: true, ..InteractionState::default() },
                selector_focus: ControlFocusState { focused: true, focus_visible: true },
                selector_enabled: true,
            },
            SelectorTemplateStateSample {
                id: "pressed",
                label: "Pressed",
                textfield_state: TextFieldState {
                    hovered: true,
                    focused: true,
                    focus_visible: true,
                    ..TextFieldState::default()
                },
                textfield_enabled: true,
                selector_state: InteractionState {
                    hovered: true,
                    focused: true,
                    pressed: true,
                    ..InteractionState::default()
                },
                selector_focus: ControlFocusState { focused: true, focus_visible: true },
                selector_enabled: true,
            },
            SelectorTemplateStateSample {
                id: "disabled",
                label: "Disabled",
                textfield_state: TextFieldState::default(),
                textfield_enabled: false,
                selector_state: InteractionState { disabled: true, ..InteractionState::default() },
                selector_focus: ControlFocusState::default(),
                selector_enabled: false,
            },
        ];
        let controls = [
            SelectorTemplateControl::AutocompleteTextBox,
            SelectorTemplateControl::ComboBox,
            SelectorTemplateControl::Selector,
            SelectorTemplateControl::SearchSelector,
        ];

        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(render_header_row(&controls, chrome.muted_text))
            .children(
                states.iter().map(|state| render_state_row(self, state, &controls, chrome.muted_text, window, cx)),
            )
    }
}

fn render_header_row(controls: &[SelectorTemplateControl], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().w(px(28.0)))
        .children(controls.iter().map(|control| {
            div()
                .w(px(180.0))
                .flex()
                .justify_center()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(control.header())
        }))
        .into_any_element()
}

fn render_state_row(
    preview: &SelectorControlsTemplatePreview,
    state: &SelectorTemplateStateSample,
    controls: &[SelectorTemplateControl],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_stretch()
        .gap(px(10.0))
        .child(render_vertical_state_rail(state.label, state.id, label_color))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(10.0))
                .children(controls.iter().map(|control| render_control_cell(preview, *control, state, window, cx))),
        )
        .into_any_element()
}

fn render_control_cell(
    preview: &SelectorControlsTemplatePreview,
    control: SelectorTemplateControl,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("selector-controls-template-{}-{}", state.id, control.header()));

    let trigger = match control {
        SelectorTemplateControl::AutocompleteTextBox => {
            render_textfield_like_trigger(preview, &id, "Type to filter…", None, state, window, cx)
        }
        SelectorTemplateControl::ComboBox => render_textfield_like_trigger(
            preview,
            &id,
            "Strict mode (exact match only)…",
            Some(LucideIcon::ChevronDown),
            state,
            window,
            cx,
        ),
        SelectorTemplateControl::SearchSelector => {
            render_textfield_like_trigger(preview, &id, "Choose a state…", Some(LucideIcon::Search), state, window, cx)
        }
        SelectorTemplateControl::Selector => render_selector_trigger(preview, &id, state, window, cx),
    };

    let popup = if state.id == "pressed" {
        Some(render_popup_preview(preview, &id, control, window, cx))
    } else {
        None
    };

    div()
        .w(px(180.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_start()
        .gap(px(6.0))
        .child(trigger)
        .when_some(popup, |root, popup| root.child(popup))
        .into_any_element()
}

fn render_textfield_like_trigger(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    placeholder: &'static str,
    right_icon: Option<LucideIcon>,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let value = SharedString::from("California");
    let placeholder = SharedString::from(placeholder);
    let character_offsets = textfield_character_offsets(
        value.as_ref(),
        preview.textfield_theme.clone(),
        state.textfield_state,
        state.textfield_enabled,
        window,
    );

    let model = TextFieldRenderModel {
        id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: None,
        enabled: state.textfield_enabled,
        full_width: true,
        state: state.textfield_state,
        caret_visible: false,
        horizontal_scroll: 0.0,
        character_offsets,
    };

    let field = preview
        .textfield_template
        .render(&model, textfield_preview_handlers(), window, cx)
        .into_any_element();

    div()
        .w(px(168.0))
        .relative()
        .child(field)
        .when_some(right_icon, |root, icon| {
            root.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .right(px(10.0))
                    .h_full()
                    .w(px(18.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(preview.theme.chrome().muted_text)
                    .child(
                        div()
                            .font_family("lucide")
                            .text_size(px(12.0))
                            .line_height(px(12.0))
                            .child(char::from(icon).to_string()),
                    ),
            )
        })
        .into_any_element()
}

fn render_selector_trigger(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let label = SharedString::from("Select status");
    let items = selector_items().into_iter().collect::<Vec<_>>();
    let model = SelectorRenderModel {
        id,
        label: &label,
        selected_icon: None,
        selected_index: None,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: SelectorPlacement::BelowStart,
        active_path: None,
        enabled: state.selector_enabled,
        item_template: None,
        focus: state.selector_focus,
        state: state.selector_state,
    };

    preview
        .selector_template
        .render(&model, selector_preview_handlers(items.len()), window, cx)
        .w(px(168.0))
        .into_any_element()
}

fn render_popup_preview(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    control: SelectorTemplateControl,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let appearance = DefaultFloatingMenuTheme::new(preview.theme.tokens()).resolve();
    let popup_id = SharedString::from(format!("{id}-popup-preview"));
    let items = popup_items_for_control(control);
    let item_hovers = (0..items.len()).map(|_| Box::new(noop_hover) as FloatingMenuHoverHandler).collect::<Vec<_>>();
    let item_clicks = (0..items.len()).map(|_| Box::new(noop_click) as FloatingMenuClickHandler).collect::<Vec<_>>();

    let menu = render_floating_menu(
        &popup_id,
        &items,
        None,
        Some(gpui_luma::controls::state::MenuPath::Root(0)),
        appearance.clone(),
        item_hovers,
        item_clicks,
    );

    if matches!(control, SelectorTemplateControl::SearchSelector) {
        let search_id = SharedString::from(format!("{id}-popup-search-preview"));
        let search_placeholder = SharedString::from("Selection search");
        let search_value = SharedString::from("");
        let search_offsets = textfield_character_offsets(
            search_value.as_ref(),
            preview.textfield_theme.clone(),
            TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
            true,
            window,
        );

        let search_model = TextFieldRenderModel {
            id: &search_id,
            placeholder: &search_placeholder,
            value: &search_value,
            prefix_icon: None,
            enabled: true,
            full_width: true,
            state: TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
            caret_visible: false,
            horizontal_scroll: 0.0,
            character_offsets: search_offsets,
        };

        return div()
            .w(px(168.0))
            .child(
                div()
                    .border_1()
                    .border_color(appearance.border)
                    .rounded(px(appearance.radius))
                    .bg(appearance.background)
                    .overflow_hidden()
                    .child(div().p(px(8.0)).child(preview.textfield_template.render(
                        &search_model,
                        textfield_preview_handlers(),
                        window,
                        cx,
                    )))
                    .child(div().h(px(1.0)).bg(appearance.border))
                    .child(search_selector::render_popup_rows(
                        &popup_id,
                        &items,
                        appearance.clone(),
                        Some(0),
                        (0..items.len()).map(|_| Box::new(noop_hover) as FloatingMenuHoverHandler).collect::<Vec<_>>(),
                        (0..items.len()).map(|_| Box::new(noop_click) as FloatingMenuClickHandler).collect::<Vec<_>>(),
                    )),
            )
            .into_any_element();
    }

    div().w(px(168.0)).child(menu).into_any_element()
}

fn popup_items_for_control(control: SelectorTemplateControl) -> Vec<MenuItem> {
    match control {
        SelectorTemplateControl::AutocompleteTextBox => vec![
            MenuItem::new("autocomplete-preview-item-1").label("Alabama"),
            MenuItem::new("autocomplete-preview-item-2").label("Alaska"),
            MenuItem::new("autocomplete-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::ComboBox => vec![
            MenuItem::new("combobox-preview-item-1").label("California"),
            MenuItem::new("combobox-preview-item-2").label("Colorado"),
            MenuItem::new("combobox-preview-item-3").label("Connecticut"),
        ],
        SelectorTemplateControl::Selector => vec![
            MenuItem::new("selector-preview-item-1").label("Alabama"),
            MenuItem::new("selector-preview-item-2").label("Alaska"),
            MenuItem::new("selector-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::SearchSelector => vec![
            MenuItem::new("search-selector-preview-item-1").label("Alabama"),
            MenuItem::new("search-selector-preview-item-2").label("Alaska"),
            MenuItem::new("search-selector-preview-item-3").label("Arizona"),
        ],
    }
}

fn selector_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

// NOTE: Row-label SVGs are authored with text centered at the viewBox center
// (`x=12, y=60` in `0 0 24 120`) and rotated around that same center.
// If you add new rotated rail labels, keep that geometry model:
// `text-anchor=middle`, `dominant-baseline=middle`,
// `transform=rotate(-90 12 60)`.
// This keeps labels visually centered against the vertical divider line.
fn state_label_asset_path(state_id: &str) -> &'static str {
    match state_id {
        "default" => "assets/labels/default-label.svg",
        "hover" => "assets/labels/hover-label.svg",
        "focused" => "assets/labels/focused-label.svg",
        "pressed" => "assets/labels/pressed-label.svg",
        "disabled" => "assets/labels/disabled-label.svg",
        _ => "assets/labels/default-label.svg",
    }
}

fn render_vertical_state_rail(label: &'static str, state_id: &str, label_color: gpui::Hsla) -> AnyElement {
    div()
        .id(format!("selector-template-state-rail-{label}"))
        .w(px(28.0))
        .min_h(px(38.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(svg().path(state_label_asset_path(state_id)).w(px(20.0)).h(px(38.0)).text_color(label_color))
        .child(div().w(px(1.0)).h_full().bg(label_color))
        .into_any_element()
}

fn textfield_character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
) -> Vec<f32> {
    let appearance = theme.resolve(state, enabled);
    let value_shared = SharedString::from(value.to_string());
    let run = TextRun {
        len: value_shared.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = appearance.typography.weight;
            font
        },
        color: appearance.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(value_shared, px(appearance.typography.size), &[run], None);
    let chars = value.chars().count();

    let mut offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = char_to_byte_offset(value, char_offset);
        offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    offsets
}

fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    if char_offset == 0 {
        return 0;
    }

    text.char_indices().nth(char_offset).map_or(text.len(), |(byte, _)| byte)
}

fn textfield_preview_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(noop_textfield_hover),
        mouse_down: Box::new(noop_textfield_mouse_down),
        mouse_move: Box::new(noop_textfield_mouse_move),
        mouse_up: Box::new(noop_textfield_mouse_up),
        mouse_up_out: Box::new(noop_textfield_mouse_up),
        click: Box::new(noop_textfield_click),
        key_down: Box::new(noop_textfield_key_down),
    }
}

fn selector_preview_handlers(root_count: usize) -> SelectorTemplateHandlers {
    SelectorTemplateHandlers {
        trigger_bounds: Box::new(noop_bounds),
        trigger_click: Box::new(noop_click),
        trigger_hover: Box::new(noop_hover),
        trigger_mouse_down: Box::new(noop_mouse_down),
        trigger_mouse_up: Box::new(noop_mouse_up),
        trigger_mouse_up_out: Box::new(noop_mouse_up),
        root_mouse_down_out: Box::new(noop_mouse_down),
        item_hovers: (0..root_count).map(|_| Box::new(noop_hover) as _).collect(),
        item_clicks: (0..root_count).map(|_| Box::new(noop_click) as _).collect(),
    }
}

fn noop_textfield_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_textfield_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_textfield_mouse_move(_: &gpui::MouseMoveEvent, _: &mut Window, _: &mut App) {}
fn noop_textfield_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_textfield_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_textfield_key_down(_: &gpui::KeyDownEvent, _: &mut Window, _: &mut App) {}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
