use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, FontWeight, IntoElement, KeyDownEvent, MouseDownEvent, MouseUpEvent, Render,
    SharedString, TextRun, Window, div, font, prelude::*, px, svg,
};
use gpui_luma::controls::autocomplete::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplate, AutocompleteItemsTemplateHandlers,
    AutocompleteTextBoxRenderModel, AutocompleteTextBoxTemplate, AutocompleteTextBoxTemplateHandlers,
    default_autocomplete_items_template, default_autocomplete_textbox_template,
};
use gpui_luma::controls::combobox::{
    ComboBoxItemsTemplate, ComboBoxPanelTemplate, ComboBoxRenderModel, ComboBoxTemplate, ComboBoxTemplateHandlers,
    default_combobox_items_template, default_combobox_panel_template, default_combobox_template,
};
use gpui_luma::controls::search_selector::{
    SearchSelectorItemsTemplate, SearchSelectorPanelTemplate, SearchSelectorRenderModel, SearchSelectorTemplate,
    SearchSelectorTemplateHandlers, default_search_selector_items_template, default_search_selector_panel_template,
    default_search_selector_template,
};
use gpui_luma::controls::selector::{
    ControlFocusState, SelectorItem, SelectorPath, SelectorPlacement, SelectorRenderModel, SelectorTemplate,
    SelectorTemplateHandlers,
};
use gpui_luma::controls::selector_panel::{
    SelectorItem as SelectorPanelItem, SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, default_selector_items_template,
};
use gpui_luma::controls::textfield::{
    TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTemplateHandlers, TextFieldTheme,
    TextFieldVariant,
};
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale};
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::{
    gallery_pane_with_description, notify_entity, render_combobox_popup_preview_from_templates,
    render_search_selector_popup_preview_from_templates,
};

const SELECTOR_TEMPLATES_DESCRIPTION: &str = concat!(
    "Template matrix for selection controls. ",
    "Rows are interaction states; columns are selection controls."
);

#[derive(Clone)]
pub(in crate::gallery) struct SelectorControlsTemplatePane {
    state_preview: gpui::Entity<SelectorControlsTemplatePreview>,
}

impl SelectorControlsTemplatePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self { state_preview: cx.new(|_| SelectorControlsTemplatePreview::new(look)) }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
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
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.state_preview, cx);
    }
}

#[derive(Clone)]
struct SelectorControlsTemplatePreview {
    look: Arc<ShadcnLook>,
    textfield_template: Arc<dyn TextFieldTemplate>,
    textfield_theme: Arc<dyn TextFieldTheme>,
    autocomplete_template: Arc<dyn AutocompleteTextBoxTemplate>,
    autocomplete_items_template: Arc<dyn AutocompleteItemsTemplate>,
    combobox_template: Arc<dyn ComboBoxTemplate>,
    combobox_items_template: Arc<dyn ComboBoxItemsTemplate>,
    combobox_panel_template: Arc<dyn ComboBoxPanelTemplate>,
    selector_template: Arc<dyn SelectorTemplate>,
    selector_items_template: Arc<dyn SelectorItemsTemplate<SelectorItem>>,
    search_selector_template: Arc<dyn SearchSelectorTemplate>,
    search_selector_items_template: Arc<dyn SearchSelectorItemsTemplate>,
    search_selector_panel_template: Arc<dyn SearchSelectorPanelTemplate>,
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
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self {
            textfield_template: look.textfield_template(),
            textfield_theme: look.textfield_theme(),
            autocomplete_template: default_autocomplete_textbox_template(),
            autocomplete_items_template: default_autocomplete_items_template(),
            combobox_template: default_combobox_template(),
            combobox_items_template: default_combobox_items_template(),
            combobox_panel_template: default_combobox_panel_template(),
            selector_template: look.selector_template(),
            selector_items_template: default_selector_items_template(),
            search_selector_template: default_search_selector_template(),
            search_selector_items_template: default_search_selector_items_template(),
            search_selector_panel_template: default_search_selector_panel_template(),
            look,
        }
    }
}

impl Render for SelectorControlsTemplatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
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
            render_autocomplete_trigger(preview, &id, "Type to filter…", state, window, cx)
        }
        SelectorTemplateControl::ComboBox => {
            render_combobox_trigger(preview, &id, "Strict mode (exact match only)…", state, window, cx)
        }
        SelectorTemplateControl::SearchSelector => {
            render_search_selector_trigger(preview, &id, "Choose a state…", state, window, cx)
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

fn render_autocomplete_trigger(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let value = SharedString::from("California");
    let placeholder = SharedString::from(placeholder);
    let status_theme = preview.look.autocomplete_textbox_theme().resolve();
    let popup_appearance = preview.look.selector_items_panel_appearance(ControlSize::Md);

    let model = AutocompleteTextBoxRenderModel {
        textfield: render_preview_textfield(preview, id, &placeholder, &value, state, window, cx),
        query_is_empty: false,
        popup_width: px(168.0),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: status_theme.status_color,
        muted_text_color: status_theme.muted_text_color,
        popup_bounds: None,
        popup_appearance,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(
            preview
                .autocomplete_template
                .render(model, AutocompleteTextBoxTemplateHandlers::default(), window, cx),
        )
        .into_any_element()
}

fn render_combobox_trigger(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let value = SharedString::from("California");
    let placeholder = SharedString::from(placeholder);
    let status_theme = preview.look.autocomplete_textbox_theme().resolve();
    let popup_appearance = preview.look.selector_items_panel_appearance(ControlSize::Md);

    let popup_bounds = (state.id == "pressed")
        .then(|| gpui::Bounds::new(gpui::point(px(0.0), px(0.0)), gpui::size(px(168.0), px(32.0))));

    let model = ComboBoxRenderModel {
        textfield: render_preview_textfield(preview, id, &placeholder, &value, state, window, cx),
        query_is_empty: false,
        show_down_arrow: true,
        show_clear_button: true,
        full_width: true,
        minimum_trigger_width: px(168.0),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: status_theme.status_color,
        muted_text_color: status_theme.muted_text_color,
        popup_bounds,
        popup_appearance,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(preview.combobox_template.render(model, ComboBoxTemplateHandlers::default(), window, cx))
        .into_any_element()
}

fn render_preview_textfield(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    placeholder: &SharedString,
    value: &SharedString,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let character_offsets = textfield_character_offsets(
        value.as_ref(),
        preview.textfield_theme.clone(),
        TextFieldVariant::Standard,
        state.textfield_state,
        state.textfield_enabled,
        window,
    );

    let appearance = preview_textfield_appearance(
        &preview.textfield_theme,
        TextFieldVariant::Standard,
        state.textfield_state,
        state.textfield_enabled,
        window,
    );
    let text_model = TextFieldRenderModel {
        id,
        placeholder,
        value,
        prefix_icon: None,
        variant: TextFieldVariant::Standard,
        enabled: state.textfield_enabled,
        full_width: true,
        state: state.textfield_state,
        caret_visible: false,
        horizontal_scroll: 0.0,
        character_offsets,
        appearance,
    };

    preview
        .textfield_template
        .render(&text_model, textfield_preview_handlers(), window, cx)
        .into_any_element()
}

fn render_search_selector_trigger(
    preview: &SelectorControlsTemplatePreview,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let model = SearchSelectorRenderModel {
        id: id.clone(),
        trigger_label: SharedString::from(placeholder),
        trigger_label_is_placeholder: true,
        trigger_state: state.textfield_state,
        trigger_theme: preview.textfield_theme.clone(),
        enabled: state.textfield_enabled,
        full_width: true,
        minimum_trigger_width: px(168.0),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: preview.look.chrome().muted_text,
        muted_text_color: preview.look.chrome().muted_text,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(
            preview
                .search_selector_template
                .render(model, SearchSelectorTemplateHandlers::default(), window, cx),
        )
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
        selected_index: None,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: SelectorPlacement::BelowStart,
        active_path: None,
        enabled: state.selector_enabled,
        item_template: None,
        panel_template: None,
        focus: state.selector_focus,
        state: state.selector_state,
    };

    preview
        .selector_template
        .render(&model, SelectorTemplateHandlers::default(), window, cx)
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
    let appearance = preview.look.selector_items_panel_appearance(ControlSize::Md);
    let popup_id = SharedString::from(format!("{id}-popup-preview"));
    let items = popup_items_for_control(control);
    let item_hovers = (0..items.len()).map(|_| Box::new(noop_hover) as SelectorPanelHoverHandler).collect::<Vec<_>>();
    let item_clicks = (0..items.len()).map(|_| Box::new(noop_click) as SelectorPanelClickHandler).collect::<Vec<_>>();

    let rows: AnyElement = match control {
        SelectorTemplateControl::AutocompleteTextBox => preview
            .autocomplete_items_template
            .render(
                &AutocompleteItemsRenderModel {
                    id: &popup_id,
                    items: &items,
                    appearance: appearance.clone(),
                    highlighted_index: Some(0),
                },
                AutocompleteItemsTemplateHandlers { item_hovers, item_clicks },
            )
            .into_any_element(),
        SelectorTemplateControl::ComboBox => render_combobox_popup_preview_from_templates(
            &popup_id,
            &items,
            appearance.clone(),
            &preview.combobox_items_template,
            &preview.combobox_panel_template,
            item_hovers,
            item_clicks,
            cx,
        ),
        SelectorTemplateControl::Selector => preview
            .selector_items_template
            .render(
                &SelectorItemsRenderModel {
                    menu_id: &popup_id,
                    selector_id: id,
                    items: &items,
                    selected_index: None,
                    active_path: Some(SelectorPath::Item(0)),
                    open: true,
                    enabled: true,
                    focus: ControlFocusState { focused: true, focus_visible: true },
                    item_template: None,
                    appearance: appearance.clone(),
                },
                SelectorItemsTemplateHandlers { item_hovers, item_clicks },
                cx,
            )
            .into_any_element(),
        SelectorTemplateControl::SearchSelector => {
            let search_id = SharedString::from(format!("{id}-popup-search-preview"));
            let search_placeholder = SharedString::from("Selection search");
            let search_value = SharedString::from("");
            let search_offsets = textfield_character_offsets(
                search_value.as_ref(),
                preview.textfield_theme.clone(),
                TextFieldVariant::Standard,
                TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
                true,
                window,
            );

            let search_state = TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() };
            let search_appearance = preview_textfield_appearance(
                &preview.textfield_theme,
                TextFieldVariant::Standard,
                search_state,
                true,
                window,
            );
            let search_model = TextFieldRenderModel {
                id: &search_id,
                placeholder: &search_placeholder,
                value: &search_value,
                prefix_icon: None,
                variant: TextFieldVariant::Standard,
                enabled: true,
                full_width: true,
                state: search_state,
                caret_visible: false,
                horizontal_scroll: 0.0,
                character_offsets: search_offsets,
                appearance: search_appearance,
            };

            let search_content = preview
                .textfield_template
                .render(&search_model, textfield_preview_handlers(), window, cx)
                .into_any_element();

            render_search_selector_popup_preview_from_templates(
                &popup_id,
                &items,
                appearance.clone(),
                &preview.search_selector_items_template,
                &preview.search_selector_panel_template,
                search_content,
                item_hovers,
                item_clicks,
                cx,
            )
        }
    };

    div().w(px(168.0)).child(rows).into_any_element()
}

fn popup_items_for_control(control: SelectorTemplateControl) -> Vec<SelectorPanelItem> {
    match control {
        SelectorTemplateControl::AutocompleteTextBox => vec![
            SelectorPanelItem::new("autocomplete-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("autocomplete-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("autocomplete-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::ComboBox => vec![
            SelectorPanelItem::new("combobox-preview-item-1").label("California"),
            SelectorPanelItem::new("combobox-preview-item-2").label("Colorado"),
            SelectorPanelItem::new("combobox-preview-item-3").label("Connecticut"),
        ],
        SelectorTemplateControl::Selector => vec![
            SelectorPanelItem::new("selector-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("selector-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("selector-preview-item-3").label("Arizona"),
        ],
        SelectorTemplateControl::SearchSelector => vec![
            SelectorPanelItem::new("search-selector-preview-item-1").label("Alabama"),
            SelectorPanelItem::new("search-selector-preview-item-2").label("Alaska"),
            SelectorPanelItem::new("search-selector-preview-item-3").label("Arizona"),
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
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
) -> Vec<f32> {
    let appearance = preview_textfield_appearance(&theme, variant, state, enabled, window);
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
        hover: Box::new(noop_hover),
        mouse_down: Box::new(noop_mouse_down),
        mouse_move: Box::new(noop_mouse_move),
        mouse_up: Box::new(noop_mouse_up),
        mouse_up_out: Box::new(noop_mouse_up),
        click: Box::new(noop_click),
        key_down: Box::new(noop_key_down),
    }
}

fn preview_textfield_appearance(
    theme: &Arc<dyn TextFieldTheme>,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    window: &Window,
) -> gpui_luma::controls::textfield::TextFieldAppearance {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_appearance(variant, state, enabled, &scale)
}

fn noop_mouse_move(_: &gpui::MouseMoveEvent, _: &mut Window, _: &mut App) {}
fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
