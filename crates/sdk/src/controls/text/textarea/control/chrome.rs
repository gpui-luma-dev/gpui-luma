use gpui::{
    Context, DragMoveEvent, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render, SharedString, Window, div,
    prelude::*, px,
};

use super::element::TextAreaElement;
use super::{FOCUS_RING_GAP, TextArea, TextAreaDrag, TextAreaResizeDrag};
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

pub(super) const TEXTAREA_RESIZE_ICON_SIZE: f32 = 10.0;

impl TextArea {
    pub(super) fn resize_drag_id(&self) -> SharedString {
        format!("{}-resize", self.model.id).into()
    }

    pub(super) fn resize_line_height(&self) -> f32 {
        self.layout_cache.as_ref().map(|cache| cache.line_height.as_f32()).unwrap_or(20.0).max(1.0)
    }

    pub(super) fn stop_resize_drag(&mut self) {
        self.resize_dragging = false;
    }

    pub(super) fn is_scrollable(&self) -> bool {
        self.layout_cache.as_ref().is_some_and(|cache| Self::max_vertical_scroll_for_cache(cache) > px(0.5))
    }

    pub(super) fn sync_scrollbar(&mut self, cx: &mut Context<Self>) {
        let Some(cache) = self.layout_cache.as_ref() else {
            return;
        };

        let max_scroll = Self::max_vertical_scroll_for_cache(cache).as_f32().max(0.0);
        let viewport_height = cache.text_viewport.size.height.as_f32().max(1.0);
        let content_height = viewport_height + max_scroll;
        let enabled = self.model.enabled && max_scroll > 0.5;
        let value = self.vertical_scroll.as_f32().clamp(0.0, max_scroll);
        let step = cache.line_height.as_f32().max(1.0);

        self.scrollbar.update(cx, |scrollbar, cx| {
            scrollbar.set_length(viewport_height, cx);
            scrollbar.set_step(step, cx);
            scrollbar.set_page_step((viewport_height * 0.85).max(step), cx);
            scrollbar.set_viewport(0.0, content_height.max(1.0), value, value + viewport_height, cx);
            scrollbar.set_enabled(enabled, cx);
        });
        self.scrollbar_enabled = enabled;
    }

    pub(super) fn handle_resize_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn handle_resize_drag_move(
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

    pub(super) fn handle_resize_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.resize_dragging {
            return;
        }

        self.stop_resize_drag();
        cx.stop_propagation();
        cx.notify();
    }
}

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.is_focused(window) {
            window.blur();
        }

        self.sync_focus(window, cx);
        let scale_factor = window.scale_factor();
        let control_size = self.model.size;
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: control_size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(control_size, metrics, scale_factor),
        );
        let look = self.resolved_look(&scale);
        let focused = self.state.focused && self.state.focus_visible && self.model.enabled;
        let focus_extent = if look.focus_border.is_some() {
            FOCUS_RING_GAP + look.border_width.max(0.0)
        } else {
            0.0
        };
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
                    .text_size(px(TEXTAREA_RESIZE_ICON_SIZE))
                    .line_height(px(TEXTAREA_RESIZE_ICON_SIZE))
                    .text_color(look.border.opacity(0.75))
                    .child(crate::infra::icon::render_icon_source(
                        &self.model.resize_handle_icon,
                        look.border.opacity(0.75),
                        TEXTAREA_RESIZE_ICON_SIZE,
                    )),
            );

        // Keep overflow on an inner clip host so elevation shadows on the chrome are not clipped.
        let mut control = div()
            .id(format!("{}-control-{}", self.model.id, self.theme_epoch))
            .relative()
            .flex()
            .items_start()
            .pl(px(look.padding_x))
            .pr(px(look.padding_x) + if show_scrollbar { scrollbar_width } else { px(0.0) })
            .py(px(look.padding_y))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_family(look.font_family.clone())
            .font_weight(look.typography.weight)
            .when(self.model.full_width, |root| root.w_full())
            .when(self.model.enabled, |root| root.cursor_text())
            .when(!self.model.enabled, |root| root.cursor_not_allowed().opacity(0.6))
            .child(
                div()
                    .relative()
                    .min_w(px(0.0))
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(TextAreaElement { input: cx.entity() }),
            )
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

        if self.model.enabled
            && let Some(shadows) = look.shadow.as_ref().filter(|shadows| !shadows.is_empty())
        {
            control = control.shadow(shadows.clone());
        }

        let adorned = div().id(format!("{}-adorned", self.model.id)).relative().child(control);
        let mut root = div().id(self.model.id.clone()).relative();
        if focus_extent > 0.0 {
            root = root.p(px(focus_extent)).child(adorned);
        } else {
            root = root.child(adorned);
        }
        if self.model.full_width {
            root = root.w_full();
        }

        if focused {
            root = root.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .right(px(0.0))
                    .bottom(px(0.0))
                    .left(px(0.0))
                    .border(px(look.border_width))
                    .border_color(look.focus_border.unwrap_or(look.border))
                    .rounded(px(look.radius + FOCUS_RING_GAP + look.border_width)),
            );
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
