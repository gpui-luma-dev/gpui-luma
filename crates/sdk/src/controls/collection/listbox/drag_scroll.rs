//! Basic linear auto-scrolling shared by host-composed ListBox drag targets.
//! Speed is proportional to pointer depth into the edge zone; displacement is
//! speed times elapsed frame time. There is no temporal easing or inertia.

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{Bounds, Pixels, Point, ScrollHandle, Window, px};

use super::ListBoxAxis;

pub(super) const DEFAULT_MAX_SPEED: f32 = 540.0;

pub(super) struct DragAutoScroll {
    pub accepted: Cell<bool>,
    pub max_speed: Cell<f32>,
    scheduled: Cell<bool>,
    last_frame: Cell<Option<Instant>>,
}

impl Default for DragAutoScroll {
    fn default() -> Self {
        Self {
            accepted: Cell::new(false),
            max_speed: Cell::new(DEFAULT_MAX_SPEED),
            scheduled: Cell::new(false),
            last_frame: Cell::new(None),
        }
    }
}

impl DragAutoScroll {
    pub fn schedule(self: &Rc<Self>, scroll: ScrollHandle, axis: ListBoxAxis, window: &Window) {
        if self.scheduled.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        window.on_next_frame(move |window, cx| {
            let Some(this) = weak.upgrade() else { return };
            this.scheduled.set(false);
            if !this.accepted.get() || !cx.has_active_drag() || !window.is_window_hovered() {
                this.last_frame.set(None);
                return;
            }
            let now = Instant::now();
            let elapsed =
                this.last_frame.replace(Some(now)).map_or(Duration::from_secs_f32(1.0 / 60.0), |last| now - last);
            let offset = scroll.offset();
            let next = edge_scroll_offset(
                scroll.bounds(),
                offset,
                scroll.max_offset(),
                window.mouse_position(),
                axis,
                elapsed,
                this.max_speed.get(),
            );
            if next == offset {
                this.last_frame.set(None);
                return;
            }
            scroll.set_offset(next);
            // Refresh hit testing and insertion markers even with a stationary pointer.
            window.refresh();
            this.schedule(scroll, axis, window);
        });
    }
}

/// Basic auto-scrolling function. A future configurable speed/easing function
/// can replace the linear response while retaining bounds and lifecycle handling.
fn edge_scroll_offset(
    bounds: Bounds<Pixels>,
    mut offset: Point<Pixels>,
    maximum: Point<Pixels>,
    pointer: Point<Pixels>,
    axis: ListBoxAxis,
    elapsed: Duration,
    max_speed: f32,
) -> Point<Pixels> {
    if !bounds.contains(&pointer) {
        return offset;
    }
    let (position, extent, limit, value) = match axis {
        ListBoxAxis::Vertical => (pointer.y - bounds.top(), bounds.size.height, maximum.y, &mut offset.y),
        ListBoxAxis::Horizontal => (pointer.x - bounds.left(), bounds.size.width, maximum.x, &mut offset.x),
    };
    let delta = -crate::infra::drag_drop::edge_scroll_delta(f32::from(position), f32::from(extent), elapsed, max_speed);
    *value = px((f32::from(*value) + delta).clamp(-f32::from(limit).max(0.0), 0.0));
    offset
}

#[cfg(test)]
mod tests {
    use gpui::{point, size};
    use super::*;

    fn step(pointer: Point<Pixels>, offset: Point<Pixels>, axis: ListBoxAxis, elapsed: Duration) -> Point<Pixels> {
        edge_scroll_offset(
            Bounds::new(point(px(10.0), px(20.0)), size(px(200.0), px(200.0))),
            offset,
            point(px(400.0), px(400.0)),
            pointer,
            axis,
            elapsed,
            DEFAULT_MAX_SPEED,
        )
    }

    #[test]
    fn only_the_hovered_viewport_edges_scroll_and_only_on_the_chosen_axis() {
        let offset = point(px(-100.0), px(-100.0));
        let frame = Duration::from_millis(20);
        for pointer in [point(px(100.0), px(100.0)), point(px(9.0), px(21.0)), point(px(211.0), px(219.0))] {
            assert_eq!(step(pointer, offset, ListBoxAxis::Vertical, frame), offset);
        }
        let up = step(point(px(100.0), px(21.0)), offset, ListBoxAxis::Vertical, frame);
        let down = step(point(px(100.0), px(219.0)), offset, ListBoxAxis::Vertical, frame);
        assert!(up.y > offset.y && down.y < offset.y);
        assert_eq!(up.x, offset.x);
        let left = step(point(px(11.0), px(100.0)), offset, ListBoxAxis::Horizontal, frame);
        let right = step(point(px(209.0), px(100.0)), offset, ListBoxAxis::Horizontal, frame);
        assert!(left.x > offset.x && right.x < offset.x);
        assert_eq!(right.y, offset.y);
    }

    #[test]
    fn stationary_pointer_reaches_both_limits_without_overscroll() {
        let mut offset = point(px(0.0), px(0.0));
        for _ in 0..100 {
            offset = step(point(px(100.0), px(219.0)), offset, ListBoxAxis::Vertical, Duration::from_millis(20));
        }
        assert_eq!(offset.y, px(-400.0));
        for _ in 0..100 {
            offset = step(point(px(100.0), px(21.0)), offset, ListBoxAxis::Vertical, Duration::from_millis(20));
        }
        assert_eq!(offset.y, px(0.0));
    }

    #[test]
    fn speed_depends_on_edge_distance_and_elapsed_time_with_a_stall_cap() {
        let offset = point(px(0.0), px(-100.0));
        let pointer = point(px(100.0), px(218.0));
        let fast = step(pointer, offset, ListBoxAxis::Vertical, Duration::from_millis(20));
        let slow = step(point(px(100.0), px(200.0)), offset, ListBoxAxis::Vertical, Duration::from_millis(20));
        assert!(fast.y < slow.y);
        let half = step(pointer, offset, ListBoxAxis::Vertical, Duration::from_millis(10));
        assert_eq!(step(pointer, half, ListBoxAxis::Vertical, Duration::from_millis(10)), fast);
        assert_eq!(
            step(pointer, offset, ListBoxAxis::Vertical, Duration::from_secs(1)),
            step(pointer, offset, ListBoxAxis::Vertical, Duration::from_millis(50))
        );
    }

    #[test]
    fn empty_or_short_content_does_not_scroll() {
        let origin = point(px(0.0), px(0.0));
        for height in [0.0, 20.0, 200.0] {
            assert_eq!(
                edge_scroll_offset(
                    Bounds::new(origin, size(px(200.0), px(height))),
                    origin,
                    origin,
                    point(px(10.0), px(0.0)),
                    ListBoxAxis::Vertical,
                    Duration::from_millis(20),
                    DEFAULT_MAX_SPEED,
                ),
                origin
            );
        }
    }

    #[test]
    fn configured_speed_scales_movement_and_zero_disables_it() {
        let offset = point(px(0.0), px(-100.0));
        let step = |speed| {
            edge_scroll_offset(
                Bounds::new(point(px(10.0), px(20.0)), size(px(200.0), px(200.0))),
                offset,
                point(px(400.0), px(400.0)),
                point(px(100.0), px(218.0)),
                ListBoxAxis::Vertical,
                Duration::from_millis(20),
                speed,
            )
        };
        assert_eq!(step(0.0), offset);
        assert_eq!(offset.y - step(DEFAULT_MAX_SPEED * 2.0).y, (offset.y - step(DEFAULT_MAX_SPEED).y) * 2.0);
    }
}
