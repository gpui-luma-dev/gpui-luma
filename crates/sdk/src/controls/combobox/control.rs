use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, IntoElement, KeyDownEvent, MouseDownEvent, Pixels, Render,
    ScrollWheelEvent, SharedString, Subscription, TextRun, Window, font, px,
};

use crate::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler};
use crate::controls::menu_item::MenuItem;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::theme::{AutocompleteTextBoxTheme, DefaultAutocompleteTextBoxTheme, DefaultFloatingMenuTheme, FloatingMenuTheme};

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::model::ComboBoxBuilder;
use crate::controls::popup_scroll_surface::PopupScrollSurface;
use super::template::{ComboBoxRenderModel, ComboBoxTemplateHandlers, render_popup_rows};
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
        if hovered && self.behavior.state.open {
            self.behavior.state.highlighted_filtered = Some(index);
            self.sync_popup_highlight_visibility(cx);
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() || !self.behavior.state.open {
            return;
        }

        if let Some(item_index) = self.behavior.state.filtered.get(index).copied() {
            self.select_item(item_index, false, cx);
        }
    }

    fn popup_page_size(&self) -> usize {
        self.popup_surface.visible_row_count().min(14).max(1)
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
                if self.behavior.state.open && !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MoveNext, &self.model.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "up" | "arrowup" => {
                if self.behavior.state.open && !self.behavior.state.filtered.is_empty() {
                    self.behavior.apply(SelectionEvent::MovePrevious, &self.model.items);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "pagedown" | "page_down" | "pgdown" => {
                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.state.open = true;
                    self.move_highlight_page(true, self.popup_page_size());
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "pageup" | "page_up" | "pgup" => {
                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.state.open = true;
                    self.move_highlight_page(false, self.popup_page_size());
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "home" => {
                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.state.open = true;
                    self.behavior.state.highlighted_filtered = Some(0);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            "end" => {
                if !self.behavior.state.filtered.is_empty() {
                    self.behavior.state.open = true;
                    self.behavior.state.highlighted_filtered = Some(self.behavior.state.filtered.len() - 1);
                    self.sync_popup_highlight_visibility(cx);
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_clear_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.clear(cx);
        cx.stop_propagation();
    }

    fn open_popup_with_all_items(&mut self, cx: &mut Context<Self>) {
        if self.model.items.is_empty() {
            self.behavior.state.filtered.clear();
            self.behavior.state.highlighted_filtered = None;
            self.behavior.state.open = false;
            return;
        }

        self.behavior.state.filtered = (0..self.model.items.len()).collect();
        self.behavior.state.highlighted_filtered = Some(0);
        self.behavior.state.open = true;
        self.behavior.state.status = SelectionStatus::Searching;
        self.sync_popup_highlight_visibility(cx);
        cx.notify();
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
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
        self.trigger_bounds = Some(bounds.clone());
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
        let appearance = DefaultFloatingMenuTheme::new(tokens).resolve();
        let selected_label = self
            .behavior
            .state
            .selected_item
            .map(|index| self.model.items[index].label.to_string())
            .unwrap_or_else(|| "none".to_string());

        let minimum_trigger_width = {
            let textfield_appearance = crate::theme::default_textfield_theme()
                .resolve(crate::controls::textfield::TextFieldState::default(), true);
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
            let prefix_width = self.model.show_down_arrow.then_some(18.0 + 10.0).unwrap_or(0.0);
            let clear_width = 18.0 + 10.0;
            let spacing = 8.0;

            px(textfield_side_padding + placeholder_width + prefix_width + clear_width + spacing)
        };

        let menu_items = self
            .behavior
            .state
            .filtered
            .iter()
            .enumerate()
            .map(|(visible_index, item_index)| {
                let item = &self.model.items[*item_index];
                MenuItem::new(format!("combobox-item-{}-{visible_index}", item.id)).label(item.label.clone())
            })
            .collect::<Vec<_>>();

        let item_hovers = (0..menu_items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as FloatingMenuHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..menu_items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_item_click(index, event, window, cx);
                })) as FloatingMenuClickHandler
            })
            .collect::<Vec<_>>();

        let status_label = SharedString::from(format!("status: {}", self.behavior.state.status.label()));
        let status_detail = SharedString::from(format!(
            "selected: {selected_label} | matches: {} | last: {} | keyboard: {}",
            self.behavior.state.filtered.len(),
            self.last_event,
            self.last_keyboard_event
        ));

        let popup_content = if self.behavior.state.open && !menu_items.is_empty() {
            let menu_id = SharedString::from("combobox-menu");
            let row_height = px(appearance.item_height);
            let content_top_padding = px(appearance.padding);
            let max_visible_rows = 7.0;
            let viewport_height = px((appearance.padding * 2.0) + (appearance.item_height * max_visible_rows));

            self.popup_surface.configure(menu_items.len(), row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let menu_content = render_popup_rows(
                &menu_id,
                &menu_items,
                appearance.clone(),
                self.behavior.state.highlighted_filtered,
                item_hovers,
                item_clicks,
            );

            Some(self.popup_surface.render(menu_content.into_any_element()))
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
            textfield: self.textfield.clone(),
            query_is_empty: self.behavior.state.query.is_empty(),
            show_down_arrow: self.model.show_down_arrow,
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
