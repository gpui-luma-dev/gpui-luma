use super::common::{
    next_position_for_arrow_key, position_to_theta, ring_geometry, size_px as component_size_px, thumb_top_left,
    value_from_position,
};
use super::ring::{ColorRingEvent, ColorRingState};
use super::visual::default_color_ring_visual;
use crate::controls::color::color_slider::color_thumb::{ColorThumb, ThumbShape};
use crate::controls::color::style::{ElementExt, StyledExt as _};
use gpui::{prelude::*, *};

#[derive(IntoElement)]
pub struct ColorRing {
    state: Entity<ColorRingState>,
}

#[derive(Clone, Copy)]
struct RingLayout {
    side_px: f32,
    ring_thickness: f32,
    thumb_size: f32,
    thumb_left: f32,
    thumb_top: f32,
}

impl ColorRing {
    pub fn new(state: &Entity<ColorRingState>) -> Self {
        Self { state: state.clone() }
    }

    fn compute_layout(state: &ColorRingState) -> RingLayout {
        let side_px = component_size_px(state.size);
        let thumb_size = state.thumb_size_px();
        let ring_thickness = state.ring_thickness_px();
        let value_position = state.effective_position();
        let theta = position_to_theta(value_position, state.rotation_turns());

        let local_bounds = Bounds { origin: point(px(0.0), px(0.0)), size: size(px(side_px), px(side_px)) };
        let (thumb_left, thumb_top) = ring_geometry(local_bounds, ring_thickness)
            .map(|geometry| thumb_top_left(geometry, theta, thumb_size))
            .unwrap_or((0.0, 0.0));

        RingLayout { side_px, ring_thickness, thumb_size, thumb_left, thumb_top }
    }

    fn frame_layer(layout: RingLayout) -> Div {
        div().absolute().left(px(0.0)).top(px(0.0)).size(px(layout.side_px))
    }

    fn border_layer(layout: RingLayout, color: Hsla) -> Div {
        Self::frame_layer(layout).rounded_full().border_1().border_color(color)
    }

    fn inner_border_layer(layout: RingLayout, color: Hsla) -> Div {
        Self::frame_layer(layout)
            .child(div().absolute().inset(px(layout.ring_thickness)).rounded_full().border_1().border_color(color))
    }

    fn thumb_layer(layout: RingLayout, thumb_color: Hsla) -> Div {
        div()
            .absolute()
            .left(px(layout.thumb_left))
            .top(px(layout.thumb_top))
            .child(ColorThumb::new(px(layout.thumb_size)).shape(ThumbShape::Circle).active(false).color(thumb_color))
    }

    fn disabled_layer(layout: RingLayout, overlay_color: Hsla, center_hole_color: Hsla) -> Div {
        Self::frame_layer(layout)
            .rounded_full()
            .bg(overlay_color)
            .child(div().absolute().inset(px(layout.ring_thickness)).rounded_full().bg(center_hole_color))
    }

    fn key_handler(state_entity: Entity<ColorRingState>) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static {
        move |event: &KeyDownEvent, _window: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, cx| match event.keystroke.key.as_str() {
                "home" => {
                    state.set_value(state.range.start, cx);
                    cx.emit(ColorRingEvent::Change(state.range.start));
                    cx.emit(ColorRingEvent::Release(state.range.start));
                    cx.stop_propagation();
                }
                "end" => {
                    state.set_value(state.range.end, cx);
                    cx.emit(ColorRingEvent::Change(state.range.end));
                    cx.emit(ColorRingEvent::Release(state.range.end));
                    cx.stop_propagation();
                }
                "left" | "up" | "right" | "down" => {
                    let current_position = state.effective_position();
                    let Some(next_position) = next_position_for_arrow_key(
                        event.keystroke.key.as_str(),
                        event.keystroke.modifiers,
                        state.range.clone(),
                        state.step,
                        state.reversed,
                        current_position,
                    ) else {
                        return;
                    };

                    let value = value_from_position(next_position, state.range.clone(), state.step, |position| {
                        state.delegate.position_to_value(state, position)
                    });
                    state.apply_keyboard_position(value, next_position, cx);
                    cx.emit(ColorRingEvent::Change(value));
                    cx.emit(ColorRingEvent::Release(value));
                    cx.stop_propagation();
                }
                _ => {}
            });
        }
    }

    fn prepaint_handler(
        state_entity: Entity<ColorRingState>,
    ) -> impl Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static {
        move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, _| {
                state.bounds = bounds;
            })
        }
    }

    fn attach_interaction_surface(
        root: Stateful<Div>,
        state_entity: Entity<ColorRingState>,
        window: &mut Window,
    ) -> Stateful<Div> {
        root.child(
            canvas(|bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal), {
                let state_entity = state_entity.clone();
                move |_, hitbox: Hitbox, window, cx| {
                    let pointer = window.mouse_position();
                    let hovered = hitbox.is_hovered(window);
                    let has_any_drag = cx.has_active_drag();

                    state_entity.update(cx, |state, _| {
                        let external_drag_active = has_any_drag && !state.is_drag_interaction_enabled();
                        state.update_cursor_state(pointer, hovered, external_drag_active, window, &hitbox);
                    });

                    window.on_mouse_event({
                        let state_entity = state_entity.clone();
                        move |ev: &MouseMoveEvent, phase, window, cx| {
                            if !phase.bubble() {
                                return;
                            }

                            state_entity.update(cx, |state, cx| {
                                state.handle_drag_move(ev.position, window, cx);
                            });
                        }
                    });

                    window.on_mouse_event({
                        let state_entity = state_entity.clone();
                        let hitbox = hitbox.clone();
                        move |ev: &MouseUpEvent, phase, window, cx| {
                            if !phase.bubble() {
                                return;
                            }

                            state_entity.update(cx, |state, cx| {
                                state.handle_drag_release(ev.position, hitbox.is_hovered(window), window, &hitbox, cx);
                            });
                        }
                    });
                }
            })
            .absolute()
            .inset_0(),
        )
        .on_mouse_down(
            MouseButton::Left,
            window.listener_for(
                &state_entity,
                |state: &mut ColorRingState,
                 ev: &MouseDownEvent,
                 window: &mut Window,
                 cx: &mut Context<ColorRingState>| {
                    if !state.begin_drag_from_pointer(ev.position, window, cx) {
                        return;
                    }
                    cx.stop_propagation();
                },
            ),
        )
        .on_mouse_move(window.listener_for(
            &state_entity,
            |_: &mut ColorRingState, _: &MouseMoveEvent, _: &mut Window, cx: &mut Context<ColorRingState>| {
                cx.notify();
            },
        ))
    }
}

impl RenderOnce for ColorRing {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state_entity = self.state.clone();
        let state = state_entity.read(cx);
        let control_id = state.id.clone();
        let layout = Self::compute_layout(state);
        let thumb_color = state.delegate.get_color_at_position(state, state.effective_position());
        let style = state.style.clone();
        let disabled = state.disabled;
        let look_visual = default_color_ring_visual(!disabled);
        let ring_inner_border = state.ring_inner_border;
        let ring_outer_border = state.ring_outer_border;
        let border_color = state.ring_border_color.unwrap_or(look_visual.border);
        let disabled_overlay_color = look_visual.disabled_overlay;
        let center_hole_color = look_visual.center_hole;

        let background = state.delegate.style_background(
            state,
            Self::frame_layer(layout).rounded_full().overflow_hidden(),
            window,
            cx,
        );

        let mut root = div()
            .id(control_id)
            .size(px(layout.side_px))
            .flex_shrink_0()
            .relative()
            .rounded_full()
            .refine_style(&style)
            .child(background)
            .when(ring_outer_border, |this| this.child(Self::border_layer(layout, border_color)))
            .when(ring_inner_border, |this| this.child(Self::inner_border_layer(layout, border_color)))
            .when(!disabled, |this| this.child(Self::thumb_layer(layout, thumb_color)))
            .when(disabled, |this| this.child(Self::disabled_layer(layout, disabled_overlay_color, center_hole_color)))
            .track_focus(&state.focus_handle)
            .on_key_down(Self::key_handler(state_entity.clone()))
            .on_prepaint(Self::prepaint_handler(state_entity.clone()));

        if !disabled {
            root = Self::attach_interaction_surface(root, state_entity.clone(), window);
        }

        root
    }
}
