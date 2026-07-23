use gpui::{
    Bounds, ClickEvent, Context, Entity, EventEmitter, IntoElement, KeyDownEvent, Pixels, Render, ScrollWheelEvent,
    SharedString, Subscription, TextRun, Window, font, px,
};
use crate::controls::selector_panel::{SelectorItem, SelectorPanelClickHandler, SelectorPanelHoverHandler};
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::autocomplete::{AutocompleteTextBoxTheme, DefaultAutocompleteTextBoxTheme};

use super::behavior::{SelectionBehavior, SelectionEvent, SelectionStatus, SubmitResult};
use super::model::AutocompleteTextBoxBuilder;
use crate::controls::popup_scroll_surface::PopupScrollSurface;
use super::template::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplate, AutocompleteItemsTemplateHandlers,
    AutocompleteTextBoxRenderModel, AutocompleteTextBoxTemplate, AutocompleteTextBoxTemplateHandlers,
};
use super::text_selection::{self, TextSelectionEvent};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AutocompleteTextBoxEvent {
    Change { query: SharedString },
    Select { item_id: SharedString, label: SharedString },
    Complete { item_id: SharedString, label: SharedString },
    Clear,
    FocusChanged { focused: bool },
    OpenChanged { open: bool },
    Dismiss,
}

pub type AutocompleteTextBox = Entity<AutocompleteTextBoxControl>;

pub struct AutocompleteTextBoxControl {
    textfield: text_selection::TextSelection,
    popup_surface: PopupScrollSurface,
    model: super::model::AutocompleteTextBoxModel,
    behavior: SelectionBehavior,
    trigger_bounds: Option<Bounds<Pixels>>,
    last_event: SharedString,
    last_keyboard_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AutocompleteTextBoxEvent> for AutocompleteTextBoxControl {}

impl AutocompleteTextBoxControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = super::behavior::SelectionItem>,
    ) -> AutocompleteTextBoxBuilder {
        AutocompleteTextBoxBuilder::new(id, items)
    }

    pub(crate) fn from_builder(builder: AutocompleteTextBoxBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model;

        let textfield = text_selection::new(format!("{}-textfield", model.id))
            .placeholder(model.placeholder.clone())
            .enabled(model.enabled)
            .size(model.size)
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
                if let Some(event) = text_selection::map_event(event) {
                    this.handle_text_selection_event(event, cx);
                }
            }),
            cx.subscribe(&popup_surface.scrollbar(), |this, _, event: &ScrollbarEvent, cx| {
                if let ScrollbarEvent::Change { value } = event {
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
            _subscriptions: subscriptions,
        }
    }

    fn handle_text_selection_event(&mut self, event: TextSelectionEvent, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        match event {
            TextSelectionEvent::Change { value } => {
                let was_open = self.behavior.state.open;
                self.behavior.set_query(value.clone(), &self.model.items);
                self.last_event = SharedString::from("change");
                self.emit_open_changed_if_needed(was_open, false, cx);
                cx.emit(AutocompleteTextBoxEvent::Change { query: value.into() });
                cx.notify();
            }
            TextSelectionEvent::Submit { value } => {
                self.last_event = SharedString::from(format!("submit:{value}"));
                let result = self.behavior.apply(SelectionEvent::Submit, &self.model.items);
                self.handle_submit_result(result, cx);
            }
            TextSelectionEvent::FocusEnter => {
                let was_focused = self.behavior.state.focused;
                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Focus, &self.model.items);
                self.emit_focus_changed_if_needed(was_focused, cx);
                self.emit_open_changed_if_needed(was_open, false, cx);
                cx.notify();
            }
            TextSelectionEvent::FocusLeave => {
                let was_focused = self.behavior.state.focused;
                if self.behavior.state.status == SelectionStatus::Searching
                    && let Some(index) = self
                        .behavior
                        .state
                        .highlighted_filtered
                        .and_then(|highlighted| self.behavior.state.filtered.get(highlighted).copied())
                {
                    self.select_item(index, false, cx);
                }

                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Blur, &self.model.items);
                self.emit_focus_changed_if_needed(was_focused, cx);
                self.emit_open_changed_if_needed(was_open, true, cx);
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
        let was_open = self.behavior.state.open;
        self.behavior.apply(SelectionEvent::Clear, &self.model.items);
        self.last_event = SharedString::from("clear");

        self.textfield.update(cx, |textfield, cx| textfield.set_value("", cx));

        self.emit_open_changed_if_needed(was_open, false, cx);
        cx.emit(AutocompleteTextBoxEvent::Clear);
        cx.notify();
    }

    fn select_item(&mut self, index: usize, exact_complete: bool, cx: &mut Context<Self>) {
        let item = &self.model.items[index];
        let item_id = item.id.clone();
        let label = item.label.clone();
        let was_open = self.behavior.state.open;

        self.behavior.set_query(label.clone(), &self.model.items);
        self.behavior.select_index(index);
        self.last_event = SharedString::from(format!("select:{}", item.label));

        self.textfield.update(cx, move |textfield, cx| textfield.set_value(label.as_ref(), cx));

        let label = self.model.items[index].label.clone();
        self.emit_open_changed_if_needed(was_open, false, cx);
        if exact_complete {
            cx.emit(AutocompleteTextBoxEvent::Complete { item_id, label });
        } else {
            cx.emit(AutocompleteTextBoxEvent::Select { item_id, label });
        }
        cx.notify();
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
                let was_open = self.behavior.state.open;
                self.behavior.apply(SelectionEvent::Escape, &self.model.items);
                self.emit_open_changed_if_needed(was_open, true, cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "down" | "arrowdown" if self.behavior.state.open && !self.behavior.state.filtered.is_empty() => {
                self.behavior.apply(SelectionEvent::MoveNext, &self.model.items);
                self.sync_popup_highlight_visibility(cx);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
            }
            "up" | "arrowup" if self.behavior.state.open && !self.behavior.state.filtered.is_empty() => {
                self.behavior.apply(SelectionEvent::MovePrevious, &self.model.items);
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

    pub fn set_size(&mut self, size: crate::theme::ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        self.textfield.update(cx, |textfield, cx| textfield.set_size(size, cx));
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        if !enabled {
            let was_open = self.behavior.state.open;
            self.behavior.state.open = false;
            self.behavior.state.highlighted_filtered = None;
            self.emit_open_changed_if_needed(was_open, false, cx);
        }
        self.textfield.update(cx, |textfield, cx| textfield.set_enabled(enabled, cx));
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn AutocompleteTextBoxTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_items_template(
        &mut self,
        template: std::sync::Arc<dyn AutocompleteItemsTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.items_template = template;
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

    fn emit_open_changed_if_needed(&mut self, was_open: bool, dismissed: bool, cx: &mut Context<Self>) -> bool {
        if was_open == self.behavior.state.open {
            return false;
        }

        cx.emit(AutocompleteTextBoxEvent::OpenChanged { open: self.behavior.state.open });
        if dismissed && !self.behavior.state.open {
            cx.emit(AutocompleteTextBoxEvent::Dismiss);
        }
        true
    }

    fn emit_focus_changed_if_needed(&mut self, was_focused: bool, cx: &mut Context<Self>) -> bool {
        if was_focused == self.behavior.state.focused {
            return false;
        }

        cx.emit(AutocompleteTextBoxEvent::FocusChanged { focused: self.behavior.state.focused });
        true
    }
}

impl Render for AutocompleteTextBoxControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = crate::theme::ThemeTokens::default();
        let autocomplete_look = DefaultAutocompleteTextBoxTheme::new(tokens.clone()).resolve(self.model.size);
        let look = (self.model.popup_look_provider)(self.model.size);
        let selected_label = self
            .behavior
            .state
            .selected_item
            .map(|index| self.model.items[index].label.to_string())
            .unwrap_or_else(|| "none".to_string());

        let menu_items = self
            .behavior
            .state
            .filtered
            .iter()
            .enumerate()
            .map(|(visible_index, item_index)| {
                let item = &self.model.items[*item_index];
                SelectorItem::new(format!("autocomplete-item-{}-{visible_index}", item.id))
                    .label(item.label.clone())
                    .enabled(item.enabled)
            })
            .collect::<Vec<_>>();

        let item_hovers = (0..menu_items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as SelectorPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..menu_items.len())
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

        let trigger_width = self.trigger_bounds.map(|bounds| bounds.size.width).unwrap_or(px(240.0));
        let popup_width = {
            let mut text_font = font(".SystemUIFont");
            text_font.weight = look.item_typography.weight;
            let max_label_width = menu_items
                .iter()
                .map(|item| {
                    let label = item.label_text();
                    let run = TextRun {
                        len: label.len(),
                        font: text_font.clone(),
                        color: look.foreground,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    window.text_system().shape_line(label.clone(), px(look.item_typography.size), &[run], None).width()
                })
                .max_by(|a, b| a.as_f32().total_cmp(&b.as_f32()))
                .unwrap_or(px(0.0));

            let horizontal_chrome = px((look.padding * 2.0) + (look.item_padding_x * 2.0) + 24.0);
            trigger_width.max(max_label_width + horizontal_chrome)
        };

        let popup_content = if self.model.enabled && self.behavior.state.open && !menu_items.is_empty() {
            let menu_id = SharedString::from("autocomplete-menu");
            let row_height = px(look.item_height);
            let content_top_padding = px(look.padding);
            let min_visible_rows = 7.0;
            let max_visible_rows = 14.0;
            let visible_rows = (menu_items.len() as f32).clamp(min_visible_rows, max_visible_rows);
            let viewport_height = px((look.padding * 2.0) + (look.item_height * visible_rows));

            self.popup_surface.configure(menu_items.len(), row_height, content_top_padding, viewport_height);
            self.popup_surface.sync(cx);

            let menu_content = self.model.items_template.render(
                &AutocompleteItemsRenderModel {
                    id: &menu_id,
                    items: &menu_items,
                    look: look.clone(),
                    highlighted_index: self.behavior.state.highlighted_filtered,
                },
                AutocompleteItemsTemplateHandlers { item_hovers, item_clicks },
            );

            Some(self.popup_surface.render(menu_content.into_any_element()))
        } else {
            None
        };

        let handlers = AutocompleteTextBoxTemplateHandlers {
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            scroll_wheel: Box::new(cx.listener(Self::handle_popup_scroll_wheel)),
            clear_click: Box::new(cx.listener(Self::handle_clear_click)),
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
        };

        let render_model = AutocompleteTextBoxRenderModel {
            id: self.model.id.clone(),
            textfield: self.textfield.clone().into_any_element(),
            query_is_empty: self.behavior.state.query.is_empty(),
            popup_width,
            status_label,
            status_detail,
            status_color: autocomplete_look.status_color,
            muted_text_color: autocomplete_look.muted_text_color,
            popup_bounds: self.trigger_bounds,
            popup_look: look,
            popup_content,
        };

        self.model.template.render(render_model, handlers, window, cx)
    }
}
