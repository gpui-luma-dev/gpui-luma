use super::arc::{ColorArcEvent, ColorArcState};
use super::common::{arc_geometry, size_px as component_size_px, thumb_top_left};
use super::visual::default_color_arc_visual;
use crate::controls::color::color_slider::color_thumb::{ColorThumb, ThumbShape};
use crate::controls::color::style::{ElementExt, StyledExt as _};
use gpui::{prelude::*, *};

#[derive(IntoElement)]
pub struct ColorArc {
    state: Entity<ColorArcState>,
}

struct ArcRenderLayout {
    size_px: f32,
    thumb_size: f32,
    value_position: f32,
    thumb_left: f32,
    thumb_top: f32,
}

impl ColorArc {
    pub fn new(state: &Entity<ColorArcState>) -> Self {
        Self { state: state.clone() }
    }

    fn render_layout(state: &ColorArcState) -> ArcRenderLayout {
        let size_px = component_size_px(state.size);
        let thumb_size = state.thumb_size_px();
        let value_position = state.effective_position();
        let thumb_turn = state.position_to_turn(value_position);

        let current_size = if state.bounds.size.width > px(0.0) {
            state.bounds.size
        } else {
            size(px(size_px), px(size_px))
        };

        let local_bounds = Bounds { origin: point(px(0.0), px(0.0)), size: current_size };
        let (thumb_left, thumb_top) = arc_geometry(local_bounds, state.arc_thickness_px())
            .map(|geometry| thumb_top_left(geometry, thumb_turn, thumb_size))
            .unwrap_or((0.0, 0.0));

        ArcRenderLayout { size_px, thumb_size, value_position, thumb_left, thumb_top }
    }

    fn render_background(state: &ColorArcState, window: &mut Window, cx: &App) -> Div {
        state.delegate.style_background(state, div().absolute().inset_0().overflow_hidden(), window, cx)
    }

    fn render_thumb(layout: &ArcRenderLayout, color: Hsla) -> Div {
        div()
            .absolute()
            .left(px(layout.thumb_left))
            .top(px(layout.thumb_top))
            .child(ColorThumb::new(px(layout.thumb_size)).shape(ThumbShape::Circle).active(false).color(color))
    }

    fn render_disabled_overlay(overlay_color: Hsla) -> Div {
        div().absolute().inset_0().bg(overlay_color)
    }

    fn handle_keyboard(state: &mut ColorArcState, event: &KeyDownEvent, cx: &mut Context<ColorArcState>) {
        match event.keystroke.key.as_str() {
            "home" => {
                state.set_value(state.range.start, cx);
                cx.emit(ColorArcEvent::Change(state.range.start));
                cx.emit(ColorArcEvent::Release(state.range.start));
                cx.stop_propagation();
            }
            "end" => {
                state.set_value(state.range.end, cx);
                cx.emit(ColorArcEvent::Change(state.range.end));
                cx.emit(ColorArcEvent::Release(state.range.end));
                cx.stop_propagation();
            }
            "left" | "up" | "right" | "down" => {
                let Some(next_value) = state.keyboard_step(event.keystroke.key.as_str(), event.keystroke.modifiers)
                else {
                    return;
                };

                let clamped_value =
                    next_value.clamp(state.range.start.min(state.range.end), state.range.end.max(state.range.start));
                let emitted = state.apply_keyboard_value(clamped_value, cx);

                cx.emit(ColorArcEvent::Change(emitted));
                cx.emit(ColorArcEvent::Release(emitted));
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn with_keyboard_handlers(root: Stateful<Div>, state_entity: Entity<ColorArcState>) -> Stateful<Div> {
        root.on_key_down(move |event: &KeyDownEvent, _window: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, cx| {
                Self::handle_keyboard(state, event, cx);
            });
        })
    }

    fn with_prepaint_handler(root: Stateful<Div>, state_entity: Entity<ColorArcState>) -> Stateful<Div> {
        root.on_prepaint(move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, _| {
                state.set_bounds(bounds);
            })
        })
    }

    fn interaction_canvas(state_entity: Entity<ColorArcState>) -> impl IntoElement {
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, _, window, _| {
                window.on_mouse_event({
                    let state_entity = state_entity.clone();
                    move |ev: &MouseMoveEvent, phase, window, cx| {
                        if !phase.bubble() {
                            return;
                        }
                        state_entity.update(cx, |state, cx| {
                            if !state.is_drag_interaction_enabled() {
                                return;
                            }
                            state.update_from_mouse(ev.position, window, cx);
                        });
                    }
                });

                window.on_mouse_event({
                    let state_entity = state_entity.clone();
                    move |_: &MouseUpEvent, phase, _, cx| {
                        if !phase.bubble() {
                            return;
                        }
                        state_entity.update(cx, |state, cx| {
                            if state.is_drag_interaction_enabled() {
                                state.set_drag_interaction_enabled(false);
                                state.emit_release(cx);
                                cx.notify();
                            }
                        });
                    }
                });
            },
        )
        .absolute()
        .inset_0()
    }

    fn with_pointer_handlers(
        root: Stateful<Div>,
        state_entity: Entity<ColorArcState>,
        window: &mut Window,
    ) -> Stateful<Div> {
        root.child(Self::interaction_canvas(state_entity.clone()))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(
                    &state_entity,
                    |state: &mut ColorArcState,
                     ev: &MouseDownEvent,
                     window: &mut Window,
                     cx: &mut Context<ColorArcState>| {
                        if !state.begin_drag_from_pointer(ev.position, window, cx) {
                            return;
                        }
                        cx.stop_propagation();
                    },
                ),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(
                    &state_entity,
                    |state: &mut ColorArcState, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<ColorArcState>| {
                        if state.is_drag_interaction_enabled() {
                            state.set_drag_interaction_enabled(false);
                            state.emit_release(cx);
                            cx.notify();
                        }
                    },
                ),
            )
    }
}

impl RenderOnce for ColorArc {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state_entity = self.state.clone();
        let state = state_entity.read(cx);
        let layout = Self::render_layout(state);
        let thumb_color = state.delegate.get_color_at_position(state, layout.value_position);
        let look_visual = default_color_arc_visual(!state.disabled);
        let background = Self::render_background(state, window, cx);

        let root = div()
            .id(state.id.clone())
            .size(px(layout.size_px))
            .relative()
            .refine_style(&state.style)
            .child(background)
            .when(!state.disabled, |this| this.child(Self::render_thumb(&layout, thumb_color)))
            .when(state.disabled, |this| this.child(Self::render_disabled_overlay(look_visual.disabled_overlay)))
            .track_focus(&state.focus_handle);
        let root = Self::with_keyboard_handlers(root, state_entity.clone());
        let root = Self::with_prepaint_handler(root, state_entity.clone());

        if state.disabled {
            root
        } else {
            Self::with_pointer_handlers(root, state_entity, window)
        }
    }
}
