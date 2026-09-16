mod chrome;
mod element;
mod input;
mod layout;

use std::{ops::Range, time::Duration};

use gpui::{
    App, Context, Empty, EventEmitter, Entity, FocusHandle, Focusable, IntoElement, Pixels, Point, Render,
    SharedString, Subscription, Task, Window, px,
};

use super::{TextAreaBuilder, TextAreaState, model::TextAreaModel};
use crate::controls::scrollbar::{Scrollbar, ScrollbarEvent, ScrollbarOrientation};
use crate::controls::text::select_all;
use crate::theme::{StandardBoxScale, observe_theme_revision};

use self::layout::TextAreaLayoutCache;

pub(super) const FOCUS_RING_GAP: f32 = 1.0;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TextAreaEvent {
    Change { value: String },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
pub struct TextAreaDrag {
    id: SharedString,
}

impl TextAreaDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for TextAreaDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

#[derive(Clone, Debug)]
pub struct TextAreaResizeDrag {
    id: SharedString,
}

impl TextAreaResizeDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for TextAreaResizeDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct TextArea {
    model: TextAreaModel,
    state: TextAreaState,
    focus_handle: FocusHandle,
    theme_epoch: u64,
    change_count: usize,
    focus_count: usize,
    blur_count: usize,
    caret_visible: bool,
    caret_paused: bool,
    caret_epoch: usize,
    caret_task: Task<()>,
    mouse_selecting: bool,
    selection_drag_position: Option<Point<Pixels>>,
    selection_scroll_epoch: usize,
    selection_scroll_task: Task<()>,
    suppress_select_all_on_next_focus: bool,
    marked_range: Option<Range<usize>>,
    vertical_scroll: Pixels,
    layout_cache: Option<TextAreaLayoutCache>,
    scrollbar: Entity<Scrollbar>,
    scrollbar_enabled: bool,
    resize_dragging: bool,
    resize_start_y: f32,
    resize_start_rows: usize,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<TextAreaEvent> for TextArea {}

impl TextArea {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TextAreaBuilder {
        TextAreaBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TextAreaBuilder, cx: &mut Context<Self>) -> Self {
        let mut state = TextAreaState { cursor: builder.model.value.chars().count(), ..Default::default() };
        let initial_rows = builder.model.rows.max(1);

        let scrollbar = Scrollbar::new(format!("{}-scrollbar", builder.model.id))
            .orientation(ScrollbarOrientation::Vertical)
            .spawn(cx);
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
            if let ScrollbarEvent::Change { value } = event {
                this.vertical_scroll = px(*value);
                this.clamp_vertical_scroll_to_cache();
                cx.notify();
            }
        })];

        let mut this = Self {
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
            theme_epoch: 0,
            change_count: 0,
            focus_count: 0,
            blur_count: 0,
            caret_visible: false,
            caret_paused: false,
            caret_epoch: 0,
            caret_task: Task::ready(()),
            mouse_selecting: false,
            selection_drag_position: None,
            selection_scroll_epoch: 0,
            selection_scroll_task: Task::ready(()),
            suppress_select_all_on_next_focus: false,
            marked_range: None,
            vertical_scroll: px(0.0),
            layout_cache: None,
            scrollbar,
            scrollbar_enabled: false,
            resize_dragging: false,
            resize_start_y: 0.0,
            resize_start_rows: initial_rows,
            _subscriptions: subscriptions,
        };
        state.clamp_cursor(this.model.value.chars().count());
        this.state = state;
        this.focus_handle = this.focus_handle.clone().tab_stop(this.model.enabled);
        this.recompute_invalid();
        this._subscriptions.push(observe_theme_revision(cx, |this, cx| {
            this.layout_cache = None;
            this.theme_epoch = this.theme_epoch.wrapping_add(1);
            cx.notify();
        }));
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

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered(false);
            self.mouse_selecting = false;
            self.stop_selection_autoscroll();
        }
        cx.emit(TextAreaEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.model.value = value.into().into();
        self.state.cursor = self.model.value.chars().count();
        self.state.clear_selection();
        self.state.preferred_column = None;
        self.marked_range = None;
        self.recompute_invalid();
        self.ensure_cursor_visible();
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

    pub fn set_max_clipboard_paste_bytes(&mut self, max_bytes: Option<usize>, cx: &mut Context<Self>) {
        self.model.max_clipboard_paste_bytes = max_bytes;
        cx.notify();
    }

    pub fn set_validator(&mut self, validator: Option<super::model::Validator>, cx: &mut Context<Self>) {
        self.model.validator = validator;
        self.recompute_invalid();
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::TextAreaTemplate>, cx: &mut Context<Self>) {
        if let Some(theme) = template.theme() {
            self.model.theme = theme;
        }
        self.model.template = template;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::TextAreaTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_look_override(
        &mut self,
        look_override: Option<super::model::TextAreaLookOverride>,
        cx: &mut Context<Self>,
    ) {
        self.model.look_override = look_override;
        self.layout_cache = None;
        cx.notify();
    }

    fn resolved_look(&self, scale: &StandardBoxScale) -> crate::controls::textarea::TextAreaLook {
        let base_state = TextAreaState { focused: false, focus_visible: false, ..self.state };
        let mut look = self.model.theme.resolve_look(base_state, self.model.enabled, self.model.size, scale);
        let focus_state = TextAreaState { focused: true, focus_visible: true, ..self.state };
        let focus_look = self.model.theme.resolve_look(focus_state, self.model.enabled, self.model.size, scale);
        if let Some(override_fn) = &self.model.look_override {
            look = override_fn(look);
            look.focus_border = Some(override_fn(focus_look).border);
        } else if self.model.enabled && !self.state.invalid {
            look.focus_border = Some(focus_look.border);
        }
        look
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
                select_all(&mut self.state, self.model.value.chars().count());
            }
            self.state.preferred_column = None;
            self.state.set_focus_visible(true);
            self.focus_count += 1;
            cx.emit(TextAreaEvent::FocusChanged { focused: true });
            self.start_caret_blink(cx);
        } else {
            if !self.mouse_selecting {
                self.stop_selection_autoscroll();
            }
            if self.model.select_all_on_tab_focus {
                self.state.clear_selection();
                self.state.cursor = self.model.value.chars().count();
            }
            self.blur_count += 1;
            cx.emit(TextAreaEvent::FocusChanged { focused: false });
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
        cx.emit(TextAreaEvent::Change { value: self.model.value.to_string() });
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
