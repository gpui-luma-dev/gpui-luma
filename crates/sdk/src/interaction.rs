//! Independent desktop input policies. These policies do not change appearance.
use gpui::{App, FocusHandle, Pixels, ScrollHandle, ScrollWheelEvent, Window};

/// Wheel eligibility, independent of keyboard ownership and boundary propagation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WheelScrollPolicy {
    /// Scroll under the pointer without acquiring keyboard focus.
    #[default]
    Pointer,
    /// Scroll only when the configured focus scope owns actual focus.
    RequireFocus,
    /// Leave wheel input to ancestors; positioning and scrollbar input still work.
    PassThrough,
}
impl WheelScrollPolicy {
    pub(crate) fn accepts(self, focused: bool) -> bool {
        match self {
            Self::Pointer => true,
            Self::RequireFocus => focused,
            Self::PassThrough => false,
        }
    }
}
impl From<bool> for WheelScrollPolicy {
    fn from(required: bool) -> Self {
        if required { Self::RequireFocus } else { Self::Pointer }
    }
}

/// Propagation of applicable wheel input after eligibility is established.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollBoundaryPolicy {
    /// Consume applicable input even at a boundary or with no overflow.
    #[default]
    Contain,
    /// Pass only wholly unhandled events. Partial movement consumes the whole event.
    Chain,
}
impl ScrollBoundaryPolicy {
    pub(crate) fn consumes(self, moved: bool) -> bool {
        moved || self == Self::Contain
    }
}

/// Which real focus handles qualify for focus-required wheel input.
/// This never grants ancestor controls keyboard navigation rights.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WheelFocusScope {
    Owner,
    #[default]
    OwnerAndScrollbar,
    /// Explicitly include embedded editors/buttons in the owner's focus subtree.
    Descendants,
}
impl WheelFocusScope {
    pub(crate) fn focused(
        self,
        owner: &FocusHandle,
        scrollbar: Option<&FocusHandle>,
        window: &Window,
        cx: &App,
    ) -> bool {
        match self {
            Self::Owner => owner.is_focused(window),
            Self::OwnerAndScrollbar => owner.is_focused(window) || scrollbar.is_some_and(|bar| bar.is_focused(window)),
            Self::Descendants => owner.contains_focused(window, cx),
        }
    }
}

/// Independent wheel settings. Explicit per-control setters replace one field only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollInteraction {
    pub wheel: WheelScrollPolicy,
    pub boundary: ScrollBoundaryPolicy,
    pub focus_scope: WheelFocusScope,
}
impl ScrollInteraction {
    /// Collection/popup default: pointer scrolling with containment.
    pub const VIEWPORT: Self = Self {
        wheel: WheelScrollPolicy::Pointer,
        boundary: ScrollBoundaryPolicy::Contain,
        focus_scope: WheelFocusScope::OwnerAndScrollbar,
    };
    /// Document/editor default: pointer scrolling with whole-event chaining.
    pub const DOCUMENT: Self = Self { boundary: ScrollBoundaryPolicy::Chain, ..Self::VIEWPORT };
}

/// Focus acquisition for pointer operations, independent of wheel eligibility.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PointerFocusPolicy {
    #[default]
    Focus,
    /// Perform the pointer operation while leaving keyboard focus with its owner.
    Preserve,
}
impl PointerFocusPolicy {
    pub(crate) fn apply(self, focus: &FocusHandle, window: &mut Window, cx: &mut App) {
        if self == Self::Focus {
            focus.focus(window, cx);
        }
    }
}

/// Use only the supported axis. Invalid, zero and unsupported-axis input passes through.
pub(crate) fn wheel_delta(event: &ScrollWheelEvent, line_height: Pixels, horizontal: bool) -> Option<Pixels> {
    let delta = event.delta.pixel_delta(line_height);
    if !delta.x.as_f32().is_finite() || !delta.y.as_f32().is_finite() {
        return None;
    }
    let value = if horizontal { delta.x } else { delta.y };
    (value.as_f32() != 0.0).then_some(value)
}

pub(crate) fn scroll_handle_by(scroll: &ScrollHandle, delta: Pixels, horizontal: bool) -> bool {
    let previous = scroll.offset();
    let mut next = previous;
    let maximum = scroll.max_offset();
    if horizontal {
        next.x = (next.x + delta).clamp(-maximum.x.max(gpui::px(0.0)), gpui::px(0.0));
    } else {
        next.y = (next.y + delta).clamp(-maximum.y.max(gpui::px(0.0)), gpui::px(0.0));
    }
    if previous == next {
        return false;
    }
    scroll.set_offset(next);
    true
}

/// Register before the native List. GPUI 1.21 has no per-list wheel-disable hook:
/// capture records the live offset, bubble restores rejected movement before
/// ancestors run or a frame is painted. Do not attach native ListState scroll
/// callbacks that could expose the transient rejected offset.
pub(crate) fn list_wheel_routing(
    state: gpui::ListState,
    accepts: impl Fn(&Window, &App) -> bool + 'static,
    boundary: ScrollBoundaryPolicy,
    after_scroll: impl Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static,
) -> impl gpui::IntoElement {
    use gpui::{DispatchPhase, HitboxBehavior, TouchPhase, canvas, prelude::*};
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, _| {
            let mut before = None;
            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if !hitbox.should_handle_scroll(window) {
                    return;
                }
                let delta = wheel_delta(event, gpui::px(20.0), false);
                let accepted = accepts(window, cx);
                if phase == DispatchPhase::Capture {
                    before = Some(state.logical_scroll_top());
                    return;
                }
                let Some(previous) = before.take() else {
                    return;
                };
                if !accepted || delta.is_none() {
                    state.scroll_to(previous);
                    let raw = event.delta.pixel_delta(gpui::px(20.0));
                    if accepted
                        && raw.x == gpui::px(0.0)
                        && raw.y == gpui::px(0.0)
                        && event.touch_phase == TouchPhase::Ended
                    {
                        after_scroll(event, window, cx);
                    }
                    return;
                }
                after_scroll(event, window, cx);
                let current = state.logical_scroll_top();
                let moved = current.item_ix != previous.item_ix || current.offset_in_item != previous.offset_in_item;
                if boundary.consumes(moved) {
                    cx.stop_propagation();
                }
            });
        },
    )
    .absolute()
    .size_full()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{ScrollDelta, point, px};

    #[test]
    fn wheel_axes_and_finite_values_are_explicit() {
        for (x, y, horizontal, expected) in [
            (0.0, 0.0, false, None),
            (3.0, 0.0, false, None),
            (0.0, 3.0, true, None),
            (-0.25, 4.5, true, Some(-0.25)),
            (-0.25, 4.5, false, Some(4.5)),
            (f32::NAN, 2.0, false, None),
            (0.0, f32::INFINITY, false, None),
        ] {
            let event = ScrollWheelEvent { delta: ScrollDelta::Pixels(point(px(x), px(y))), ..Default::default() };
            assert_eq!(wheel_delta(&event, px(20.0), horizontal), expected.map(px));
        }
        let event = ScrollWheelEvent { delta: ScrollDelta::Lines(point(0.0, -2.0)), ..Default::default() };
        assert_eq!(wheel_delta(&event, px(20.0), false), Some(px(-40.0)));
    }
}
