use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, Render, SharedString, Window, div,
    prelude::*,
};

use super::{TextAreaBuilder, TextAreaRenderModel, TextAreaState, TextAreaTemplateHandlers, model::TextAreaModel};

#[derive(Clone, Debug)]
pub enum TextAreaEvent {
    Change { value: String },
    Focus,
    Blur,
}

pub struct TextArea {
    model: TextAreaModel,
    state: TextAreaState,
    focus_handle: FocusHandle,
    change_count: usize,
    focus_count: usize,
    blur_count: usize,
}

impl EventEmitter<TextAreaEvent> for TextArea {}

fn typed_text_from_event(event: &KeyDownEvent) -> Option<&str> {
    let modifiers = &event.keystroke.modifiers;
    let shortcut_modifier = modifiers.secondary() || modifiers.control || modifiers.platform || modifiers.function;
    if shortcut_modifier {
        return None;
    }

    event
        .keystroke
        .key_char
        .as_deref()
        .filter(|text| !text.is_empty() && !text.chars().any(|ch| ch.is_control()))
}

impl TextArea {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TextAreaBuilder {
        TextAreaBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TextAreaBuilder, cx: &mut Context<Self>) -> Self {
        let mut state = TextAreaState::default();
        state.cursor = builder.model.value.chars().count();

        let mut this = Self {
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
            change_count: 0,
            focus_count: 0,
            blur_count: 0,
        };
        this.focus_handle = this.focus_handle.clone().tab_stop(this.model.enabled);
        this.recompute_invalid();
        this
    }

    pub fn value(&self) -> String {
        self.model.value.to_string()
    }

    pub fn state(&self) -> TextAreaState {
        self.state
    }

    pub fn change_count(&self) -> usize {
        self.change_count
    }

    pub fn focus_count(&self) -> usize {
        self.focus_count
    }

    pub fn blur_count(&self) -> usize {
        self.blur_count
    }

    pub fn is_focused(&self) -> bool {
        self.state.focused
    }

    pub fn rows(&self) -> usize {
        self.model.rows
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered(false);
        }
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.model.value = value.into().into();
        self.state.cursor = self.model.value.chars().count();
        self.recompute_invalid();
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>, cx: &mut Context<Self>) {
        self.model.placeholder = placeholder.into().into();
        cx.notify();
    }

    pub fn set_clean_on_escape(&mut self, clean_on_escape: bool, cx: &mut Context<Self>) {
        self.model.clean_on_escape = clean_on_escape;
        cx.notify();
    }

    pub fn set_validator(&mut self, validator: Option<super::model::Validator>, cx: &mut Context<Self>) {
        self.model.validator = validator;
        self.recompute_invalid();
        cx.notify();
    }

    pub fn set_rows(&mut self, rows: usize, cx: &mut Context<Self>) {
        self.model.rows = rows.max(2);
        cx.notify();
    }

    fn render_model<'a>(&'a self) -> TextAreaRenderModel<'a> {
        TextAreaRenderModel {
            id: &self.model.id,
            placeholder: &self.model.placeholder,
            value: &self.model.value,
            enabled: self.model.enabled,
            rows: self.model.rows,
            state: self.state,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> TextAreaTemplateHandlers {
        TextAreaTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover_changed)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            click: Box::new(cx.listener(Self::handle_click)),
            key_down: Box::new(cx.listener(Self::handle_key_down)),
        }
    }

    fn sync_focus(&mut self, window: &Window, cx: &mut Context<Self>) {
        let focused = self.focus_handle.is_focused(window);
        if self.state.focused == focused {
            return;
        }

        self.state.set_focused(focused);
        if focused {
            self.state.set_focus_visible(true);
            self.focus_count += 1;
            cx.emit(TextAreaEvent::Focus);
        } else {
            self.blur_count += 1;
            cx.emit(TextAreaEvent::Blur);
        }

        cx.notify();
    }

    fn recompute_invalid(&mut self) {
        let invalid = self.model.validator.as_ref().is_some_and(|validate| !validate(self.model.value.as_ref()));
        self.state.set_invalid(invalid);
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.change_count += 1;
        cx.emit(TextAreaEvent::Change { value: self.model.value.to_string() });
    }

    fn handle_hover_changed(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.state.set_hovered(*hovered);
        cx.notify();
    }

    fn handle_mouse_down(&mut self, _event: &gpui::MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.focus_handle.focus(window, cx);
        self.state.cursor = self.model.value.chars().count();
        cx.notify();
    }

    fn handle_click(&mut self, _event: &gpui::ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        let mut chars = self.model.value.chars().collect::<Vec<_>>();
        self.state.clamp_cursor(chars.len());
        let mut changed = false;
        let mut handled = false;

        match event.keystroke.key.as_str() {
            "backspace" => {
                if self.state.cursor > 0 {
                    let delete_ix = self.state.cursor - 1;
                    chars.remove(delete_ix);
                    self.state.cursor = delete_ix;
                    changed = true;
                }
                handled = true;
            }
            "delete" => {
                if self.state.cursor < chars.len() {
                    chars.remove(self.state.cursor);
                    changed = true;
                }
                handled = true;
            }
            "left" => {
                self.state.cursor = self.state.cursor.saturating_sub(1);
                handled = true;
            }
            "right" => {
                self.state.cursor = (self.state.cursor + 1).min(chars.len());
                handled = true;
            }
            "home" => {
                self.state.cursor = 0;
                handled = true;
            }
            "end" => {
                self.state.cursor = chars.len();
                handled = true;
            }
            "enter" => {
                chars.insert(self.state.cursor, '\n');
                self.state.cursor += 1;
                changed = true;
                handled = true;
            }
            "escape" => {
                if self.model.clean_on_escape && !chars.is_empty() {
                    chars.clear();
                    self.state.cursor = 0;
                    changed = true;
                }
                handled = true;
            }
            "tab" => {
                chars.insert(self.state.cursor, '\t');
                self.state.cursor += 1;
                changed = true;
                handled = true;
            }
            "space" => {
                chars.insert(self.state.cursor, ' ');
                self.state.cursor += 1;
                changed = true;
                handled = true;
            }
            _ => {
                if let Some(typed) = typed_text_from_event(event) {
                    for ch in typed.chars() {
                        chars.insert(self.state.cursor, ch);
                        self.state.cursor += 1;
                    }
                    changed = true;
                    handled = true;
                }
            }
        }

        if handled {
            window.prevent_default();
            cx.stop_propagation();
        }

        if changed {
            self.model.value = chars.into_iter().collect::<String>().into();
            self.recompute_invalid();
            self.emit_change(cx);
            cx.notify();
        } else if handled {
            cx.notify();
        }
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.is_focused(window) {
            window.blur();
        }

        self.sync_focus(window, cx);

        div()
            .child(
                self.model
                    .template
                    .render(&self.render_model(), self.template_handlers(cx), window, cx)
                    .track_focus(&self.focus_handle)
                    .tab_stop(self.model.enabled),
            )
            .into_any_element()
    }
}
