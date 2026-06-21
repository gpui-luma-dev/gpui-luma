use super::color_thumb::{ColorThumb, ThumbAxis, ThumbShape, TrackEndcaps};
use super::slider::{Axis, ColorSliderEvent, ColorSliderState, ThumbPosition};
use super::visual::default_color_slider_visual;
use crate::controls::color::style::{ElementExt, StyledExt as _};
use gpui::{prelude::*, *};

#[derive(Clone)]
struct ColorSliderDrag(EntityId);

impl Render for ColorSliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

#[derive(IntoElement)]
pub struct ColorSlider {
    state: Entity<ColorSliderState>,
}

#[derive(Clone, Copy)]
struct SliderLayout {
    is_vertical: bool,
    track_thickness: f32,
    thumb_size: f32,
    track_inset: f32,
    track_hitsize: f32,
    thumb_pos_pct: f32,
    thumb_main_adjust: Pixels,
}

impl ColorSlider {
    pub fn new(state: &Entity<ColorSliderState>) -> Self {
        Self { state: state.clone() }
    }

    fn compute_layout(state: &ColorSliderState) -> SliderLayout {
        let track_thickness = state.track_thickness();
        let thumb_size = state.thumb_size_px();
        let track_inset = state.track_inset();
        let is_vertical = state.dimensions.axis == Axis::Vertical;
        let track_hitsize = track_thickness.max(thumb_size);

        let thumb_pos_pct = super::slider::normalized_value_percent(state.value, state.range.start, state.range.end);
        let thumb_pos_pct = if state.reversed {
            1.0 - thumb_pos_pct
        } else {
            thumb_pos_pct
        };

        let thumb_main_adjust = px(state.thumb_main_axis_size() * thumb_pos_pct);

        SliderLayout {
            is_vertical,
            track_thickness,
            thumb_size,
            track_inset,
            track_hitsize,
            thumb_pos_pct,
            thumb_main_adjust,
        }
    }

    fn resolve_track_radius(state: &ColorSliderState, rem_size: Pixels) -> Corners<Pixels> {
        let corner_radii = state.style.corner_radii.clone();
        let uses_custom_corner_radii = corner_radii.top_left.is_some()
            || corner_radii.top_right.is_some()
            || corner_radii.bottom_left.is_some()
            || corner_radii.bottom_right.is_some();
        let default_radius = if uses_custom_corner_radii {
            px(999.)
        } else {
            match state.thumb.shape.layout_hint().preferred_track_endcaps {
                TrackEndcaps::Rounded => px(999.),
            }
        };

        Corners {
            top_left: corner_radii.top_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius),
            top_right: corner_radii.top_right.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius),
            bottom_left: corner_radii.bottom_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius),
            bottom_right: corner_radii.bottom_right.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius),
        }
    }

    fn track_frame(layout: SliderLayout, radius: Corners<Pixels>, border_color: Hsla) -> Div {
        div()
            .absolute()
            .when(layout.is_vertical, |this| {
                this.w(px(layout.track_thickness))
                    .left(px((layout.track_hitsize - layout.track_thickness) / 2.0))
                    .top(px(layout.track_inset))
                    .bottom(px(layout.track_inset))
            })
            .when(!layout.is_vertical, |this| {
                this.h(px(layout.track_thickness))
                    .top(px((layout.track_hitsize - layout.track_thickness) / 2.0))
                    .left(px(layout.track_inset))
                    .right(px(layout.track_inset))
            })
            .corner_radii(radius)
            .overflow_hidden()
            .border_1()
            .border_color(border_color)
    }

    fn disabled_overlay(layout: SliderLayout, radius: Corners<Pixels>, color: Hsla) -> Div {
        div()
            .absolute()
            .when(layout.is_vertical, |this| {
                this.w(px(layout.track_thickness))
                    .left(px((layout.track_hitsize - layout.track_thickness) / 2.0))
                    .top(px(layout.track_inset))
                    .bottom(px(layout.track_inset))
            })
            .when(!layout.is_vertical, |this| {
                this.h(px(layout.track_thickness))
                    .top(px((layout.track_hitsize - layout.track_thickness) / 2.0))
                    .left(px(layout.track_inset))
                    .right(px(layout.track_inset))
            })
            .corner_radii(radius)
            .bg(color)
    }

    fn thumb_fill_color(state: &ColorSliderState, layout: SliderLayout) -> Option<Hsla> {
        if state.thumb.shape == ThumbShape::Bar {
            return Some(state.delegate.get_color_at_position(state, layout.thumb_pos_pct));
        }

        let thumb_extends_beyond_track = layout.thumb_size > layout.track_thickness;
        match state.effective_thumb_position() {
            ThumbPosition::InsideSlider if !thumb_extends_beyond_track => None,
            _ => Some(state.delegate.get_color_at_position(state, layout.thumb_pos_pct)),
        }
    }

    fn thumb_element(state: &ColorSliderState, layout: SliderLayout, fill_color: Option<Hsla>) -> Div {
        let thumb_axis = if layout.is_vertical {
            ThumbAxis::Vertical
        } else {
            ThumbAxis::Horizontal
        };
        let thumb_cross_offset_outer = px((layout.track_hitsize - layout.thumb_size) / 2.0);

        div()
            .absolute()
            .when(layout.is_vertical, |this| {
                this.left(thumb_cross_offset_outer)
                    .top(relative(layout.thumb_pos_pct))
                    .mt(-layout.thumb_main_adjust)
            })
            .when(!layout.is_vertical, |this| {
                this.top(thumb_cross_offset_outer)
                    .left(relative(layout.thumb_pos_pct))
                    .ml(-layout.thumb_main_adjust)
            })
            .child(
                ColorThumb::new(px(layout.thumb_size))
                    .shape(state.thumb.shape)
                    .axis(thumb_axis)
                    .active(false)
                    .when_some(fill_color, |this, color| this.color(color)),
            )
    }

    fn value_from_key(state: &ColorSliderState, event: &KeyDownEvent) -> Option<f32> {
        let base_step = state.step.unwrap_or((state.range.end - state.range.start).abs() / 100.0);
        let multiplier = if event.keystroke.modifiers.shift {
            10.0
        } else if event.keystroke.modifiers.alt {
            0.1
        } else {
            1.0
        };
        let step = base_step * multiplier;
        let is_horizontal = state.dimensions.axis == Axis::Horizontal;
        let reversed = state.reversed;

        match event.keystroke.key.as_str() {
            "left" if is_horizontal => {
                if reversed {
                    Some(state.value + step)
                } else {
                    Some(state.value - step)
                }
            }
            "right" if is_horizontal => {
                if reversed {
                    Some(state.value - step)
                } else {
                    Some(state.value + step)
                }
            }
            "up" if !is_horizontal => {
                if reversed {
                    Some(state.value + step)
                } else {
                    Some(state.value - step)
                }
            }
            "down" if !is_horizontal => {
                if reversed {
                    Some(state.value - step)
                } else {
                    Some(state.value + step)
                }
            }
            "home" => Some(state.range.start),
            "end" => Some(state.range.end),
            _ => None,
        }
    }

    fn key_handler(state_entity: Entity<ColorSliderState>) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static {
        move |event: &KeyDownEvent, _window: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, cx| {
                if let Some(value) = Self::value_from_key(state, event) {
                    let clamped_value = state.clamp_to_range(value);
                    state.set_value(clamped_value, cx);
                    cx.emit(ColorSliderEvent::Change(clamped_value));
                    cx.emit(ColorSliderEvent::Release(clamped_value));
                    cx.stop_propagation();
                }
            });
        }
    }

    fn prepaint_handler(
        state_entity: Entity<ColorSliderState>,
    ) -> impl Fn(Bounds<Pixels>, &mut Window, &mut App) + 'static {
        move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
            state_entity.update(cx, |state, _| {
                if state.dimensions.bounds != bounds {
                    state.dimensions.bounds = bounds;
                }
            })
        }
    }

    fn attach_pointer_interactions(
        root: Stateful<Div>,
        state_entity: Entity<ColorSliderState>,
        drag_id: EntityId,
        window: &mut Window,
    ) -> Stateful<Div> {
        root.child(
            canvas(|bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal), {
                let state_entity = state_entity.clone();
                move |_, _, window, _cx| {
                    window.on_mouse_event({
                        let state_entity = state_entity.clone();
                        move |_: &MouseUpEvent, phase, _, cx| {
                            if !phase.bubble() {
                                return;
                            }
                            state_entity.update(cx, |state, cx| {
                                state.end_interaction(cx);
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
                |state: &mut ColorSliderState,
                 ev: &MouseDownEvent,
                 window: &mut Window,
                 cx: &mut Context<ColorSliderState>| {
                    state.focus_handle.focus(window, cx);
                    state.begin_interaction();
                    state.update_from_mouse(ev.position, window, cx);
                },
            ),
        )
        .on_mouse_up(
            MouseButton::Left,
            window.listener_for(
                &state_entity,
                |state: &mut ColorSliderState, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<ColorSliderState>| {
                    state.end_interaction(cx);
                },
            ),
        )
        .on_drag(ColorSliderDrag(drag_id), |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_drag_move(window.listener_for(
            &state_entity,
            move |state: &mut ColorSliderState,
                  ev: &DragMoveEvent<ColorSliderDrag>,
                  window: &mut Window,
                  cx: &mut Context<ColorSliderState>| {
                if ev.drag(cx).0 != drag_id {
                    return;
                }
                state.update_from_mouse(ev.event.position, window, cx);
            },
        ))
    }
}

impl RenderOnce for ColorSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state_entity = self.state.clone();
        let entity_id = state_entity.entity_id();
        let state = state_entity.read(cx);
        let control_id = state.id.clone();
        let disabled = state.disabled;
        let visual = default_color_slider_visual(!disabled);

        let track_border_color = visual.border;
        let disabled_overlay_color = visual.disabled_overlay;

        let layout = Self::compute_layout(state);
        let radius = Self::resolve_track_radius(state, window.rem_size());
        let container = Self::track_frame(layout, radius, track_border_color);
        let background_element = state.delegate.style_background(state, container, window, cx);
        let fill_color = Self::thumb_fill_color(state, layout);
        let thumb = Self::thumb_element(state, layout, fill_color);

        let style = state.style.clone();
        let mut root = div()
            .id(control_id)
            .when(layout.is_vertical, |this| this.w(px(layout.track_hitsize)).h_full())
            .when(!layout.is_vertical, |this| this.h(px(layout.track_hitsize)).w_full())
            .relative()
            .refine_style(&style)
            .child(background_element)
            .when(disabled, |this| this.child(Self::disabled_overlay(layout, radius, disabled_overlay_color)))
            .when(!disabled, |this| this.child(thumb))
            .track_focus(&state.focus_handle)
            .on_key_down(Self::key_handler(state_entity.clone()))
            .on_prepaint(Self::prepaint_handler(state_entity.clone()));

        if !disabled {
            root = Self::attach_pointer_interactions(root, state_entity.clone(), entity_id, window);
        }

        root
    }
}
