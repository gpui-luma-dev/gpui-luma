use std::{ops::Range, time::Duration};

use gpui::{
    App, Bounds, ClipboardItem, Context, DragMoveEvent, ElementInputHandler, Empty, EntityInputHandler, EventEmitter,
    Entity, FocusHandle, Focusable, GlobalElementId, IntoElement, KeyDownEvent, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, Render, ScrollWheelEvent, SharedString, ShapedLine, Style,
    Subscription, Task, TextAlign, TextRun, UTF16Selection, Window, div, fill, font, point, prelude::*, px, relative,
    size,
};

use lucide_icons::Icon as LucideIcon;

use super::{TextAreaBuilder, TextAreaState, TextAreaTemplateHandlers, model::TextAreaModel};
use crate::controls::scrollbar::{Scrollbar, ScrollbarEvent, ScrollbarOrientation};
use crate::controls::text::{EditableTextPolicy, FocusNavigation, handle_key_down, select_all, word_cluster_range};
use crate::controls::value::ControlRange;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ControlSize, LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

const TEXTAREA_RESIZE_ICON_SIZE: f32 = 10.0;

#[derive(Clone)]
struct TextAreaCachedLine {
    start: usize,
    end: usize,
    text: String,
    line: ShapedLine,
}

#[derive(Clone)]
struct TextAreaLayoutCache {
    text_viewport: Bounds<Pixels>,
    line_height: Pixels,
    lines: Vec<TextAreaCachedLine>,
}

struct TextAreaElement {
    input: gpui::Entity<TextArea>,
}

struct TextAreaPrepaintState {
    lines: Vec<TextAreaCachedLine>,
    selection_quads: Vec<PaintQuad>,
    caret_quad: Option<PaintQuad>,
    placeholder_line: Option<ShapedLine>,
    line_height: Pixels,
    vertical_scroll: Pixels,
}

fn colored_runs_for_text(
    text: &str,
    global_offset: usize,
    selection: Option<(usize, usize)>,
    appearance: &crate::controls::textarea::TextAreaAppearance,
    font_family: String,
    font_weight: gpui::FontWeight,
) -> Vec<TextRun> {
    let mut base_font = font(font_family);
    base_font.weight = font_weight;
    let run = |value: &str, color: gpui::Hsla| TextRun {
        len: value.len(),
        font: base_font.clone(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };

    if text.is_empty() {
        return Vec::new();
    }

    let Some((sel_start, sel_end)) = selection else {
        return vec![run(text, appearance.foreground)];
    };

    let mut runs = Vec::new();
    let mut chunk = String::new();
    let mut chunk_selected = None::<bool>;

    for (char_ix, ch) in text.chars().enumerate() {
        let selected = global_offset + char_ix >= sel_start && global_offset + char_ix < sel_end;
        match chunk_selected {
            None => {
                chunk_selected = Some(selected);
                chunk.push(ch);
            }
            Some(current) if current == selected => chunk.push(ch),
            Some(current) => {
                runs.push(run(
                    &chunk,
                    if current {
                        appearance.selection_foreground
                    } else {
                        appearance.foreground
                    },
                ));
                chunk.clear();
                chunk_selected = Some(selected);
                chunk.push(ch);
            }
        }
    }

    if let Some(current) = chunk_selected {
        runs.push(run(
            &chunk,
            if current {
                appearance.selection_foreground
            } else {
                appearance.foreground
            },
        ));
    }

    runs
}

#[derive(Clone, Debug)]
pub enum TextAreaEvent {
    Change { value: String },
    Focus,
    Blur,
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
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| match event {
            ScrollbarEvent::Change { value } => {
                this.vertical_scroll = px(*value);
                this.clamp_vertical_scroll_to_cache();
                cx.notify();
            }
        })];

        let mut this = Self {
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
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
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered(false);
            self.mouse_selecting = false;
            self.stop_selection_autoscroll();
        }
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

    pub fn set_validator(&mut self, validator: Option<super::model::Validator>, cx: &mut Context<Self>) {
        self.model.validator = validator;
        self.recompute_invalid();
        cx.notify();
    }

    fn logical_lines(value: &str) -> Vec<(usize, usize, String)> {
        let mut lines = Vec::new();
        let mut start = 0usize;
        let mut current = String::new();

        for (ix, ch) in value.chars().enumerate() {
            if ch == '\n' {
                lines.push((start, ix, std::mem::take(&mut current)));
                start = ix + 1;
            } else {
                current.push(ch);
            }
        }

        lines.push((start, value.chars().count(), current));
        lines
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> TextAreaTemplateHandlers {
        TextAreaTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover_changed)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_move: Box::new(cx.listener(Self::handle_mouse_move)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            click: Box::new(cx.listener(Self::handle_click)),
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
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
                select_all(&mut self.state, self.model.value.chars().count());
            }
            self.state.preferred_column = None;
            self.state.set_focus_visible(true);
            self.focus_count += 1;
            cx.emit(TextAreaEvent::Focus);
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
            cx.emit(TextAreaEvent::Blur);
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

    fn next_selection_scroll_epoch(&mut self) -> usize {
        self.selection_scroll_epoch += 1;
        self.selection_scroll_epoch
    }

    fn stop_selection_autoscroll(&mut self) {
        self.selection_drag_position = None;
        self.next_selection_scroll_epoch();
        self.selection_scroll_task = Task::ready(());
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

    fn resize_drag_id(&self) -> SharedString {
        format!("{}-resize", self.model.id).into()
    }

    fn resize_line_height(&self) -> f32 {
        self.layout_cache.as_ref().map(|cache| cache.line_height.as_f32()).unwrap_or(20.0).max(1.0)
    }

    fn stop_resize_drag(&mut self) {
        self.resize_dragging = false;
    }

    fn max_vertical_scroll_for_cache(cache: &TextAreaLayoutCache) -> Pixels {
        (cache.line_height * cache.lines.len() as f32 - cache.text_viewport.size.height).max(px(0.0))
    }

    fn clamped_vertical_scroll(cache: &TextAreaLayoutCache, scroll: Pixels) -> Pixels {
        scroll.max(px(0.0)).min(Self::max_vertical_scroll_for_cache(cache))
    }

    fn clamp_vertical_scroll_to_cache(&mut self) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let clamped = Self::clamped_vertical_scroll(cache, self.vertical_scroll);
        if clamped == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = clamped;
        true
    }

    fn scroll_by(&mut self, delta: Pixels) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let next = Self::clamped_vertical_scroll(cache, self.vertical_scroll + delta);
        if next == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = next;
        true
    }

    fn selection_autoscroll_delta_for_point(&self, position: Point<Pixels>) -> Pixels {
        let Some(cache) = self.layout_cache.as_ref() else {
            return px(0.0);
        };

        Self::selection_autoscroll_delta_for_cache(cache, position)
    }

    fn selection_autoscroll_delta_for_cache(cache: &TextAreaLayoutCache, position: Point<Pixels>) -> Pixels {
        if Self::max_vertical_scroll_for_cache(cache) <= px(0.5) {
            return px(0.0);
        }

        let top = cache.text_viewport.top();
        let bottom = cache.text_viewport.bottom();
        let overflow = if position.y < top {
            position.y - top
        } else if position.y > bottom {
            position.y - bottom
        } else {
            px(0.0)
        };

        if overflow == px(0.0) {
            return px(0.0);
        }

        let direction = overflow.as_f32().signum();
        let distance = overflow.as_f32().abs();
        let line_height = cache.line_height.as_f32().max(1.0);
        let lines_per_tick = (distance / line_height).clamp(1.0, 6.0);
        px(direction * line_height * lines_per_tick)
    }

    fn selection_hit_position(&self, position: Point<Pixels>) -> Point<Pixels> {
        let Some(cache) = self.layout_cache.as_ref() else {
            return position;
        };

        Self::selection_hit_position_for_cache(cache, position)
    }

    fn selection_hit_position_for_cache(cache: &TextAreaLayoutCache, position: Point<Pixels>) -> Point<Pixels> {
        let top = cache.text_viewport.top();
        let bottom = cache.text_viewport.bottom();
        let left = cache.text_viewport.left();
        let right = cache.text_viewport.right() - px(0.5);

        if position.y < top {
            return point(left, top);
        }

        if position.y > bottom {
            return point(right, bottom - px(0.5));
        }

        let x = position.x.max(left).min(right);
        point(x, position.y)
    }

    fn update_drag_selection_at(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let hit_position = self.selection_hit_position(position);
        let index = self.char_offset_for_point(hit_position).min(self.model.value.chars().count());
        self.state.set_cursor(index, true);
        self.state.preferred_column = None;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn update_selection_autoscroll(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.selection_drag_position = Some(position);
        if self.selection_autoscroll_delta_for_point(position) == px(0.0) {
            self.next_selection_scroll_epoch();
            return;
        }

        let epoch = self.next_selection_scroll_epoch();
        self.selection_scroll_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(33)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_selection_autoscroll(epoch, cx));
            }
        });
    }

    fn tick_selection_autoscroll(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if epoch != self.selection_scroll_epoch || !self.model.enabled || !self.mouse_selecting {
            return;
        }

        let Some(position) = self.selection_drag_position else {
            return;
        };

        let delta = self.selection_autoscroll_delta_for_point(position);
        if delta == px(0.0) {
            return;
        }

        if self.scroll_by(delta) {
            self.update_drag_selection_at(position, cx);
        }

        let epoch = self.next_selection_scroll_epoch();
        self.selection_scroll_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(33)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_selection_autoscroll(epoch, cx));
            }
        });
    }

    fn vertical_scroll_to_reveal_cursor(cache: &TextAreaLayoutCache, cursor: usize, scroll: Pixels) -> Pixels {
        let current = Self::clamped_vertical_scroll(cache, scroll);
        let Some((line_ix, _)) = cache
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| cursor >= line.start && cursor <= line.end)
            .or_else(|| cache.lines.iter().enumerate().next_back())
        else {
            return current;
        };

        let line_top = cache.line_height * line_ix as f32;
        let line_bottom = line_top + cache.line_height;
        let viewport_bottom = current + cache.text_viewport.size.height;

        if line_top < current {
            return Self::clamped_vertical_scroll(cache, line_top);
        }

        if line_bottom > viewport_bottom {
            return Self::clamped_vertical_scroll(cache, line_bottom - cache.text_viewport.size.height);
        }

        current
    }

    fn ensure_cursor_visible(&mut self) -> bool {
        let Some(cache) = self.layout_cache.as_ref() else {
            return false;
        };

        let next = Self::vertical_scroll_to_reveal_cursor(cache, self.state.cursor, self.vertical_scroll);
        if next == self.vertical_scroll {
            return false;
        }

        self.vertical_scroll = next;
        true
    }

    fn is_scrollable(&self) -> bool {
        self.layout_cache.as_ref().is_some_and(|cache| Self::max_vertical_scroll_for_cache(cache) > px(0.5))
    }

    fn sync_scrollbar(&mut self, cx: &mut Context<Self>) {
        let Some(cache) = self.layout_cache.as_ref() else {
            return;
        };

        let max_scroll = Self::max_vertical_scroll_for_cache(cache).as_f32().max(0.0);
        let viewport_height = cache.text_viewport.size.height.as_f32().max(1.0);
        let content_height = viewport_height + max_scroll;
        let thumb_fraction = if content_height > 0.0 {
            (viewport_height / content_height).clamp(0.05, 1.0)
        } else {
            1.0
        };
        let enabled = self.model.enabled && max_scroll > 0.5;
        let value = self.vertical_scroll.as_f32().clamp(0.0, max_scroll);
        let step = cache.line_height.as_f32().max(1.0);
        let enabled_changed = self.scrollbar_enabled != enabled;

        self.scrollbar.update(cx, |scrollbar, cx| {
            scrollbar.set_length(viewport_height, cx);
            scrollbar.set_range(ControlRange::new(0.0, max_scroll.max(1.0)), cx);
            scrollbar.set_step(step, cx);
            scrollbar.set_page_step((viewport_height * 0.85).max(step), cx);
            scrollbar.set_thumb_fraction(thumb_fraction, cx);
            scrollbar.set_value(value, cx);
            if enabled_changed {
                scrollbar.set_enabled(enabled, cx);
            }
        });
        self.scrollbar_enabled = enabled;
    }

    fn handle_hover_changed(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.state.set_hovered(*hovered);
        cx.notify();
    }

    fn char_offset_for_point(&self, position: Point<Pixels>) -> usize {
        let Some(cache) = self.layout_cache.as_ref() else {
            return self.model.value.chars().count();
        };

        let local_y = Self::local_axis_position(position.y, cache.text_viewport.top());
        if local_y < px(0.0) {
            return cache.lines.first().map(|line| line.start).unwrap_or(0);
        }
        if local_y > cache.text_viewport.size.height {
            return cache.lines.last().map(|line| line.end).unwrap_or(0);
        }

        let line_ix = Self::line_index_for_local_y(local_y, self.vertical_scroll, cache.line_height, cache.lines.len());
        let Some(line) = cache.lines.get(line_ix) else {
            return 0;
        };
        let local_x = Self::local_axis_position(position.x, cache.text_viewport.left());
        if local_x < px(0.0) {
            return line.start;
        }
        if local_x > cache.text_viewport.size.width {
            return line.end;
        }

        let byte_index = line.line.closest_index_for_x(local_x);
        line.start + Self::byte_to_char_offset_for_line(self.model.value.as_ref(), line.start, line.end, byte_index)
    }

    fn handle_resize_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.resize_dragging = true;
        self.resize_start_y = event.position.y.as_f32();
        self.resize_start_rows = self.model.rows.max(1);
        self.mouse_selecting = false;
        self.stop_selection_autoscroll();
        cx.stop_propagation();
        cx.notify();
    }

    fn handle_resize_drag_move(
        &mut self,
        event: &DragMoveEvent<TextAreaResizeDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.resize_drag_id() || !self.model.enabled || !self.resize_dragging {
            return;
        }

        let delta_rows =
            ((event.event.position.y.as_f32() - self.resize_start_y) / self.resize_line_height()).round() as isize;
        let next_rows = (self.resize_start_rows as isize + delta_rows).clamp(1, 40) as usize;

        if next_rows != self.model.rows {
            self.model.rows = next_rows;
            self.layout_cache = None;
            self.ensure_cursor_visible();
            cx.notify();
        }

        cx.stop_propagation();
    }

    fn handle_resize_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.resize_dragging {
            return;
        }

        self.stop_resize_drag();
        cx.stop_propagation();
        cx.notify();
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.suppress_select_all_on_next_focus = true;
        self.focus_handle.focus(window, cx);
        let chars = self.model.value.chars().collect::<Vec<_>>();
        let len = chars.len();
        let index = self.char_offset_for_point(event.position).min(len);

        if event.click_count >= 3 {
            select_all(&mut self.state, len);
            self.mouse_selecting = false;
            self.stop_selection_autoscroll();
            self.pause_caret_blink(cx);
            cx.notify();
            return;
        }

        if event.click_count == 2 {
            if let Some((start, end)) = word_cluster_range(&chars, index) {
                self.state.selection_anchor = Some(start);
                self.state.cursor = end;
                self.state.preferred_column = None;
                self.mouse_selecting = false;
                self.stop_selection_autoscroll();
                self.pause_caret_blink(cx);
                cx.notify();
            }
            return;
        }

        self.state.set_cursor(index, event.modifiers.shift);
        self.state.preferred_column = None;
        self.mouse_selecting = true;
        self.selection_drag_position = Some(event.position);
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.mouse_selecting {
            return;
        }

        self.update_drag_selection_at(event.position, cx);
        self.update_selection_autoscroll(event.position, cx);
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<TextAreaDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        if event.drag(cx).id != self.model.id || !self.model.enabled {
            return;
        }

        self.mouse_selecting = true;
        self.update_drag_selection_at(event.event.position, cx);
        self.update_selection_autoscroll(event.event.position, cx);
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.resize_dragging {
            self.stop_resize_drag();
            cx.notify();
            return;
        }

        self.mouse_selecting = false;
        self.stop_selection_autoscroll();
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

    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let line_height = self.layout_cache.as_ref().map(|cache| cache.line_height).unwrap_or(px(20.0));
        let delta = event.delta.pixel_delta(line_height).y;
        if self.scroll_by(delta) {
            self.pause_caret_blink(cx);
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        let clipboard_text = if event.keystroke.modifiers.secondary() && event.keystroke.key == "v" {
            cx.read_from_clipboard().and_then(|item| item.text())
        } else {
            None
        };
        let result = handle_key_down(
            &mut self.state,
            self.model.value.as_ref(),
            event,
            clipboard_text.as_deref(),
            EditableTextPolicy {
                multiline: true,
                submit_on_enter: false,
                strip_newlines_on_paste: false,
                allow_tab_character: false,
                clear_on_escape: self.model.clean_on_escape,
            },
        );

        if let Some(text) = result.clipboard_write.clone() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }

        if let Some(navigation) = result.focus_navigation {
            match navigation {
                FocusNavigation::Next => window.focus_next(cx),
                FocusNavigation::Prev => window.focus_prev(cx),
            }
        }

        if result.handled {
            window.prevent_default();
            cx.stop_propagation();
        }

        if result.changed {
            self.model.value = result.value.into();
            self.marked_range = None;
            self.recompute_invalid();
            self.emit_change(cx);
        }

        if result.changed || result.handled {
            self.ensure_cursor_visible();
            self.pause_caret_blink(cx);
            cx.notify();
        }
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl IntoElement for TextAreaElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl gpui::Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let (theme, state, enabled, rows) = {
            let input = self.input.read(cx);
            (input.model.theme.clone(), input.state, input.model.enabled, input.model.rows.max(1))
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(ControlSize::Md, metrics, scale_factor),
        );
        let appearance = theme.resolve_appearance(state, enabled, &scale);
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = px(appearance.typography.line_height * rows as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (theme, state, enabled) = {
            let input = self.input.read(cx);
            (input.model.theme.clone(), input.state, input.model.enabled)
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(ControlSize::Md, metrics, scale_factor),
        );
        let input = self.input.read(cx);
        let appearance = theme.resolve_appearance(state, enabled, &scale);
        let line_height = px(appearance.typography.line_height);
        let font_size = px(appearance.typography.size);
        let mut lines = Vec::new();
        let mut selection_quads = Vec::new();
        let mut caret_quad = None;
        let selection = input.state.selection_range();
        let cursor = input.state.cursor.min(input.model.value.chars().count());
        let show_placeholder = input.model.value.is_empty() && !input.state.focused;
        let font_family = appearance.font_family.clone();
        let font_weight = appearance.typography.weight;
        let run_for = move |len: usize, color| TextRun {
            len,
            font: {
                let mut font = font(font_family.clone());
                font.weight = font_weight;
                font
            },
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let placeholder_line = if show_placeholder {
            let run = run_for(input.model.placeholder.len(), appearance.placeholder);
            Some(window.text_system().shape_line(input.model.placeholder.clone(), font_size, &[run], None))
        } else {
            None
        };

        let logical_lines = TextArea::logical_lines(input.model.value.as_ref());
        let wrap_width = bounds.size.width.max(px(1.0));

        for (hard_start, _hard_end, text) in logical_lines.into_iter() {
            let full_run = run_for(text.len(), appearance.foreground);
            let full_shaped = window.text_system().shape_line(text.clone().into(), font_size, &[full_run], None);
            let line_chars = text.chars().collect::<Vec<_>>();
            let char_count = line_chars.len();

            if char_count == 0 {
                lines.push(TextAreaCachedLine {
                    start: hard_start,
                    end: hard_start,
                    text: String::new(),
                    line: full_shaped,
                });
                continue;
            }

            let mut byte_offsets = Vec::with_capacity(char_count + 1);
            for char_offset in 0..=char_count {
                byte_offsets.push(TextArea::char_to_byte_offset(&text, char_offset));
            }

            let mut local_start = 0usize;
            while local_start < char_count {
                let start_x = full_shaped.x_for_index(byte_offsets[local_start]);
                let mut fit_end = local_start + 1;
                for (probe, _) in byte_offsets.iter().enumerate().take(char_count + 1).skip(local_start + 1) {
                    let probe_x = full_shaped.x_for_index(byte_offsets[probe]);
                    if probe_x - start_x <= wrap_width {
                        fit_end = probe;
                    } else {
                        break;
                    }
                }

                let mut local_end = fit_end;
                if fit_end < char_count {
                    for probe in ((local_start + 1)..=fit_end).rev() {
                        if line_chars[probe - 1].is_whitespace() {
                            local_end = probe;
                            break;
                        }
                    }
                    local_end = local_end.max(local_start + 1);
                }

                let segment_text = line_chars[local_start..local_end].iter().collect::<String>();
                let runs = colored_runs_for_text(
                    &segment_text,
                    hard_start + local_start,
                    selection,
                    &appearance,
                    appearance.font_family.clone(),
                    font_weight,
                );
                let shaped = window.text_system().shape_line(segment_text.clone().into(), font_size, &runs, None);

                lines.push(TextAreaCachedLine {
                    start: hard_start + local_start,
                    end: hard_start + local_end,
                    text: segment_text,
                    line: shaped,
                });

                local_start = local_end;
            }
        }

        let content_height = line_height * lines.len() as f32;
        let max_vertical_scroll = (content_height - bounds.size.height).max(px(0.0));
        let vertical_scroll = input.vertical_scroll.max(px(0.0)).min(max_vertical_scroll);

        for (line_ix, line) in lines.iter().enumerate() {
            let top = bounds.top() + line_height * line_ix as f32 - vertical_scroll;
            let bottom = top + line_height;

            if bottom >= bounds.top() && top <= bounds.bottom() {
                let start = line.start;
                let end = line.end;

                if let Some((selection_start, selection_end)) = selection {
                    let local_start = selection_start.max(start).min(end).saturating_sub(start);
                    let local_end = selection_end.max(start).min(end).saturating_sub(start);
                    if local_start < local_end || (selection_start <= end && selection_end > end && end == start) {
                        let x1 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_start));
                        let x2 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_end));
                        selection_quads.push(fill(
                            Bounds::from_corners(
                                point(bounds.left() + x1, top),
                                point(bounds.left() + x2.max(x1), bottom),
                            ),
                            appearance.selection_background,
                        ));
                    }
                }

                if selection.is_none() && cursor >= start && cursor <= end {
                    let local_cursor = cursor.saturating_sub(start);
                    let x = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_cursor));
                    caret_quad = Some(fill(
                        Bounds::new(point(bounds.left() + x, top), size(px(1.5), line_height)),
                        appearance.caret,
                    ));
                }
            }
        }

        TextAreaPrepaintState { lines, selection_quads, caret_quad, placeholder_line, line_height, vertical_scroll }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(&focus_handle, ElementInputHandler::new(bounds, self.input.clone()), cx);

        for selection in prepaint.selection_quads.drain(..) {
            window.paint_quad(selection);
        }

        if let Some(line) = prepaint.placeholder_line.take() {
            line.paint(bounds.origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
        } else {
            for (line_ix, line) in prepaint.lines.iter().enumerate() {
                let origin = point(
                    bounds.left(),
                    bounds.top() + prepaint.line_height * line_ix as f32 - prepaint.vertical_scroll,
                );
                if origin.y + prepaint.line_height >= bounds.top() && origin.y <= bounds.bottom() {
                    line.line.paint(origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
                }
            }
        }

        let focused = focus_handle.is_focused(window);
        if focused && let Some(caret) = prepaint.caret_quad.take() {
            window.paint_quad(caret);
        }

        self.input.update(cx, |input, cx| {
            input.vertical_scroll = prepaint.vertical_scroll;
            input.layout_cache = Some(TextAreaLayoutCache {
                text_viewport: bounds,
                line_height: prepaint.line_height,
                lines: prepaint.lines.clone(),
            });
            input.clamp_vertical_scroll_to_cache();
            input.sync_scrollbar(cx);
        });
    }
}

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.is_focused(window) {
            window.blur();
        }

        self.sync_focus(window, cx);
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(ControlSize::Md, metrics, scale_factor),
        );
        let appearance = self.model.theme.resolve_appearance(self.state, self.model.enabled, &scale);
        let show_scrollbar = self.is_scrollable();
        let scrollbar_width = px(12.0);
        let resize_handle = div()
            .id(format!("{}-resize-handle", self.model.id))
            .absolute()
            .right(px(3.0))
            .bottom(px(3.0))
            .w(px(12.0))
            .h(px(12.0))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_resize_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_resize_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_resize_mouse_up))
            .on_drag(TextAreaResizeDrag::new(self.resize_drag_id()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(cx.listener(Self::handle_resize_drag_move))
            .child(
                div()
                    .size(px(TEXTAREA_RESIZE_ICON_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family("lucide")
                    .text_size(px(TEXTAREA_RESIZE_ICON_SIZE))
                    .line_height(px(TEXTAREA_RESIZE_ICON_SIZE))
                    .text_color(appearance.border.opacity(0.75))
                    .child(char::from(LucideIcon::Scaling).to_string()),
            );

        let control = div()
            .id(format!("{}-control", self.model.id))
            .relative()
            .flex()
            .items_start()
            .pl(px(appearance.padding_x))
            .pr(px(appearance.padding_x) + if show_scrollbar { scrollbar_width } else { px(0.0) })
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .border(px(appearance.border_width))
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .overflow_hidden()
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_family(appearance.font_family.clone())
            .font_weight(appearance.typography.weight)
            .when(self.model.full_width, |root| root.w_full())
            .when(self.model.enabled, |root| root.cursor_text())
            .when(!self.model.enabled, |root| root.cursor_not_allowed().opacity(0.6))
            .child(TextAreaElement { input: cx.entity() })
            .when(show_scrollbar, |root| {
                root.child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .right(px(2.0))
                        .bottom(px(2.0))
                        .w(scrollbar_width)
                        .flex()
                        .justify_center()
                        .child(self.scrollbar.clone()),
                )
            })
            .when(self.model.enabled, |root| root.child(resize_handle));

        let mut root =
            render_button_family_focus_ring(self.model.id.clone(), control, appearance.focus_ring, appearance.radius);
        if self.model.full_width {
            root = root.w_full();
        }

        root.track_focus(&self.focus_handle)
            .tab_stop(self.model.enabled)
            .on_hover(self.template_handlers(cx).hover)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_click(cx.listener(Self::handle_click))
            .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_drag(TextAreaDrag::new(self.model.id.clone()), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move(cx.listener(Self::handle_drag_move))
    }
}

impl TextArea {
    fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
        text.chars().take(char_offset).map(char::len_utf8).sum()
    }

    fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
        let mut char_offset = 0usize;
        let mut consumed = 0usize;
        for ch in text.chars() {
            if consumed >= byte_offset {
                break;
            }
            consumed += ch.len_utf8();
            char_offset += 1;
        }
        char_offset
    }

    fn byte_to_char_offset_for_line(text: &str, start: usize, end: usize, byte_offset: usize) -> usize {
        let line = text.chars().skip(start).take(end.saturating_sub(start)).collect::<String>();
        Self::byte_to_char_offset(&line, byte_offset)
    }

    fn local_axis_position(position: Pixels, viewport_start: Pixels) -> Pixels {
        position - viewport_start
    }

    fn line_index_for_local_y(
        local_y: Pixels,
        vertical_scroll: Pixels,
        line_height: Pixels,
        line_count: usize,
    ) -> usize {
        if line_count == 0 || line_height <= px(0.0) {
            return 0;
        }

        let scrolled_y = (local_y + vertical_scroll).max(px(0.0));
        let line_ix = (scrolled_y.as_f32() / line_height.as_f32()).floor() as usize;
        line_ix.min(line_count.saturating_sub(1))
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut char_offset = 0usize;
        let mut utf16_count = 0usize;

        for ch in self.model.value.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            char_offset += 1;
        }

        char_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.model.value.chars().take(offset).map(char::len_utf16).sum()
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn selected_range(&self) -> Range<usize> {
        self.state
            .selection_range()
            .map(|(start, end)| start..end)
            .unwrap_or(self.state.cursor..self.state.cursor)
    }

    fn replace_char_range(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        selected_range_utf16: Option<Range<usize>>,
        mark_text: bool,
        cx: &mut Context<Self>,
    ) {
        let mut chars = self.model.value.chars().collect::<Vec<_>>();
        chars.splice(range.clone(), new_text.chars());
        self.model.value = chars.into_iter().collect::<String>().into();

        let inserted_len = new_text.chars().count();
        if let Some(range_utf16) = selected_range_utf16 {
            let mut rel_start = 0usize;
            let mut rel_end = inserted_len;
            let mut utf16_count = 0usize;
            for (ix, ch) in new_text.chars().enumerate() {
                if utf16_count < range_utf16.start {
                    rel_start = ix + 1;
                }
                if utf16_count < range_utf16.end {
                    rel_end = ix + 1;
                }
                utf16_count += ch.len_utf16();
            }
            self.state.selection_anchor = Some(range.start + rel_start);
            self.state.cursor = range.start + rel_end;
        } else {
            self.state.clear_selection();
            self.state.cursor = range.start + inserted_len;
        }

        self.marked_range = if mark_text && inserted_len > 0 {
            Some(range.start..range.start + inserted_len)
        } else {
            None
        };
        self.state.preferred_column = None;
        self.recompute_invalid();
        self.emit_change(cx);
        self.ensure_cursor_visible();
        cx.notify();
    }

    fn range_bounds_from_cache(
        cache: &TextAreaLayoutCache,
        text: &str,
        vertical_scroll: Pixels,
        range: Range<usize>,
    ) -> Option<Bounds<Pixels>> {
        let start = range.start.min(range.end);
        let end = range.end.max(range.start);
        let (line_ix, line) = cache
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| start >= line.start && start <= line.end)
            .or_else(|| cache.lines.iter().enumerate().next())?;
        let line_text = text.chars().skip(line.start).take(line.end.saturating_sub(line.start)).collect::<String>();
        let local_start = start.saturating_sub(line.start);
        let local_end = end.min(line.end).saturating_sub(line.start).max(local_start);
        let start_x = line.line.x_for_index(Self::char_to_byte_offset(&line_text, local_start));
        let end_x = line.line.x_for_index(Self::char_to_byte_offset(&line_text, local_end));
        let top = cache.text_viewport.top() + cache.line_height * line_ix as f32 - vertical_scroll;

        Some(Bounds::from_corners(
            point(cache.text_viewport.left() + start_x, top),
            point(cache.text_viewport.left() + end_x.max(start_x), top + cache.line_height),
        ))
    }
}

impl EntityInputHandler for TextArea {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.model.value.chars().skip(range.start).take(range.end.saturating_sub(range.start)).collect())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection { range: self.range_to_utf16(&self.selected_range()), reversed: false })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        self.replace_char_range(range, text, None, false, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        self.replace_char_range(range, new_text, new_selected_range, true, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        self.layout_cache
            .as_ref()
            .and_then(|cache| {
                Self::range_bounds_from_cache(cache, self.model.value.as_ref(), self.vertical_scroll, range)
            })
            .or(Some(element_bounds))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.char_offset_for_point(point)))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, ShapedLine, point, px, size};

    use super::{TextArea, TextAreaCachedLine, TextAreaLayoutCache};

    fn cache_with_lines(lines: Vec<(usize, usize)>) -> TextAreaLayoutCache {
        TextAreaLayoutCache {
            text_viewport: Bounds::new(point(px(0.0), px(0.0)), size(px(100.0), px(60.0))),
            line_height: px(20.0),
            lines: lines
                .into_iter()
                .map(|(start, end)| TextAreaCachedLine { start, end, text: String::new(), line: ShapedLine::default() })
                .collect(),
        }
    }

    #[test]
    fn local_axis_position_uses_window_coordinates() {
        assert_eq!(TextArea::local_axis_position(px(110.0), px(100.0)), px(10.0));
        assert_eq!(TextArea::local_axis_position(px(180.0), px(100.0)), px(80.0));
    }

    #[test]
    fn local_axis_position_preserves_above_viewport_points() {
        assert_eq!(TextArea::local_axis_position(px(90.0), px(100.0)), px(-10.0));
    }

    #[test]
    fn line_index_for_local_y_maps_each_visible_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(0.0), px(20.0), 4), 0);
        assert_eq!(TextArea::line_index_for_local_y(px(19.0), px(0.0), px(20.0), 4), 0);
        assert_eq!(TextArea::line_index_for_local_y(px(20.0), px(0.0), px(20.0), 4), 1);
        assert_eq!(TextArea::line_index_for_local_y(px(40.0), px(0.0), px(20.0), 4), 2);
    }

    #[test]
    fn line_index_for_local_y_accounts_for_scroll_and_clamps_to_last_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(20.0), px(20.0), 4), 1);
        assert_eq!(TextArea::line_index_for_local_y(px(500.0), px(0.0), px(20.0), 4), 3);
        assert_eq!(TextArea::line_index_for_local_y(px(-50.0), px(0.0), px(20.0), 4), 0);
    }

    #[test]
    fn line_index_for_local_y_keeps_top_edge_as_first_line() {
        assert_eq!(TextArea::line_index_for_local_y(px(0.0), px(0.0), px(20.0), 4), 0);
    }

    #[test]
    fn max_vertical_scroll_uses_content_minus_viewport_height() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::max_vertical_scroll_for_cache(&cache), px(40.0));
    }

    #[test]
    fn clamped_vertical_scroll_stays_inside_scrollable_range() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(-10.0)), px(0.0));
        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(30.0)), px(30.0));
        assert_eq!(TextArea::clamped_vertical_scroll(&cache, px(100.0)), px(40.0));
    }

    #[test]
    fn vertical_scroll_to_reveal_cursor_scrolls_down_to_caret_line() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::vertical_scroll_to_reveal_cursor(&cache, 21, px(0.0)), px(40.0));
    }

    #[test]
    fn vertical_scroll_to_reveal_cursor_scrolls_up_to_caret_line() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::vertical_scroll_to_reveal_cursor(&cache, 2, px(40.0)), px(0.0));
    }

    #[test]
    fn selection_autoscroll_delta_is_zero_inside_viewport() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(30.0))), px(0.0));
    }

    #[test]
    fn selection_autoscroll_delta_follows_pointer_overflow_direction() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(-10.0))), px(-20.0));
        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(70.0))), px(20.0));
    }

    #[test]
    fn selection_autoscroll_delta_accelerates_and_caps_by_distance() {
        let cache = cache_with_lines(vec![
            (0, 4),
            (5, 9),
            (10, 14),
            (15, 19),
            (20, 24),
            (25, 29),
            (30, 34),
            (35, 39),
            (40, 44),
        ]);

        assert_eq!(TextArea::selection_autoscroll_delta_for_cache(&cache, point(px(10.0), px(260.0))), px(120.0));
    }

    #[test]
    fn selection_hit_position_clamps_to_viewport_edges() {
        let cache = cache_with_lines(vec![(0, 4), (5, 9), (10, 14), (15, 19), (20, 24)]);

        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(-20.0), px(-10.0))),
            point(px(0.0), px(0.0))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(120.0), px(90.0))),
            point(px(99.5), px(59.5))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(-20.0), px(30.0))),
            point(px(0.0), px(30.0))
        );
        assert_eq!(
            TextArea::selection_hit_position_for_cache(&cache, point(px(120.0), px(30.0))),
            point(px(99.5), px(30.0))
        );
    }
}
