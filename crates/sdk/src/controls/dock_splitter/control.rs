use gpui::{
    Context, EventEmitter, HitboxBehavior, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Render, SharedString, Window, canvas, div, prelude::*,
};

use super::{
    DockSplitterAppearance, DockSplitterBuilder, DockSplitterModel, DockSplitterRenderModel,
    DockSplitterTemplateHandlers, SplitterOrientation,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
pub enum DockSplitterEvent {
    ResizeStart,
    Resize { delta: f32 },
    ResizeEnd,
}

pub struct DockSplitter {
    model: DockSplitterModel,
    hovered: bool,
    dragging: bool,
    last_axis_px: f32,
}

impl EventEmitter<DockSplitterEvent> for DockSplitter {}

impl DockSplitter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>, orientation: SplitterOrientation) -> DockSplitterBuilder {
        DockSplitterBuilder::new(id, orientation)
    }

    pub(crate) fn from_builder(builder: DockSplitterBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { model: builder.model, hovered: false, dragging: false, last_axis_px: 0.0 }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        if !enabled {
            self.hovered = false;
            self.dragging = false;
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::DockSplitterTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::DockSplitterTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        cx.notify();
    }

    fn render_model(&self) -> DockSplitterRenderModel<'_> {
        DockSplitterRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            enabled: self.model.enabled,
            hovered: self.hovered,
            dragging: self.dragging,
        }
    }

    fn appearance(&self) -> DockSplitterAppearance {
        self.model.theme.resolve(self.model.enabled)
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> DockSplitterTemplateHandlers {
        DockSplitterTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
        }
    }

    fn axis_position(&self, position: gpui::Point<gpui::Pixels>) -> f32 {
        match self.model.orientation {
            SplitterOrientation::Vertical => position.x.as_f32(),
            SplitterOrientation::Horizontal => position.y.as_f32(),
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.hovered == *hovered {
            return;
        }
        self.hovered = *hovered;
        cx.notify();
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }

        self.dragging = true;
        self.last_axis_px = self.axis_position(event.position);
        cx.emit(DockSplitterEvent::ResizeStart);
        cx.notify();
    }

    fn handle_window_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.dragging {
            return;
        }

        let axis_px = self.axis_position(event.position);
        let delta = axis_px - self.last_axis_px;
        self.last_axis_px = axis_px;

        if delta.abs() <= f32::EPSILON {
            return;
        }

        if let Some(on_resize) = &self.model.on_resize {
            on_resize(&delta, window, cx);
        }
        cx.emit(DockSplitterEvent::Resize { delta });
    }

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.button != MouseButton::Left || !self.dragging {
            return;
        }

        self.dragging = false;
        cx.emit(DockSplitterEvent::ResizeEnd);
        cx.notify();
    }
}

impl Render for DockSplitter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model();
        let appearance = self.appearance();
        let handlers = self.template_handlers(cx);
        let entity = cx.entity().clone();

        div()
            .relative()
            .child(self.model.template.render(&model, &appearance, handlers, window, cx))
            .child(
                canvas(
                    |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                    move |_, _, window, _cx| {
                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |event: &MouseMoveEvent, phase, window, cx| {
                                if !phase.bubble() {
                                    return;
                                }
                                entity.update(cx, |this, cx| {
                                    this.handle_window_mouse_move(event, window, cx);
                                });
                            }
                        });
                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |event: &MouseUpEvent, phase, window, cx| {
                                if !phase.bubble() {
                                    return;
                                }
                                entity.update(cx, |this, cx| {
                                    this.handle_mouse_up(event, window, cx);
                                });
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            )
            .into_any_element()
    }
}
