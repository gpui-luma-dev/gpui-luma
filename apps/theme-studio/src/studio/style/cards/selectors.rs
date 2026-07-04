use super::super::*;
use super::inputs::{input_noop_click, input_noop_hover, input_textfield_handlers};

pub(in crate::studio::style::style_guide) fn render_selector_templates_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let states = selector_template_state_samples();
    let controls = [
        SelectorTemplateControl::AutocompleteTextBox,
        SelectorTemplateControl::ComboBox,
        SelectorTemplateControl::Selector,
        SelectorTemplateControl::SearchSelector,
    ];

    section_shell_with_width(
        960.0,
        "Selectors",
        "Interaction states across selector triggers.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(10.0))
            .child(render_selector_header_row(&controls, chrome.muted_text))
            .children(
                states
                    .iter()
                    .map(|state| render_selector_state_row(&look, state, &controls, chrome.muted_text, window, cx)),
            )
            .into_any_element(),
    )
}

fn render_selector_header_row(controls: &[SelectorTemplateControl], label_color: gpui::Hsla) -> AnyElement {
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

fn render_selector_state_row(
    look: &Arc<ShadcnLook>,
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
            div().flex().items_start().gap(px(10.0)).children(
                controls.iter().map(|control| render_selector_control_cell(look, *control, state, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_selector_control_cell(
    look: &Arc<ShadcnLook>,
    control: SelectorTemplateControl,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("theme-studio-selector-controls-template-{}-{}", state.id, control.header()));

    let trigger = match control {
        SelectorTemplateControl::AutocompleteTextBox => {
            render_selector_autocomplete_trigger(look, &id, "Type to filter...", state, window, cx)
        }
        SelectorTemplateControl::ComboBox => {
            render_selector_combobox_trigger(look, &id, "Strict mode (exact match only)...", state, window, cx)
        }
        SelectorTemplateControl::SearchSelector => {
            render_selector_search_selector_trigger(look, &id, "Choose a state...", state, window, cx)
        }
        SelectorTemplateControl::Selector => render_selector_selector_trigger(look, &id, state, window, cx),
    };

    let popup = if state.id == "pressed" {
        Some(render_selector_popup_preview(look, &id, control, window, cx))
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

fn render_selector_autocomplete_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let textfield_template = look.input_textfield_template();
    let textfield_theme = look.input_textfield_theme();
    let autocomplete_template = default_autocomplete_textbox_template();
    let value = SharedString::from("California");
    let placeholder = SharedString::from(placeholder);
    let status_theme = look.autocomplete_textbox_theme().resolve();
    let popup_look = look.selector_items_panel_look(ControlSize::Md);

    let model = AutocompleteTextBoxRenderModel {
        textfield: render_selector_preview_textfield(
            &textfield_template,
            &textfield_theme,
            id,
            &placeholder,
            &value,
            state,
            window,
            cx,
        ),
        query_is_empty: false,
        popup_width: px(168.0),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: status_theme.status_color,
        muted_text_color: status_theme.muted_text_color,
        popup_bounds: None,
        popup_look,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(autocomplete_template.render(model, AutocompleteTextBoxTemplateHandlers::default(), window, cx))
        .into_any_element()
}

fn render_selector_combobox_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let textfield_template = look.input_textfield_template();
    let textfield_theme = look.input_textfield_theme();
    let combobox_template = default_combobox_template();
    let value = SharedString::from("California");
    let placeholder = SharedString::from(placeholder);
    let status_theme = look.autocomplete_textbox_theme().resolve();
    let popup_look = look.selector_items_panel_look(ControlSize::Md);
    let popup_bounds = (state.id == "pressed")
        .then(|| gpui::Bounds::new(gpui::point(px(0.0), px(0.0)), gpui::size(px(168.0), px(32.0))));

    let model = ComboBoxRenderModel {
        textfield: render_selector_preview_textfield(
            &textfield_template,
            &textfield_theme,
            id,
            &placeholder,
            &value,
            state,
            window,
            cx,
        ),
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
        popup_look,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(combobox_template.render(model, ComboBoxTemplateHandlers::default(), window, cx))
        .into_any_element()
}

fn render_selector_search_selector_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    placeholder: &'static str,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let search_selector_template = default_search_selector_template();
    let model = SearchSelectorRenderModel {
        id: id.clone(),
        trigger_label: SharedString::from(placeholder),
        trigger_label_is_placeholder: true,
        trigger_state: state.textfield_state,
        trigger_theme: look.input_textfield_theme(),
        enabled: state.textfield_enabled,
        full_width: true,
        minimum_trigger_width: px(168.0),
        status_label: SharedString::from(""),
        status_detail: SharedString::from(""),
        status_color: look.chrome().muted_text,
        muted_text_color: look.chrome().muted_text,
        popup_content: None,
    };

    div()
        .w(px(168.0))
        .child(search_selector_template.render(model, SearchSelectorTemplateHandlers::default(), window, cx))
        .into_any_element()
}

fn render_selector_selector_trigger(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selector_template = look.selector_template();
    let label = SharedString::from("Select status");
    let items = selector_trigger_items().into_iter().collect::<Vec<_>>();
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

    selector_template
        .render(&model, SelectorTemplateHandlers::default(), window, cx)
        .w(px(168.0))
        .into_any_element()
}

fn render_selector_preview_textfield(
    textfield_template: &Arc<dyn TextFieldTemplate>,
    textfield_theme: &Arc<dyn TextFieldTheme>,
    id: &SharedString,
    placeholder: &SharedString,
    value: &SharedString,
    state: &SelectorTemplateStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let character_offsets = selector_textfield_character_offsets(
        value.as_ref(),
        textfield_theme.clone(),
        TextFieldVariant::Standard,
        state.textfield_state,
        state.textfield_enabled,
        window,
    );

    let look = selector_preview_textfield_look(
        textfield_theme,
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
        look,
    };

    textfield_template.render(&text_model, input_textfield_handlers(), window, cx).into_any_element()
}

fn render_selector_popup_preview(
    look: &Arc<ShadcnLook>,
    id: &SharedString,
    control: SelectorTemplateControl,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let popup_look = look.selector_items_panel_look(ControlSize::Md);
    let popup_id = SharedString::from(format!("{id}-popup-preview"));
    let items = selector_popup_items_for_control(control);
    let item_hovers = (0..items.len())
        .map(|_| Box::new(input_noop_hover) as SelectorPanelHoverHandler)
        .collect::<Vec<_>>();
    let item_clicks = (0..items.len())
        .map(|_| Box::new(input_noop_click) as SelectorPanelClickHandler)
        .collect::<Vec<_>>();

    let rows: AnyElement = match control {
        SelectorTemplateControl::AutocompleteTextBox => default_autocomplete_items_template()
            .render(
                &AutocompleteItemsRenderModel {
                    id: &popup_id,
                    items: &items,
                    look: popup_look.clone(),
                    highlighted_index: Some(0),
                },
                AutocompleteItemsTemplateHandlers { item_hovers, item_clicks },
            )
            .into_any_element(),
        SelectorTemplateControl::ComboBox => render_selector_combobox_popup_preview_from_templates(
            &popup_id,
            &items,
            popup_look.clone(),
            &default_combobox_items_template(),
            &default_combobox_panel_template(),
            item_hovers,
            item_clicks,
            cx,
        ),
        SelectorTemplateControl::Selector => default_selector_items_template()
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
                    look: popup_look.clone(),
                    max_height: px(240.0),
                },
                SelectorItemsTemplateHandlers { item_hovers, item_clicks },
                cx,
            )
            .into_any_element(),
        SelectorTemplateControl::SearchSelector => {
            let search_id = SharedString::from(format!("{id}-popup-search-preview"));
            let search_placeholder = SharedString::from("Selection search");
            let search_value = SharedString::from("");
            let search_offsets = selector_textfield_character_offsets(
                search_value.as_ref(),
                look.input_textfield_theme(),
                TextFieldVariant::Standard,
                TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
                true,
                window,
            );
            let search_state = TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() };
            let search_look = selector_preview_textfield_look(
                &look.input_textfield_theme(),
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
                look: search_look,
            };

            let search_content = look
                .input_textfield_template()
                .render(&search_model, input_textfield_handlers(), window, cx)
                .into_any_element();

            render_selector_search_selector_popup_preview_from_templates(
                &popup_id,
                &items,
                popup_look.clone(),
                &default_search_selector_items_template(),
                &default_search_selector_panel_template(),
                search_content,
                item_hovers,
                item_clicks,
                cx,
            )
        }
    };

    div().w(px(168.0)).child(rows).into_any_element()
}

fn selector_template_state_samples() -> [SelectorTemplateStateSample; 5] {
    [
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
    ]
}

fn selector_trigger_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

fn selector_popup_items_for_control(control: SelectorTemplateControl) -> Vec<SelectorPanelItem> {
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

fn selector_textfield_character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
) -> Vec<f32> {
    let look = selector_preview_textfield_look(&theme, variant, state, enabled, window);
    let value_shared = SharedString::from(value.to_string());
    let run = TextRun {
        len: value_shared.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = look.typography.weight;
            font
        },
        color: look.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(value_shared, px(look.typography.size), &[run], None);
    let chars = value.chars().count();

    let mut offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = selector_char_to_byte_offset(value, char_offset);
        offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    offsets
}

fn selector_char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    if char_offset == 0 {
        return 0;
    }

    text.char_indices().nth(char_offset).map_or(text.len(), |(byte, _)| byte)
}

fn selector_preview_textfield_look(
    theme: &Arc<dyn TextFieldTheme>,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
    window: &Window,
) -> gpui_luma::controls::textfield::TextFieldLook {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_look(variant, state, enabled, &scale)
}

fn render_selector_combobox_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    look: SelectorItemsPanelLook,
    items_template: &Arc<dyn ComboBoxItemsTemplate>,
    panel_template: &Arc<dyn ComboBoxPanelTemplate>,
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
    cx: &mut App,
) -> AnyElement {
    let combobox_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            ComboBoxSelectionItem::new(format!("combobox-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..combobox_items.len()).collect::<Vec<_>>();

    let list = items_template.render(
        &ComboBoxItemsRenderModel {
            menu_id: popup_id,
            combobox_id: popup_id,
            items: &combobox_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            look: look.clone(),
        },
        ComboBoxItemsTemplateHandlers { item_hovers, item_clicks },
        cx,
    );

    panel_template.render(
        ComboBoxPanelRenderModel {
            id: popup_id,
            items: &combobox_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_look: look,
            list_content: list.into_any_element(),
        },
        cx,
    )
}

fn render_selector_search_selector_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    look: SelectorItemsPanelLook,
    items_template: &Arc<dyn SearchSelectorItemsTemplate>,
    panel_template: &Arc<dyn SearchSelectorPanelTemplate>,
    search_content: AnyElement,
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
    cx: &mut App,
) -> AnyElement {
    let search_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            SearchSelectorSelectionItem::new(format!("search-selector-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..search_items.len()).collect::<Vec<_>>();

    let rows = items_template
        .render(
            &SearchSelectorItemsRenderModel {
                menu_id: popup_id,
                search_selector_id: popup_id,
                items: &search_items,
                visible_indices: &visible_indices,
                selected_source_index: None,
                active_visible_index: Some(0),
                open: true,
                enabled: true,
                item_template: None,
                look: look.clone(),
            },
            SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks },
            cx,
        )
        .into_any_element();

    panel_template.render(
        SearchSelectorPanelRenderModel {
            id: popup_id,
            items: &search_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_look: look,
            search_content: Some(search_content),
            list_content: rows,
        },
        cx,
    )
}
