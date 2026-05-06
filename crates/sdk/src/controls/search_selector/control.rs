use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent,
    MouseDownEvent, Pixels, Render, ScrollWheelEvent, SharedString, Subscription, TextRun, Window, div, font,
    prelude::*, px,
};

use crate::controls::autocomplete::{AutocompleteTextBoxTheme, DefaultAutocompleteTextBoxTheme};
use crate::controls::floating_menu::{
    DefaultFloatingMenuTheme, FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuTheme,
};
use crate::controls::interaction::ControlInteraction;
use crate::controls::menu_item::MenuItem;
use crate::controls::popup_scroll_surface::PopupScrollSurface;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::textfield::TextFieldState;

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::model::SearchSelectorBuilder;
use super::template::{SearchSelectorRenderModel, SearchSelectorTemplateHandlers, render_popup_rows};
use super::text_selection::{self, TextSelectionEvent};

#[derive(Clone, Debug)]
pub enum SearchSelectorEvent {
    Change,
    Select,
    Complete,
    Clear,
}

pub type SearchSelector = Entity<SearchSelectorControl>;

pub struct SearchSelectorControl {
    popup_search_textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    model: super::model::SearchSelectorModel,
    behavior: SelectionBehavior,
    interaction: ControlInteraction,
    trigger_focused: bool,
    trigger_bounds: Option<Bounds<Pixels>>,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    committed_selection: Option<usize>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SearchSelectorEvent> for SearchSelectorControl {}

impl SearchSelectorControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
    ) -> SearchSelectorBuilder {
        SearchSelectorBuilder::new(id, items)
    }

    pub(crate) fn from_builder(builder: SearchSelectorBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model;

        let popup_search_textfield = text_selection::new(format!("{}-popup-search", model.id))
            .placeholder(model.search_placeholder.clone())
            .full_width(true)
            .clean_on_escape(model.clean_on_escape)
            .propagate_home_end_to_parent(true)
            .template(model.textfield_template.clone())
            .spawn(cx);

        let mut popup_surface =
            PopupScrollSurface::new(format!("{}-popup-surface", model.id), model.scrollbar_template.clone(), cx);
        popup_surface.set_scrolling_enabled(model.scrolling);

        let subscriptions = vec![
            cx.subscribe(&popup_search_textfield, |this, _, event: &crate::controls::textfield::TextFieldEvent, cx| {
                this.handle_popup_search_event(text_selection::map_event(event), cx);
            }),
            cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| match event {
                ScrollbarEvent::Change { value } => {
                    this.popup_surface.set_vertical_offset(*value, cx);
                    cx.notify();
                }
            }),
        ];

        Self {
            popup_search_textfield,
            popup_surface,
            model,
            behavior: SelectionBehavior::new(),
            interaction: ControlInteraction::new(true, cx),
            trigger_focused: false,
            trigger_bounds: None,
            last_event: SharedString::from("none"),
            last_keyboard_event: SharedString::from("none"),
            committed_selection: None,
            _subscriptions: subscriptions,
        }
    }

    fn handle_popup_search_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        match event {
            TextSelectionEvent::Change { value } => {
                self.behavior.set_query(value.clone(), &self.model.items);
                self.behavior.state.open = true;
                self.last_event = SharedString::from("search:change");

                if self.behavior.state.filtered.is_empty() {
                    self.clear_committed_selection(cx);
                }

                cx.emit(SearchSelectorEvent::Change);
                self.sync_popup_highlight_visibility(cx);
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("search:submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                self.behavior.apply(SelectionEvent::Focus, &self.model.items);
                if self.behavior.state.query.trim().is_empty() {
                    self.open_popup_with_all_items(cx);
                } else {
                    self.behavior.state.open = true;
                    self.sync_popup_highlight_visibility(cx);
                    cx.notify();
                }
            }
            TextSelectionEvent::FocusLeave => {
                self.behavior.apply(SelectionEvent::Blur, &self.model.items);
                cx.notify();
            }
        }
    }

    fn handle_submit_result(&mut self, result: SubmitResult, cx: &mut Context<Self>) {
        match result {
            SubmitResult::None => {
                self.last_event = SharedString::from("submit:no-match");
                self.clear_committed_selection(cx);
                cx.notify();
            }
            SubmitResult::Select { index, exact_complete } => {
                self.select_item(index, exact_complete, cx);
            }
        }
    }

    fn clear_committed_selection(&mut self, cx: &mut Context<Self>) {
        self.committed_selection = None;
        cx.emit(SearchSelectorEvent::Clear);
    }

    fn clear_popup_search(&mut self, cx: &mut Context<Self>) {
        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.model.items[index];

        self.behavior.select_index(index);
        self.committed_selection = Some(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.clear_popup_search(cx);

        if exact_complete {
            cx.emit(SearchSelectorEvent::Complete);
        } else {
            cx.emit(SearchSelectorEvent::Select);
        }
        cx.notify();
    }

    fn close_popup(&mut self, cx: &mut Context<Self>) {
        self.behavior.state.open = false;
        self.behavior.state.highlighted_filtered = None;
        self.clear_popup_search(cx);
        cx.notify();
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
        self.last_keyboard_event = SharedString::from(format!("key:{}", event.keystroke.key));

        if event.keystroke.modifiers.control || event.keystroke.modifiers.secondary() {
            return;
        }

        match event.keystroke.key.as_str() {
            "escape" => {
                self.behavior.apply(SelectionEvent::Escape, &self.model.items);
                self.close_popup(cx);
                window.prevent_default();
                cx.stop_propagation();
            }
            "down" | "arrowdown" => {
                if !self.behavior.state.open {
                    self.open_popup_with_all_items(cx);
                    self.focus_popup_search(window, cx);
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
            "enter" => {
                if self.behavior.state.open {
                    let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                    self.handle_submit_result(result, cx);
                    window.prevent_default();
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn open_popup_with_all_items(&mut self, cx: &mut Context<Self>) {
        if self.model.items.is_empty() {
            self.behavior.state.filtered.clear();
            self.behavior.state.highlighted_filtered = None;
            self.behavior.state.open = false;
            return;
        }

        self.popup_search_textfield.update(cx, |textfield, cx| textfield.set_value("", cx));
        self.behavior.state.query = SharedString::default();
        self.behavior.state.filtered = (0..self.model.items.len()).collect();
        self.behavior.state.highlighted_filtered = Some(0);
        self.behavior.state.open = true;
        self.behavior.state.status = SelectionStatus::Searching;
        self.sync_popup_highlight_visibility(cx);
        cx.notify();
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.behavior.state.open {
            self.close_popup(cx);
            cx.stop_propagation();
            return;
        }

        self.open_popup_with_all_items(cx);
        self.focus_popup_search(window, cx);
        cx.stop_propagation();
    }

    fn handle_trigger_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_trigger_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_down(true, window, cx) {
            cx.notify();
        }
    }

    fn handle_trigger_mouse_up(&mut self, _event: &gpui::MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(*bounds);
    }

    fn handle_popup_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.behavior.state.open {
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

    fn sync_trigger_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focused = self.interaction.focus_handle().is_focused(window);
        if focused == self.trigger_focused {
            return;
        }

        self.trigger_focused = focused;

        if focused {
            self.behavior.apply(SelectionEvent::Focus, &self.model.items);
            self.open_popup_with_all_items(cx);
            self.focus_popup_search(window, cx);
        } else {
            self.behavior.apply(SelectionEvent::Blur, &self.model.items);
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.clear_popup_search(cx);
            cx.notify();
        }
    }

    fn focus_popup_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        let focus_handle = self.popup_search_textfield.read(cx).focus_handle(cx);
        focus_handle.focus(window, cx);
    }
}

impl Focusable for SearchSelectorControl {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for SearchSelectorControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_trigger_focus(window, cx);

        let tokens = crate::theme::ThemeTokens::default();
        let autocomplete_appearance = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve();
        let appearance = DefaultFloatingMenuTheme::new(tokens).resolve();
        let selected_label = self
            .committed_selection
            .and_then(|index| self.model.items.get(index))
            .map(|item| item.label.to_string());

        let minimum_trigger_width = {
            let textfield_appearance = crate::controls::textfield::default_textfield_theme()
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
            let search_icon_width = 18.0 + 10.0;
            let spacing = 8.0;

            px(textfield_side_padding + placeholder_width + search_icon_width + spacing)
        };

        let menu_items = self
            .behavior
            .state
            .filtered
            .iter()
            .enumerate()
            .map(|(visible_index, item_index)| {
                let item = &self.model.items[*item_index];
                MenuItem::new(format!("search-selector-item-{}-{visible_index}", item.id)).label(item.label.clone())
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
            "selected: {} | matches: {} | last: {} | keyboard: {}",
            selected_label.clone().unwrap_or_else(|| "none".to_string()),
            self.behavior.state.filtered.len(),
            self.last_event,
            self.last_keyboard_event
        ));

        let (popup_content, popup_search_field, popup_list_panel) = if self.behavior.state.open {
            let list_id = SharedString::from("search-selector-menu");
            let row_height = px(appearance.item_height);
            let content_top_padding = px(appearance.padding);
            let item_count = menu_items.len();
            let min_visible_rows = self.model.min_visible_rows.max(1);
            let max_visible_rows = self.model.max_visible_rows.max(min_visible_rows);
            let visible_rows = item_count.max(1).clamp(min_visible_rows, max_visible_rows) as f32;
            let viewport_height = px((appearance.padding * 2.0) + (appearance.item_height * visible_rows));
            let allow_scrolling = self.model.scrolling && item_count > max_visible_rows;

            self.popup_surface.set_scrolling_enabled(allow_scrolling);
            self.popup_surface.configure(item_count.max(1), row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let list_content = if menu_items.is_empty() {
                div()
                    .flex()
                    .items_center()
                    .min_h(px(appearance.item_height))
                    .px(px(appearance.item_padding_x))
                    .text_size(px(appearance.item_typography.size))
                    .line_height(px(appearance.item_typography.line_height))
                    .font_weight(appearance.item_typography.weight)
                    .text_color(autocomplete_appearance.muted_text_color)
                    .child("No matches")
                    .into_any_element()
            } else {
                render_popup_rows(
                    &list_id,
                    &menu_items,
                    appearance.clone(),
                    self.behavior.state.highlighted_filtered,
                    item_hovers,
                    item_clicks,
                )
                .into_any_element()
            };

            (
                Some(div().w_full().into_any_element()),
                Some(self.popup_search_textfield.clone().into_any_element()),
                Some(self.popup_surface.render(list_content)),
            )
        } else {
            (None, None, None)
        };

        let handlers = SearchSelectorTemplateHandlers {
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            scroll_wheel: Box::new(cx.listener(Self::handle_popup_scroll_wheel)),
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_hover: Box::new(cx.listener(Self::handle_trigger_hover)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_trigger_mouse_down)),
            trigger_mouse_up: Box::new(cx.listener(Self::handle_trigger_mouse_up)),
            trigger_mouse_up_out: Box::new(cx.listener(Self::handle_trigger_mouse_up)),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
        };

        let interaction_state = self.interaction.render_state(true, window);
        let render_model = SearchSelectorRenderModel {
            id: self.model.id.clone(),
            trigger_label: selected_label.map(SharedString::from).unwrap_or_else(|| self.model.placeholder.clone()),
            trigger_label_is_placeholder: self.committed_selection.is_none(),
            trigger_state: TextFieldState {
                hovered: interaction_state.hovered,
                focused: self.behavior.state.open || self.trigger_focused,
                focus_visible: self.behavior.state.open || self.trigger_focused,
                ..TextFieldState::default()
            },
            full_width: self.model.full_width,
            minimum_trigger_width,
            status_label,
            status_detail,
            status_color: autocomplete_appearance.status_color,
            muted_text_color: autocomplete_appearance.muted_text_color,
            popup_bounds: self.trigger_bounds,
            popup_appearance: appearance,
            popup_content,
            popup_search_field,
            popup_list_panel,
        };

        div()
            .child(
                self.model
                    .template
                    .render(render_model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle()),
            )
            .into_any_element()
    }
}
