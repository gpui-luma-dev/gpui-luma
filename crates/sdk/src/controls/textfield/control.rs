use std::time::Duration;

use gpui::{
    App, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Render, SharedString, Task, Window, div, prelude::*,
};

use super::{TextFieldBuilder, TextFieldRenderModel, TextFieldState, TextFieldTemplateHandlers, model::TextFieldModel};

#[derive(Clone, Debug)]
pub enum TextFieldEvent {
    Change { value: String },
    Submit { value: String },
    Focus,
    Blur,
}

pub struct TextField {
    model: TextFieldModel,
    state: TextFieldState,
    focus_handle: FocusHandle,
    change_count: usize,
    submit_count: usize,
    focus_count: usize,
    blur_count: usize,
    caret_visible: bool,
    caret_paused: bool,
    caret_epoch: usize,
    caret_task: Task<()>,
    mouse_selecting: bool,
    suppress_select_all_on_next_focus: bool,
}

impl EventEmitter<TextFieldEvent> for TextField {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharClass {
    Word,
    Whitespace,
    Other,
}

impl CharClass {
    fn from_char(ch: char) -> Self {
        if is_word_char(ch) {
            Self::Word
        } else if ch.is_whitespace() {
            Self::Whitespace
        } else {
            Self::Other
        }
    }

    fn is_connectable(self, ch: char) -> bool {
        Self::from_char(ch) == self
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn prev_word_boundary(chars: &[char], cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }

    let mut ix = cursor;
    while ix > 0 && chars.get(ix - 1).is_some_and(|ch| ch.is_whitespace()) {
        ix -= 1;
    }
    while ix > 0 && chars.get(ix - 1).is_some_and(|ch| is_word_char(*ch)) {
        ix -= 1;
    }
    ix
}

fn next_word_boundary(chars: &[char], cursor: usize) -> usize {
    let mut ix = cursor.min(chars.len());
    while ix < chars.len() && chars.get(ix).is_some_and(|ch| ch.is_whitespace()) {
        ix += 1;
    }
    while ix < chars.len() && chars.get(ix).is_some_and(|ch| is_word_char(*ch)) {
        ix += 1;
    }
    ix
}

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

fn move_left_target(chars: &[char], cursor: usize, modifiers: gpui::Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return 0;
        }
        if modifiers.alt {
            return prev_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return prev_word_boundary(chars, cursor);
        }
    }

    cursor.saturating_sub(1)
}

fn move_right_target(chars: &[char], cursor: usize, modifiers: gpui::Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return chars.len();
        }
        if modifiers.alt {
            return next_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return next_word_boundary(chars, cursor);
        }
    }

    (cursor + 1).min(chars.len())
}

fn delete_left_target(chars: &[char], cursor: usize, modifiers: gpui::Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return 0;
        }
        if modifiers.alt {
            return prev_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return prev_word_boundary(chars, cursor);
        }
    }

    cursor.saturating_sub(1)
}

fn delete_right_target(chars: &[char], cursor: usize, modifiers: gpui::Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return chars.len();
        }
        if modifiers.alt {
            return next_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return next_word_boundary(chars, cursor);
        }
    }

    (cursor + 1).min(chars.len())
}

impl TextField {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
        TextFieldBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TextFieldBuilder, cx: &mut Context<Self>) -> Self {
        let mut state = TextFieldState::default();
        state.cursor = builder.model.value.chars().count();

        let mut this = Self {
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
            change_count: 0,
            submit_count: 0,
            focus_count: 0,
            blur_count: 0,
            caret_visible: false,
            caret_paused: false,
            caret_epoch: 0,
            caret_task: Task::ready(()),
            mouse_selecting: false,
            suppress_select_all_on_next_focus: false,
        };
        this.focus_handle = this.focus_handle.clone().tab_stop(this.model.enabled);
        this.recompute_invalid();
        this
    }

    pub fn value(&self) -> String {
        self.model.value.to_string()
    }

    pub fn state(&self) -> TextFieldState {
        self.state
    }

    pub fn change_count(&self) -> usize {
        self.change_count
    }

    pub fn submit_count(&self) -> usize {
        self.submit_count
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

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered(false);
            self.mouse_selecting = false;
        }
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.model.value = value.into().into();
        self.state.cursor = self.model.value.chars().count();
        self.state.clear_selection();
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

    fn render_model<'a>(&'a self) -> TextFieldRenderModel<'a> {
        TextFieldRenderModel {
            id: &self.model.id,
            placeholder: &self.model.placeholder,
            value: &self.model.value,
            prefix_icon: self.model.prefix_icon.as_ref(),
            enabled: self.model.enabled,
            full_width: self.model.full_width,
            state: self.state,
            caret_visible: self.state.focused && self.model.enabled && self.caret_visible,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> TextFieldTemplateHandlers {
        let char_count = self.model.value.chars().count() + 1;

        TextFieldTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover_changed)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            click: Box::new(cx.listener(Self::handle_click)),
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            cell_mouse_down: (0..char_count)
                .map(|index| {
                    Box::new(cx.listener(move |this: &mut Self, event: &MouseDownEvent, window: &mut Window, cx| {
                        this.handle_cell_mouse_down(index, event, window, cx);
                    })) as _
                })
                .collect(),
            cell_mouse_move: (0..char_count)
                .map(|index| {
                    Box::new(cx.listener(move |this: &mut Self, event: &MouseMoveEvent, window: &mut Window, cx| {
                        this.handle_cell_mouse_move(index, event, window, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn sync_focus(&mut self, window: &Window, cx: &mut Context<Self>) {
        let focused = self.focus_handle.is_focused(window);
        if self.state.focused == focused {
            return;
        }

        self.state.set_focused(focused);

        if focused {
            let keyboard_focus = window.last_input_was_keyboard();
            if self.model.select_all_on_tab_focus && keyboard_focus && !self.suppress_select_all_on_next_focus {
                self.select_all(self.model.value.chars().count());
            }
            self.state.set_focus_visible(true);
            self.focus_count += 1;
            cx.emit(TextFieldEvent::Focus);
            self.start_caret_blink(cx);
        } else {
            self.mouse_selecting = false;
            if self.model.select_all_on_tab_focus {
                self.state.clear_selection();
                self.state.cursor = self.model.value.chars().count();
            }
            self.blur_count += 1;
            cx.emit(TextFieldEvent::Blur);
            self.stop_caret_blink(cx);
        }

        self.suppress_select_all_on_next_focus = false;
        cx.notify();
    }

    fn next_caret_epoch(&mut self) -> usize {
        self.caret_epoch += 1;
        self.caret_epoch
    }

    fn start_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_paused = false;
        self.caret_visible = true;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(500)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_caret_blink(epoch, cx));
            }
        });
    }

    fn stop_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_epoch = 0;
        self.caret_paused = false;
        self.caret_visible = false;
        cx.notify();
    }

    fn pause_caret_blink(&mut self, cx: &mut Context<Self>) {
        if !self.state.focused {
            return;
        }

        self.caret_paused = true;
        self.caret_visible = true;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(300)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.caret_paused = false;
                    this.tick_caret_blink(epoch, cx);
                });
            }
        });
    }

    fn tick_caret_blink(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if !self.state.focused {
            self.caret_visible = false;
            return;
        }

        if self.caret_paused || epoch != self.caret_epoch {
            self.caret_visible = true;
            return;
        }

        self.caret_visible = !self.caret_visible;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(500)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_caret_blink(epoch, cx));
            }
        });
    }

    fn recompute_invalid(&mut self) {
        let invalid = self.model.validator.as_ref().is_some_and(|validate| !validate(self.model.value.as_ref()));
        self.state.set_invalid(invalid);
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.change_count += 1;
        cx.emit(TextFieldEvent::Change { value: self.model.value.to_string() });
    }

    fn delete_selection(&mut self, chars: &mut Vec<char>) -> bool {
        let Some((start, end)) = self.state.selection_range() else {
            return false;
        };
        chars.drain(start..end);
        self.state.cursor = start;
        self.state.clear_selection();
        true
    }

    fn insert_text(&mut self, chars: &mut Vec<char>, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }

        let _ = self.delete_selection(chars);
        let mut inserted = 0usize;
        for ch in value.chars() {
            chars.insert(self.state.cursor + inserted, ch);
            inserted += 1;
        }
        self.state.cursor += inserted;
        self.state.clear_selection();
        inserted > 0
    }

    fn word_cluster_range(chars: &[char], offset: usize) -> Option<(usize, usize)> {
        if chars.is_empty() {
            return None;
        }

        let target = offset.min(chars.len().saturating_sub(1));
        let class = CharClass::from_char(chars[target]);
        let mut start = target;
        let mut end = target + 1;

        while start > 0 && class.is_connectable(chars[start - 1]) {
            start -= 1;
        }
        while end < chars.len() && class.is_connectable(chars[end]) {
            end += 1;
        }

        Some((start, end))
    }

    fn select_all(&mut self, len: usize) {
        self.state.selection_anchor = Some(0);
        self.state.cursor = len;
    }

    fn handle_hover_changed(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.state.set_hovered(*hovered);
        cx.notify();
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.suppress_select_all_on_next_focus = true;
        self.focus_handle.focus(window, cx);
        let len = self.model.value.chars().count();
        if event.modifiers.shift {
            self.state.set_cursor(len, true);
        } else {
            self.state.set_cursor(len, false);
        }
        self.mouse_selecting = true;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_cell_mouse_down(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.suppress_select_all_on_next_focus = true;
        self.focus_handle.focus(window, cx);
        cx.stop_propagation();

        let chars = self.model.value.chars().collect::<Vec<_>>();
        let len = chars.len();
        let index = index.min(len);

        if event.click_count >= 3 {
            self.select_all(len);
            self.mouse_selecting = false;
            self.pause_caret_blink(cx);
            cx.notify();
            return;
        }

        if event.click_count == 2 {
            if let Some((start, end)) = Self::word_cluster_range(&chars, index) {
                self.state.selection_anchor = Some(start);
                self.state.cursor = end;
                self.mouse_selecting = false;
                self.pause_caret_blink(cx);
                cx.notify();
            }
            return;
        }

        if event.modifiers.shift {
            self.state.set_cursor(index, true);
        } else {
            self.state.set_cursor(index, false);
        }

        self.mouse_selecting = true;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_cell_mouse_move(
        &mut self,
        index: usize,
        _event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || !self.mouse_selecting {
            return;
        }

        self.state.set_cursor(index.min(self.model.value.chars().count()), true);
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.mouse_selecting = false;
        if self.state.selection_range().is_none() {
            self.state.clear_selection();
        }
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
        let selecting = event.keystroke.modifiers.shift;
        let secondary = event.keystroke.modifiers.secondary();
        let modifiers = event.keystroke.modifiers;

        match event.keystroke.key.as_str() {
            "backspace" => {
                if self.delete_selection(&mut chars) {
                    changed = true;
                } else {
                    let next = delete_left_target(&chars, self.state.cursor, modifiers);
                    if next < self.state.cursor {
                        chars.drain(next..self.state.cursor);
                        self.state.cursor = next;
                        changed = true;
                    }
                }
                handled = true;
            }
            "delete" => {
                if self.delete_selection(&mut chars) {
                    changed = true;
                } else {
                    let next = delete_right_target(&chars, self.state.cursor, modifiers);
                    if next > self.state.cursor {
                        chars.drain(self.state.cursor..next);
                        changed = true;
                    }
                }
                handled = true;
            }
            "left" => {
                let next = if !selecting {
                    if let Some((start, _)) = self.state.selection_range() {
                        start
                    } else {
                        move_left_target(&chars, self.state.cursor, modifiers)
                    }
                } else {
                    move_left_target(&chars, self.state.cursor, modifiers)
                };
                self.state.set_cursor(next, selecting);
                handled = true;
            }
            "right" => {
                let next = if !selecting {
                    if let Some((_, end)) = self.state.selection_range() {
                        end
                    } else {
                        move_right_target(&chars, self.state.cursor, modifiers)
                    }
                } else {
                    move_right_target(&chars, self.state.cursor, modifiers)
                };
                self.state.set_cursor(next, selecting);
                handled = true;
            }
            "home" => {
                self.state.set_cursor(0, selecting);
                handled = true;
            }
            "end" => {
                self.state.set_cursor(chars.len(), selecting);
                handled = true;
            }
            "enter" => {
                self.submit_count += 1;
                cx.emit(TextFieldEvent::Submit { value: self.model.value.to_string() });
                handled = true;
            }
            "tab" => {
                let was_focused = self.focus_handle.is_focused(window);
                if modifiers.shift {
                    window.focus_prev(cx);
                } else {
                    window.focus_next(cx);
                }
                handled = was_focused && !self.focus_handle.is_focused(window);
            }
            "escape" => {
                if self.model.clean_on_escape && !chars.is_empty() {
                    chars.clear();
                    self.state.cursor = 0;
                    self.state.clear_selection();
                    changed = true;
                }
                handled = true;
            }
            "a" if secondary => {
                self.select_all(chars.len());
                handled = true;
            }
            "c" if secondary => {
                if let Some((start, end)) = self.state.selection_range() {
                    let selected_text: String = self.model.value.chars().skip(start).take(end - start).collect();
                    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
                }
                handled = true;
            }
            "x" if secondary => {
                if let Some((start, end)) = self.state.selection_range() {
                    let selected_text: String = self.model.value.chars().skip(start).take(end - start).collect();
                    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
                    if self.delete_selection(&mut chars) {
                        changed = true;
                    }
                }
                handled = true;
            }
            "v" if secondary => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    let text = text.replace('\n', "");
                    if !text.is_empty() {
                        changed = self.insert_text(&mut chars, &text);
                    }
                }
                handled = true;
            }
            _ => {
                if let Some(typed) = typed_text_from_event(event) {
                    changed = self.insert_text(&mut chars, typed);
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
        }

        if changed || handled {
            self.pause_caret_blink(cx);
            cx.notify();
        }
    }
}

impl Focusable for TextField {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextField {
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

#[cfg(test)]
mod tests {
    use gpui::{KeyDownEvent, Keystroke, Modifiers};

    use super::{
        TextField, delete_left_target, delete_right_target, move_left_target, move_right_target, typed_text_from_event,
    };

    #[test]
    fn typed_text_uses_key_char() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "a".into(), key_char: Some("A".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), Some("A"));
    }

    #[test]
    fn typed_text_ignores_secondary_shortcuts() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::secondary_key(), key: "a".into(), key_char: Some("a".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), None);
    }

    #[test]
    fn typed_text_ignores_tab_control_character() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "tab".into(), key_char: Some("\t".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), None);
    }

    #[test]
    fn movement_targets_clamp_to_bounds() {
        let chars = "hello world".chars().collect::<Vec<_>>();
        assert_eq!(move_left_target(&chars, 0, Modifiers::none()), 0);
        assert_eq!(move_right_target(&chars, chars.len(), Modifiers::none()), chars.len());
        assert_eq!(delete_left_target(&chars, 0, Modifiers::none()), 0);
        assert_eq!(delete_right_target(&chars, chars.len(), Modifiers::none()), chars.len());
    }

    #[test]
    fn double_click_word_cluster_selects_word() {
        let chars = "abc 123".chars().collect::<Vec<_>>();
        assert_eq!(TextField::word_cluster_range(&chars, 1), Some((0, 3)));
        assert_eq!(TextField::word_cluster_range(&chars, 4), Some((4, 7)));
    }

    #[test]
    fn double_click_word_cluster_selects_whitespace_run() {
        let chars = "a   b".chars().collect::<Vec<_>>();
        assert_eq!(TextField::word_cluster_range(&chars, 2), Some((1, 4)));
    }
}
