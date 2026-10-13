use gpui::{Context, Pixels, Point, ScrollWheelEvent, Window};

use super::{ScrollingPlot, WINDOW_SECONDS};

impl ScrollingPlot {
    pub(super) fn set_start(&mut self, requested: f64, cx: &mut Context<Self>) {
        // SDK scrollbar owns snapping and range clamping. Its setter is deliberately
        // silent, so update the plot explicitly for wheel/keyboard input.
        let start = self.scrollbar.update(cx, |bar, cx| {
            bar.set_value(requested, cx);
            f64::from(bar.value())
        });
        self.wheel_position = start;
        if self.start != start {
            self.start = start;
            cx.notify();
        }
    }

    pub(super) fn pan(&mut self, seconds: f64, cx: &mut Context<Self>) {
        self.set_start(self.start + seconds, cx);
    }

    pub(super) fn scroll_wheel(&mut self, event: &ScrollWheelEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(width) = self.diagnostics.read(cx).plot_width() else {
            return;
        };
        let delta = event.delta.pixel_delta(window.line_height());
        if let Some(seconds) = wheel_seconds(delta, event.modifiers.shift, width) {
            // Accumulate sub-step motion so slow trackpad gestures survive SDK snapping.
            let requested = self.wheel_position + seconds;
            let target = f64::from(self.scrollbar.read(cx).range().clamp(requested as f32));
            self.set_start(target, cx);
            self.wheel_position = target;
            cx.stop_propagation();
        }
    }
}

/// Horizontal gestures pan in data units; vertical gestures remain page scrolling.
/// Shift-wheel supports mice with a vertical wheel. Match GPUI's content-motion sign.
fn wheel_seconds(delta: Point<Pixels>, shift: bool, width: f32) -> Option<f64> {
    let x = f32::from(delta.x);
    let y = f32::from(delta.y);
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || width <= 0.0 {
        return None;
    }
    let horizontal = if shift && x == 0.0 { y } else { x };
    if horizontal == 0.0 || (!shift && y.abs() > x.abs()) {
        return None;
    }
    Some(-f64::from(horizontal) * WINDOW_SECONDS / f64::from(width))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px};

    #[test]
    fn trackpad_direction_and_distance_follow_viewport_width() {
        assert_eq!(wheel_seconds(point(px(-20.0), px(0.0)), false, 800.0), Some(3.0));
        assert_eq!(wheel_seconds(point(px(20.0), px(0.0)), false, 800.0), Some(-3.0));
        assert_eq!(wheel_seconds(point(px(-20.0), px(0.0)), false, 400.0), Some(6.0));
    }

    #[test]
    fn vertical_gestures_pass_through_and_shift_wheel_pans() {
        assert_eq!(wheel_seconds(point(px(0.0), px(-20.0)), false, 800.0), None);
        assert_eq!(wheel_seconds(point(px(1.0), px(-20.0)), false, 800.0), None);
        assert_eq!(wheel_seconds(point(px(0.0), px(-20.0)), true, 800.0), Some(3.0));
        assert_eq!(wheel_seconds(point(px(f32::NAN), px(0.0)), false, 800.0), None);
        assert_eq!(wheel_seconds(point(px(-20.0), px(0.0)), false, 0.0), None);
    }
}
