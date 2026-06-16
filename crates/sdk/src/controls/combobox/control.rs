use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, IntoElement, KeyDownEvent, MouseDownEvent, Pixels, Render,
    ScrollWheelEvent, SharedString, Subscription, TextRun, Window, font, px,
};

use crate::controls::selector_panel::{SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::autocomplete::{AutocompleteTextBoxTheme, DefaultAutocompleteTextBoxTheme};
use crate::theme::observe_theme_revision;

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::item_template::ComboBoxItemTemplate;
use super::model::ComboBoxBuilder;
use super::panel_template::{ComboBoxPanelRenderModel, ComboBoxPanelTemplate};
use super::template::{ComboBoxItemsTemplate, ComboBoxTemplate};
use crate::controls::popup_scroll_surface::PopupScrollSurface;
use super::template::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplateHandlers, ComboBoxRenderModel, ComboBoxTemplateHandlers,
};
use super::text_selection::{self, TextSelectionEvent};

#[derive(Clone, Debug)]
pub enum ComboBoxEvent {
    Change,
    Select,
    Complete,
    Clear,
}

pub type ComboBox = Entity<ComboBoxControl>;

pub struct ComboBoxControl {
    textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    model: super::model::ComboBoxModel,
    behavior: SelectionBehavior,
    trigger_bounds: Option<Bounds<Pixels>>,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    committed_selection: Option<usize>,
    open_popup_on_next_focus: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ComboBoxEvent> for ComboBoxControl {}

impl ComboBoxControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
    ) -> ComboBoxBuilder {
        ComboBoxBuilder::new(id, items)
    }

    pub(crate) fn from_builder(builder: ComboBoxBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model;

        let textfield = text_selection::new(format!("{}-textfield", model.id))
            .placeholder(model.placeholder.clone())
            .enabled(model.enabled)
            .full_width(model.full_width)
            .clean_on_escape(model.clean_on_escape)
            .propagate_home_end_to_parent(true)
            .template(model.textfield_template.clone())
            .spawn(cx);

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-popup-surface", model.id), model.scrollbar_template.clone(), cx);
        popup_surface.set_scrolling_enabled(model.scrolling);

        let subscriptions = vec![
            cx.subscribe(&textfield, |this, _, event: &crate::controls::textfield::TextFieldEvent, cx| {
                this.handle_text_selection_event(text_selection::map_event(event), cx);
            }),
            cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| match event {
                ScrollbarEvent::Change { value } => {
                    this.popup_surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            }),
            observe_theme_revision(cx, |_, cx| cx.notify()),
        ];

        Self {
            textfield,
            popup_surface,
            model,
            behavior: SelectionBehavior::new(),
            trigger_bounds: None,
            last_event: SharedString::from("none"),
            last_keyboard_event: SharedString::from("none"),
            committed_selection: None,
            open_popup_on_next_focus: false,
            _subscriptions: subscriptions,
        }
    }

    fn handle_text_selection_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        match event {
            TextSelectionEvent::Change { value } => {
                self.behavior.set_query(value.clone(), &self.model.items);
                self.last_event = SharedString::from("change");
                cx.emit(ComboBoxEvent::Change);
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                self.behavior.apply(SelectionEvent::Focus, &self.model.items);
                if self.open_popup_on_next_focus {
                    self.open_popup_on_next_focus = false;
                    self.open_popup_with_all_items(cx);
                }
                cx.notify();
            }
            TextSelectionEvent::FocusLeave => {
                self.open_popup_on_next_focus = false;
                self.apply_blur_selection_policy(cx);
                self.behavior.apply(SelectionEvent::Blur, &self.model.items);
                cx.notify();
            }
        }
    }

    fn handle_submit_result(&mut self, result: SubmitResult, cx: &mut Context<Self>) {
        match result {
            SubmitResult::None => {
                self.last_event = SharedString::from("submit:no-match");
                cx.notify();
            }
            SubmitResult::Select { index, exact_complete } => {
                self.select_item(index, exact_complete, cx);
            }
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.committed_selection = None;
        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.last_event = SharedString::from("clear");

        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));

        cx.emit(ComboBoxEvent::Clear);
        cx.notify();
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.model.items[index];
        let label = item.label.clone();

        self.behavior.set_query(label.clone(), &self.model.items);
        self.behavior.select_index(index);
        self.committed_selection = Some(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));

        if exact_complete {
            cx.emit(ComboBoxEvent::Complete);
        } else {
            cx.emit(ComboBoxEvent::Select);
        }
        cx.notify();
    }

    fn apply_blur_selection_policy(&mut self, cx: &mut Context<Self>) {
        let query = self.behavior.state.query.to_string();
        let trimmed_query = query.trim();

        if trimmed_query.is_empty() {
            self.committed_selection = None;
            self.behavior.apply(SelectionEvent::Clear, &self.model.items);
            self.last_event = SharedString::from("blur:clear-empty");
            self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
            return;
        }

        if self.behavior.state.status == SelectionStatus::Complete {
            if let Some(index) = self.behavior.state.selected_item {
                self.committed_selection = Some(index);
            }
            self.last_event = SharedString::from("blur:commit");
            return;
        }

        if let Some(index) = self.committed_selection.filter(|index| *index < self.model.items.len()) {
            let item = &self.model.items[index];
            let label = item.label.clone();
            self.behavior.set_query(label.clone(), &self.model.items);
            self.behavior.select_index(index);
            self.last_event = SharedString::from(format!("blur:revert:{}", item.label));
            self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));
            return;
        }

        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.last_event = SharedString::from("blur:clear-invalid");
        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let enabled_item = self
            .behavior
            .state
            .filtered
            .get(index)
            .and_then(|item_index| self.model.items.get(*item_index))
            .is_some_and(|item| item.enabled);

        if hovered && enabled_item && self.behavior.state.open {
            self.behavior.state.highlighted_filtered = Some(index);
            self.sync_popup_highlight_visibility(cx);
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() || !self.behavior.state.open {
            return;
        }

        if let Some(item_index) = self.behavior.state.filtered.get(index).copied()
            && self.model.items.get(item_index).is_some_and(|item| item.enabled)
        {
            self.select_item(item_index, false, cx);
        }
    }

    fn popup_page_size(&self) -> usize {
        self.popup_surface.visible_row_count().clamp(1, 14)
    }

    fn move_highlight_page(&mut self, forward: bool, page_size: usize) {
        if self.behavior.state.filtered.is_empty() {
            return;
        }

        let len = self.behavior.state.filtered.len();
        let step = page_size.max(1);
        let current = self.behavior.state.highlighted_filtered.unwrap_or(if forward { 0 } else { len - 1 });
        let next = if forward {
            (current + step).min(len - 1)
        } else {
            current.saturating_sub(step)
        };

        self.behavior.state.highlighted_filtered = Some(next);
        self.behavior.state.open = true;
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.last_keyboard_event = SharedString::from(format!("key:{}", event.keystroke.key));

        if event.keystroke.modifiers.control || event.keystroke.modifiers.secondary() {
            return;
        }

        match event.keystroke.key.as_str() {
            "escape" => {
                self.behavior.apply(SelectionEvent::Escape, &self.model.items);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "down" | "arrowdown" => {
                if !self.behavior.state.open {
                    self.open_popup_with_all_items(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    return;
                }

                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MoveNext, &self.model.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "up" | "arrowup" if self.behavior.state.open && !self.behavior.state.filtered.is_empty() => {
                self.behavior.apply(SelectionEvent::MovePrevious, &self.model.items);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "pagedown" | "page_down" | "pgdown" if !self.behavior.state.filtered.is_empty() => {
                self.behavior.state.open = true;
                self.move_highlight_page(true, self.popup_page_size());
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "pageup" | "page_up" | "pgup" if !self.behavior.state.filtered.is_empty() => {
                self.behavior.state.open = true;
                self.move_highlight_page(false, self.popup_page_size());
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "home" if !self.behavior.state.filtered.is_empty() => {
                self.behavior.state.open = true;
                self.behavior.state.highlighted_filtered = Some(0);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "end" if !self.behavior.state.filtered.is_empty() => {
                self.behavior.state.open = true;
                self.behavior.state.highlighted_filtered = Some(self.behavior.state.filtered.len() - 1);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            _ => {}
        }
    }

    fn handle_clear_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() {
            return;
        }

        self.clear(cx);
        cx.stop_propagation();
    }

    fn open_popup_with_all_items(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.model.items.is_empty() {
            self.behavior.state.filtered.clear();
            self.behavior.state.highlighted_filtered = None;
            self.behavior.state.open = false;
            return;
        }

        self.behavior.state.filtered = (0..self.model.items.len()).collect();
        self.behavior.state.highlighted_filtered =
            self.model.items.iter().enumerate().find_map(|(index, item)| item.enabled.then_some(index));
        self.behavior.state.open = true;
        self.behavior.state.status = SelectionStatus::Searching;
        self.sync_popup_highlight_visibility(cx);
        cx.notify();
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.is_keyboard() {
            return;
        }

        if self.behavior.state.open {
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.open_popup_on_next_focus = false;
            cx.stop_propagation();
            cx.notify();
            return;
        }

        if self.is_textfield_focused(cx) {
            self.open_popup_with_all_items(cx);
        } else {
            self.open_popup_on_next_focus = true;
        }

        cx.stop_propagation();
    }

    fn handle_trigger_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if event.click_count >= 2 {
            if self.is_textfield_focused(cx) {
                self.open_popup_with_all_items(cx);
            } else {
                self.open_popup_on_next_focus = true;
            }
        }
    }

    fn is_textfield_focused(&self, cx: &mut Context<Self>) -> bool {
        let mut focused = false;
        self.textfield.update(cx, |textfield, _| {
            focused = textfield.is_focused();
        });
        focused
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(*bounds);
    }

    fn handle_popup_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.behavior.state.open || self.behavior.state.filtered.is_empty() {
            return;
        }

        let moved = self.popup_surface.scroll_wheel(event, cx);
        if moved {
            cx.notify();
        }

        let delta_y = event.delta.pixel_delta(px(20.0)).y.as_f32();
        if delta_y.is_finite() && delta_y.abs() > f32::EPSILON {
            cx.stop_propagation();
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.textfield.update(cx, |textfield, cx| textfield.set_enabled(enabled, cx));
        if !enabled {
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.open_popup_on_next_focus = false;
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn ComboBoxTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_items_template(&mut self, template: std::sync::Arc<dyn ComboBoxItemsTemplate>, cx: &mut Context<Self>) {
        self.model.items_template = template;
        cx.notify();
    }

    pub fn set_panel_template(&mut self, template: std::sync::Arc<dyn ComboBoxPanelTemplate>, cx: &mut Context<Self>) {
        self.model.panel_template = template;
        cx.notify();
    }

    pub fn set_item_template(
        &mut self,
        template: Option<ComboBoxItemTemplate<super::behavior::SelectionItem>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_template = template;
        cx.notify();
    }

    fn sync_popup_highlight_visibility(&self, cx: &mut Context<Self>) {
        if !self.behavior.state.open {
            return;
        }

        if let Some(visible_index) = self.behavior.state.highlighted_filtered {
            self.popup_surface.ensure_item_visible(visible_index, cx);
        }
    }
}

impl Render for ComboBoxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = crate::theme::ThemeTokens::default();
        let autocomplete_appearance = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve();
        let appearance = (self.model.popup_appearance_provider)();
        let selected_label = self
            .behavior
            .state
            .selected_item
            .map(|index| self.model.items[index].label.to_string())
            .unwrap_or_else(|| "none".to_string());

        let minimum_trigger_width = {
            let textfield_theme = crate::controls::textfield::default_textfield_theme();
            let textfield_appearance = textfield_theme.resolve_appearance(
                crate::controls::textfield::TextFieldVariant::Standard,
                crate::controls::textfield::TextFieldState::default(),
                true,
                &crate::theme::StandardBoxScale::compute(
                    crate::theme::ControlSize::Md,
                    &textfield_theme.metrics(),
                    1.0,
                ),
            );
            let mut text_font = font(".SystemUIFont");
            text_font.weight = textfield_appearance.typography.weight;
            let placeholder_run = TextRun {
                len: self.model.placeholder.len(),
                font: text_font,
                color: autocomplete_appearance.muted_text_color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let placeholder_line = window.text_system().shape_line(
                self.model.placeholder.clone(),
                px(textfield_appearance.typography.size),
                &[placeholder_run],
                None,
            );
            let placeholder_width = placeholder_line.width().as_f32();

            let textfield_side_padding = (textfield_appearance.padding_x * 2.0) + textfield_appearance.border_width;
            let prefix_width = if self.model.show_down_arrow { 18.0 + 10.0 } else { 0.0 };
            let clear_width = if self.model.show_clear_button { 18.0 + 10.0 } else { 0.0 };
            let spacing = 8.0;

            px(textfield_side_padding + placeholder_width + prefix_width + clear_width + spacing)
        };

        let visible_indices = self.behavior.state.filtered.clone();

        let item_hovers = (0..visible_indices.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as SelectorPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..visible_indices.len())
            .map(|index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(index, event, window, cx);
                })) as SelectorPanelClickHandler
            })
            .collect::<Vec<_>>();

        let status_label = SharedString::from(format!("status: {}", self.behavior.state.status.label()));
        let status_detail = SharedString::from(format!(
            "selected: {selected_label} | matches: {} | last: {} | keyboard: {}",
            self.behavior.state.filtered.len(),
            self.last_event,
            self.last_keyboard_event
        ));

        let popup_content = if self.model.enabled && self.behavior.state.open && !visible_indices.is_empty() {
            let menu_id = SharedString::from("combobox-menu");
            let row_height = px(appearance.item_height);
            let content_top_padding = px(appearance.padding);
            let item_count = visible_indices.len();
            let min_visible_rows = self.model.min_visible_rows.max(1);
            let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
            let visible_rows = item_count.clamp(min_visible_rows, max_visible_rows) as f32;
            let viewport_height = px((appearance.padding * 2.0) + (appearance.item_height * visible_rows));
            let allow_scrolling = self.model.scrolling && item_count > max_visible_rows;

            self.popup_surface.set_scrolling_enabled(allow_scrolling);
            self.popup_surface.configure(item_count, row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let list_content = self.model.items_template.render(
                &ComboBoxItemsRenderModel {
                    menu_id: &menu_id,
                    combobox_id: &self.model.id,
                    items: &self.model.items,
                    visible_indices: &visible_indices,
                    selected_source_index: self.behavior.state.selected_item,
                    active_visible_index: self.behavior.state.highlighted_filtered,
                    open: self.behavior.state.open,
                    enabled: self.model.enabled,
                    item_template: self.model.item_template.as_ref(),
                    appearance: appearance.clone(),
                },
                ComboBoxItemsTemplateHandlers { item_hovers, item_clicks },
                cx,
            );

            let panel_content = self.model.panel_template.render(
                ComboBoxPanelRenderModel {
                    id: &self.model.id,
                    items: &self.model.items,
                    visible_indices: &visible_indices,
                    selected_source_index: self.behavior.state.selected_item,
                    active_visible_index: self.behavior.state.highlighted_filtered,
                    open: self.behavior.state.open,
                    enabled: self.model.enabled,
                    item_template: self.model.item_template.as_ref(),
                    popup_bounds: self.trigger_bounds,
                    popup_appearance: appearance.clone(),
                    list_content: self.popup_surface.render(list_content.into_any_element()),
                },
                cx,
            );

            Some(panel_content)
        } else {
            None
        };

        let handlers = ComboBoxTemplateHandlers {
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            scroll_wheel: Box::new(cx.listener(Self::handle_popup_scroll_wheel)),
            clear_click: Box::new(cx.listener(Self::handle_clear_click)),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_trigger_mouse_down)),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
        };

        let render_model = ComboBoxRenderModel {
            textfield: self.textfield.clone().into_any_element(),
            query_is_empty: self.behavior.state.query.is_empty(),
            show_down_arrow: self.model.show_down_arrow,
            show_clear_button: self.model.show_clear_button,
            full_width: self.model.full_width,
            minimum_trigger_width,
            status_label,
            status_detail,
            status_color: autocomplete_appearance.status_color,
            muted_text_color: autocomplete_appearance.muted_text_color,
            popup_bounds: self.trigger_bounds,
            popup_appearance: appearance,
            popup_content,
        };

        self.model.template.render(render_model, handlers, window, cx)
    }
}
