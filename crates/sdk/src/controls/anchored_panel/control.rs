use gpui::{
    anchored, deferred, point, App, Bounds, Context, Corner, EventEmitter, FocusHandle, FocusOutEvent, Focusable,
    IntoElement, MouseButton, MouseDownEvent, Pixels, Point, Render, SharedString, Size, Subscription, Window, div,
    prelude::*, px,
};

use super::{AnchoredPanelBuilder, AnchoredPanelDismissPolicy, AnchoredPanelPlacement, AnchoredPanelRenderModel};
use crate::controls::color::style::ElementExt;
use crate::focus::EscapeFocus;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AnchoredPanelEvent {
    OpenChanged { open: bool },
    Dismiss,
    FocusChanged { focused: bool },
}

pub struct AnchoredPanel {
    model: super::model::AnchoredPanelModel,
    focus_handle: FocusHandle,
    content_bounds: Option<Bounds<Pixels>>,
    emitted_focused: bool,
    focus_out_subscription: Option<Subscription>,
    pending_focus: bool,
    dismiss_guard: bool,
}

impl EventEmitter<AnchoredPanelEvent> for AnchoredPanel {}

impl AnchoredPanel {
    pub(crate) fn from_builder(builder: AnchoredPanelBuilder, cx: &mut Context<Self>) -> Self {
        Self {
            pending_focus: builder.model.open && builder.model.focus_on_open,
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(true),
            content_bounds: None,
            emitted_focused: false,
            focus_out_subscription: None,
            dismiss_guard: false,
        }
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> AnchoredPanelBuilder {
        AnchoredPanelBuilder::new(id)
    }

    pub fn is_open(&self) -> bool {
        self.model.open
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.set_open(true, cx);
    }

    pub fn open_guarded(&mut self, cx: &mut Context<Self>) {
        self.dismiss_guard = true;
        self.set_open(true, cx);
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.set_open(false, cx);
    }

    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if self.close_internal(cx) {
            cx.emit(AnchoredPanelEvent::Dismiss);
            cx.notify();
        }
    }

    pub fn toggle_guarded(&mut self, cx: &mut Context<Self>) {
        if self.model.open {
            self.dismiss(cx);
        } else {
            self.open_guarded(cx);
        }
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if open {
            self.open_internal(cx);
        } else {
            self.close_internal(cx);
        }
        cx.notify();
    }

    pub fn set_anchor_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        if self.model.anchor_bounds == Some(bounds) {
            return;
        }
        self.model.anchor_bounds = Some(bounds);
        cx.notify();
    }

    pub fn clear_anchor_bounds(&mut self, cx: &mut Context<Self>) {
        if self.model.anchor_bounds.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_placement(&mut self, placement: AnchoredPanelPlacement, cx: &mut Context<Self>) {
        if self.model.placement == placement {
            return;
        }
        self.model.placement = placement;
        cx.notify();
    }

    pub fn set_offset_y(&mut self, offset_y: Pixels, cx: &mut Context<Self>) {
        if self.model.offset_y == offset_y {
            return;
        }
        self.model.offset_y = offset_y;
        cx.notify();
    }

    pub fn set_initial_content_size(&mut self, size: Size<Pixels>, cx: &mut Context<Self>) {
        if self.model.initial_content_size == Some(size) {
            return;
        }
        self.model.initial_content_size = Some(size);
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> AnchoredPanelRenderModel<'a> {
        AnchoredPanelRenderModel {
            id: &self.model.id,
            open: self.model.open,
            anchor_bounds: self.model.anchor_bounds,
            placement: self.model.placement,
            dismiss_policy: self.model.dismiss_policy,
            dismissible: self.model.dismissible,
            focused: self.focus_handle.is_focused(window),
            content_bounds: self.content_bounds,
        }
    }

    fn open_internal(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.model.open;
        self.model.open = true;
        if self.model.focus_on_open {
            self.pending_focus = true;
        }
        if !was_open {
            cx.emit(AnchoredPanelEvent::OpenChanged { open: true });
        }
        !was_open
    }

    fn close_internal(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.model.open;
        self.model.open = false;
        self.pending_focus = false;
        self.dismiss_guard = false;
        if was_open {
            cx.emit(AnchoredPanelEvent::OpenChanged { open: false });
        }
        was_open
    }

    fn can_dismiss_on_click_away(&self) -> bool {
        self.model.dismissible
            && matches!(
                self.model.dismiss_policy,
                AnchoredPanelDismissPolicy::CloseOnClickAway | AnchoredPanelDismissPolicy::CloseOnClickAwayOrFocusLoss
            )
    }

    fn can_dismiss_on_focus_loss(&self) -> bool {
        self.model.dismissible
            && matches!(
                self.model.dismiss_policy,
                AnchoredPanelDismissPolicy::CloseOnFocusLoss | AnchoredPanelDismissPolicy::CloseOnClickAwayOrFocusLoss
            )
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) {
        if self.emitted_focused == focused {
            return;
        }
        self.emitted_focused = focused;
        cx.emit(AnchoredPanelEvent::FocusChanged { focused });
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.emit_focus_changed(false, cx);
        if self.dismiss_guard {
            self.dismiss_guard = false;
            cx.notify();
            return;
        }
        if !window.is_window_active() || !self.can_dismiss_on_focus_loss() {
            cx.notify();
            return;
        }
        self.dismiss(cx);
    }

    fn handle_shell_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        self.emit_focus_changed(true, cx);
        cx.stop_propagation();
    }

    fn handle_mouse_down_out(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.anchor_bounds.is_some_and(|bounds| bounds.contains(&event.position)) {
            return;
        }
        if self.dismiss_guard {
            self.dismiss_guard = false;
            cx.notify();
            return;
        }
        if !window.is_window_active() || !self.can_dismiss_on_click_away() {
            return;
        }
        self.dismiss(cx);
    }

    fn handle_content_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        if bounds.size.width <= px(0.0) || self.content_bounds == Some(bounds) {
            return;
        }
        self.content_bounds = Some(bounds);
        cx.notify();
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.open && self.model.dismissible {
            self.dismiss(cx);
        } else {
            cx.propagate();
        }
    }

    fn sync_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.pending_focus || !self.model.open || self.model.anchor_bounds.is_none() {
            return;
        }
        self.pending_focus = false;
        let focus = self.focus_handle.clone();
        cx.on_next_frame(window, move |this, window, cx| {
            if this.model.open {
                focus.focus(window, cx);
                this.emit_focus_changed(true, cx);
            }
        });
        self.dismiss_guard = false;
    }
}

impl Focusable for AnchoredPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AnchoredPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        self.sync_focus(window, cx);

        let model = self.render_model(window);
        if !model.open {
            return div().into_any_element();
        }

        let Some(anchor_bounds) = model.anchor_bounds else {
            return div().into_any_element();
        };

        let panel = cx.entity();
        let content = (self.model.content)(&model, window, cx);
        let content = div()
            .on_prepaint(move |bounds, _, cx| {
                panel.update(cx, |panel, cx| panel.handle_content_bounds(bounds, cx));
            })
            .child(content)
            .into_any_element();
        let content_size = self.content_bounds.map(|bounds| bounds.size).or(self.model.initial_content_size);
        let placement = resolve_anchored_panel_placement(
            anchor_bounds,
            content_size,
            self.model.placement,
            self.model.offset_y,
            window.viewport_size(),
            self.model.window_margin,
        );
        let overlay = anchored()
            .snap_to_window_with_margin(self.model.window_margin)
            .anchor(placement.anchor)
            .position(placement.position)
            .offset(placement.offset)
            .child(content);

        div()
            .id(self.model.id.clone())
            .track_focus(&self.focus_handle)
            .key_context("LumaFocus")
            .occlude()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_shell_mouse_down))
            .on_mouse_down_out(cx.listener(Self::handle_mouse_down_out))
            .on_action(cx.listener(Self::handle_escape_focus))
            .child(deferred(overlay).with_priority(1))
            .into_any_element()
    }
}

#[derive(Clone, Copy, Debug)]
struct ResolvedAnchoredPanelPlacement {
    anchor: Corner,
    position: Point<Pixels>,
    offset: Point<Pixels>,
}

fn resolve_anchored_panel_placement(
    anchor_bounds: Bounds<Pixels>,
    content_size: Option<Size<Pixels>>,
    placement: AnchoredPanelPlacement,
    offset_y: Pixels,
    viewport_size: Size<Pixels>,
    window_margin: Pixels,
) -> ResolvedAnchoredPanelPlacement {
    let content_height = content_size.map_or(px(0.0), |size| size.height);
    let resolved = match placement {
        AnchoredPanelPlacement::SmartStart => {
            let viewport_bottom = viewport_size.height - window_margin;
            if anchor_bounds.bottom() + offset_y + content_height <= viewport_bottom {
                AnchoredPanelPlacement::BelowStart
            } else {
                AnchoredPanelPlacement::AboveStart
            }
        }
        placement => placement,
    };

    match resolved {
        AnchoredPanelPlacement::BelowStart | AnchoredPanelPlacement::SmartStart => ResolvedAnchoredPanelPlacement {
            anchor: Corner::TopLeft,
            position: point(anchor_bounds.left(), anchor_bounds.bottom()),
            offset: point(px(0.0), offset_y),
        },
        AnchoredPanelPlacement::BelowCenter => ResolvedAnchoredPanelPlacement {
            anchor: Corner::TopLeft,
            position: point(anchor_bounds.center().x, anchor_bounds.bottom()),
            offset: point(content_size.map_or(px(0.0), |size| -(size.width * 0.5)), offset_y),
        },
        AnchoredPanelPlacement::AboveStart => ResolvedAnchoredPanelPlacement {
            anchor: Corner::BottomLeft,
            position: point(anchor_bounds.left(), anchor_bounds.top()),
            offset: point(px(0.0), -offset_y),
        },
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size, Bounds};

    use super::*;

    #[test]
    fn below_center_uses_content_width_for_horizontal_offset() {
        let placement = resolve_anchored_panel_placement(
            Bounds::new(point(px(20.0), px(10.0)), size(px(40.0), px(20.0))),
            Some(size(px(100.0), px(50.0))),
            AnchoredPanelPlacement::BelowCenter,
            px(4.0),
            size(px(500.0), px(500.0)),
            px(8.0),
        );

        assert_eq!(placement.anchor, Corner::TopLeft);
        assert_eq!(placement.position, point(px(40.0), px(30.0)));
        assert_eq!(placement.offset, point(px(-50.0), px(4.0)));
    }

    #[test]
    fn smart_start_flips_above_when_below_does_not_fit() {
        let placement = resolve_anchored_panel_placement(
            Bounds::new(point(px(20.0), px(450.0)), size(px(40.0), px(20.0))),
            Some(size(px(100.0), px(80.0))),
            AnchoredPanelPlacement::SmartStart,
            px(4.0),
            size(px(500.0), px(500.0)),
            px(8.0),
        );

        assert_eq!(placement.anchor, Corner::BottomLeft);
        assert_eq!(placement.position, point(px(20.0), px(450.0)));
        assert_eq!(placement.offset, point(px(0.0), px(-4.0)));
    }
}
