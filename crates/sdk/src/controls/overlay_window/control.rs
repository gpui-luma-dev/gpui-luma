use gpui::{
    anchored, deferred, point, Anchor, App, Context, DragMoveEvent, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Render, SharedString, Size, Subscription,
    Window, div, prelude::*, px,
};

use super::{
    DialogBuilder, DialogDismissPolicy, DialogEvent, DialogMode, DialogPosition, DialogRenderModel,
    DialogTemplateHandlers, DialogTemplateParts, default_dialog_theme,
};
use super::model::ThemeInvalidator;
use crate::theme::{ControlSize, observe_theme_revision};

#[derive(Clone)]
struct DialogHeaderDrag {
    id: SharedString,
}

impl Render for DialogHeaderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub struct DialogControl {
    model: super::DialogModel,
    focus_handle: FocusHandle,
    restore_focus: Option<FocusHandle>,
    pending_focus: bool,
    pending_focus_restore: bool,
    dragging: Option<DialogDragState>,
    theme_children: Vec<ThemeInvalidator>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct DialogDragState {
    pointer_origin: Point<Pixels>,
    dialog_origin: Point<Pixels>,
}

type DialogMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
type DialogMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

impl EventEmitter<DialogEvent> for DialogControl {}

impl DialogControl {
    pub(crate) fn from_builder(builder: DialogBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |this, cx| {
            this.invalidate_theme_children(cx);
            cx.notify();
        })
        .detach();

        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            model: builder.model,
            restore_focus: None,
            pending_focus: false,
            pending_focus_restore: false,
            dragging: None,
            theme_children: builder.theme_children,
            _subscriptions: Vec::new(),
        }
    }

    fn invalidate_theme_children(&self, cx: &mut Context<Self>) {
        for child in &self.theme_children {
            child.invalidate(cx);
        }
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.open_from(None, cx);
    }

    pub fn open_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        if let Some(opener) = opener {
            self.restore_focus = Some(opener);
        }
        if self.model.open {
            return;
        }
        self.model.open = true;
        self.pending_focus = true;
        self.pending_focus_restore = false;
        cx.emit(DialogEvent::Opened);
        cx.notify();
    }

    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.model.open {
            return;
        }
        self.model.open = false;
        self.pending_focus = false;
        self.pending_focus_restore = self.restore_focus.is_some();
        self.dragging = None;
        cx.emit(DialogEvent::Dismissed);
        cx.notify();
    }

    pub fn set_mode(&mut self, mode: DialogMode, cx: &mut Context<Self>) {
        self.model.mode = mode;
        cx.notify();
    }

    pub fn set_position(&mut self, position: DialogPosition, cx: &mut Context<Self>) {
        self.model.position = position;
        cx.notify();
    }

    pub fn set_dismiss_policy(&mut self, dismiss_policy: DialogDismissPolicy, cx: &mut Context<Self>) {
        self.model.dismiss_policy = dismiss_policy;
        cx.notify();
    }

    pub fn set_dismissible(&mut self, dismissible: bool, cx: &mut Context<Self>) {
        self.model.dismissible = dismissible;
        cx.notify();
    }

    pub fn set_draggable(&mut self, draggable: bool, cx: &mut Context<Self>) {
        self.model.draggable = draggable;
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.model.open
    }

    fn render_model<'a>(&'a self, window: &Window) -> DialogRenderModel<'a> {
        DialogRenderModel {
            id: &self.model.id,
            mode: self.model.mode,
            position: self.model.position,
            dismissible: self.model.dismissible,
            draggable: self.model.draggable,
            size: self.model.size,
            width: self.model.width,
            focused: self.focus_handle.is_focused(window),
            content: &self.model.content,
        }
    }

    fn has_internal_focus(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    fn handle_shell_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        cx.stop_propagation();
        if self.model.dismiss_policy == DialogDismissPolicy::CloseOnFocusLoss {
            self.pending_focus = false;
        }
    }

    fn handle_shell_mouse_down_out(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.mode == DialogMode::Modeless
            && self.model.dismissible
            && self.model.dismiss_policy == DialogDismissPolicy::CloseOnClickAway
        {
            self.dismiss(cx);
        }
    }

    fn handle_backdrop_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.dismissible {
            window.prevent_default();
            cx.stop_propagation();
            self.dismiss(cx);
        }
    }

    fn handle_header_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        if !self.model.draggable {
            return;
        }

        let viewport = window.viewport_size();
        let origin = resolve_dialog_origin(self.model.position, viewport, self.model.size, self.model.width);
        self.dragging = Some(DialogDragState { pointer_origin: event.position, dialog_origin: origin });
    }

    fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<DialogHeaderDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.model.id {
            return;
        }
        let Some(dragging) = self.dragging else {
            return;
        };
        let delta = event.event.position - dragging.pointer_origin;
        let origin = point(dragging.dialog_origin.x + delta.x, dragging.dialog_origin.y + delta.y);
        self.model.position = DialogPosition::Absolute(clamp_dialog_origin(
            origin,
            _window.viewport_size(),
            self.model.size,
            self.model.width,
        ));
        cx.notify();
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.dragging = None;
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() == "escape" && self.model.dismissible {
            window.prevent_default();
            cx.stop_propagation();
            self.dismiss(cx);
        }
    }

    fn sync_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_focus && self.model.open {
            self.pending_focus = false;
            let focus = self.focus_handle.clone();
            cx.on_next_frame(window, move |_this, window, cx| {
                focus.focus(window, cx);
            });
        }

        if self.pending_focus_restore && !self.model.open {
            self.pending_focus_restore = false;
            if let Some(focus) = self.restore_focus.clone() {
                cx.on_next_frame(window, move |_this, window, cx| {
                    focus.focus(window, cx);
                });
            }
        }
    }
}

impl Focusable for DialogControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DialogControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_focus(window, cx);

        if self.model.open
            && self.model.mode == DialogMode::Modeless
            && self.model.dismiss_policy == DialogDismissPolicy::CloseOnFocusLoss
            && !self.pending_focus
            && !self.has_internal_focus(window)
        {
            self.dismiss(cx);
        }

        if !self.model.open {
            return div().into_any_element();
        }

        let look = default_dialog_theme().resolve(self.model.size, self.model.mode);
        let model = self.render_model(window);
        let header_drag_handle = self.model.draggable.then(|| {
            div()
                .id(format!("{}-drag-header", self.model.id))
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(56.0))
                .occlude()
                .cursor_grab()
                .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_header_mouse_down))
                .on_drag(DialogHeaderDrag { id: self.model.id.clone() }, |drag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                })
                .into_any_element()
        });

        let shell = self.model.template.render(
            &model,
            &look,
            DialogTemplateParts {
                header_drag_handle,
                handlers: DialogTemplateHandlers {
                    key_down: Box::new(cx.listener(Self::handle_key_down)),
                    header_mouse_down: Box::new(cx.listener(Self::handle_header_mouse_down)),
                    shell_mouse_down: Box::new(cx.listener(Self::handle_shell_mouse_down)),
                    shell_mouse_down_out: Box::new(cx.listener(Self::handle_shell_mouse_down_out)),
                },
            },
            window,
            cx,
        );

        let shell = shell
            .track_focus(&self.focus_handle)
            .on_drag_move(cx.listener(Self::handle_drag_move))
            .into_any_element();

        let overlay = match self.model.mode {
            DialogMode::Modal => render_modal_overlay(
                &self.model.id,
                shell,
                &look,
                self.model.position,
                window.viewport_size(),
                Box::new(cx.listener(Self::handle_backdrop_mouse_down)),
            ),
            DialogMode::Modeless => render_modeless_overlay(
                &self.model.id,
                shell,
                self.model.position,
                self.model.size,
                self.model.width,
                window.viewport_size(),
                self.dragging.is_some(),
                Box::new(cx.listener(Self::handle_mouse_up)),
                Box::new(cx.listener(Self::handle_mouse_up)),
            ),
        };

        div().child(deferred(overlay).with_priority(10)).into_any_element()
    }
}

fn render_modal_overlay(
    id: &SharedString,
    shell: gpui::AnyElement,
    look: &super::DialogLook,
    position: DialogPosition,
    viewport: Size<Pixels>,
    backdrop_mouse_down: DialogMouseDownHandler,
) -> gpui::AnyElement {
    let content = render_modal_content(position, shell);

    anchored()
        .snap_to_window_with_margin(px(0.0))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), px(0.0)))
        .child(
            div()
                .id(format!("{id}-overlay"))
                .relative()
                .w(viewport.width)
                .h(viewport.height)
                .on_scroll_wheel(|_event, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(look.backdrop_background)
                        .on_mouse_down(MouseButton::Left, backdrop_mouse_down),
                )
                .child(content),
        )
        .into_any_element()
}

fn render_modal_content(position: DialogPosition, shell: gpui::AnyElement) -> gpui::AnyElement {
    match position {
        DialogPosition::Absolute(point) => div()
            .relative()
            .size_full()
            .child(div().absolute().left(point.x).top(point.y).child(shell))
            .into_any_element(),
        DialogPosition::Center => {
            div().size_full().flex().items_center().justify_center().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::Top => {
            div().size_full().flex().items_center().justify_start().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::Bottom => {
            div().size_full().flex().items_center().justify_end().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::Left => {
            div().size_full().flex().items_start().justify_center().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::Right => {
            div().size_full().flex().items_end().justify_center().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::TopLeft => {
            div().size_full().flex().items_start().justify_start().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::TopRight => {
            div().size_full().flex().items_end().justify_start().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::BottomLeft => {
            div().size_full().flex().items_start().justify_end().p(px(24.0)).child(shell).into_any_element()
        }
        DialogPosition::BottomRight => {
            div().size_full().flex().items_end().justify_end().p(px(24.0)).child(shell).into_any_element()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_modeless_overlay(
    id: &SharedString,
    shell: gpui::AnyElement,
    position: DialogPosition,
    size: ControlSize,
    width: Option<f32>,
    viewport: Size<Pixels>,
    dragging: bool,
    mouse_up: DialogMouseUpHandler,
    mouse_up_out: DialogMouseUpHandler,
) -> gpui::AnyElement {
    let origin = resolve_dialog_origin(position, viewport, size, width);
    anchored()
        .snap_to_window_with_margin(px(12.0))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), px(0.0)))
        .child(
            div()
                .id(format!("{id}-modeless"))
                .relative()
                .w(viewport.width)
                .h(viewport.height)
                .when(dragging, |root| {
                    root.child(div().absolute().inset_0().occlude().on_mouse_down(
                        MouseButton::Left,
                        |_event, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        },
                    ))
                })
                .on_mouse_up(MouseButton::Left, move |event, window, cx| mouse_up(event, window, cx))
                .on_mouse_up_out(MouseButton::Left, move |event, window, cx| mouse_up_out(event, window, cx))
                .child(div().absolute().left(origin.x).top(origin.y).child(shell)),
        )
        .into_any_element()
}

fn resolve_dialog_origin(
    position: DialogPosition,
    viewport: Size<Pixels>,
    size: ControlSize,
    width: Option<f32>,
) -> Point<Pixels> {
    let look = default_dialog_theme().resolve(size, DialogMode::Modeless);
    let width = px(match width {
        Some(width) => width.max(look.min_width),
        None => look.min_width.clamp(look.min_width, look.max_width),
    });
    let height = px(look.estimated_height);
    let inset = px(24.0);

    match position {
        DialogPosition::Center => point((viewport.width - width) * 0.5, (viewport.height - height) * 0.5),
        DialogPosition::Top => point((viewport.width - width) * 0.5, inset),
        DialogPosition::Bottom => point((viewport.width - width) * 0.5, viewport.height - height - inset),
        DialogPosition::Left => point(inset, (viewport.height - height) * 0.5),
        DialogPosition::Right => point(viewport.width - width - inset, (viewport.height - height) * 0.5),
        DialogPosition::TopLeft => point(inset, inset),
        DialogPosition::TopRight => point(viewport.width - width - inset, inset),
        DialogPosition::BottomLeft => point(inset, viewport.height - height - inset),
        DialogPosition::BottomRight => point(viewport.width - width - inset, viewport.height - height - inset),
        DialogPosition::Absolute(point) => point,
    }
}

fn clamp_dialog_origin(
    origin: Point<Pixels>,
    viewport: Size<Pixels>,
    size: ControlSize,
    width: Option<f32>,
) -> Point<Pixels> {
    let look = default_dialog_theme().resolve(size, DialogMode::Modeless);
    let width = px(width.unwrap_or(look.min_width).max(look.min_width));
    let height = px(look.estimated_height);
    let inset = px(12.0);
    let max_x = (viewport.width - width - inset).max(inset);
    let max_y = (viewport.height - height - inset).max(inset);
    point(origin.x.clamp(inset, max_x), origin.y.clamp(inset, max_y))
}
