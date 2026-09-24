//! Frame-driven offset animation shared by scroll containers and item adapters.
use std::{
    cell::Cell,
    rc::{Rc, Weak},
    time::Instant,
};
use gpui::{App, EntityId, Pixels, Point, ScrollHandle, Window, px};
use crate::motion::{DEFAULT_TRANSITION_DURATION, ease_out_cubic};

type Destination = Rc<dyn Fn(&ScrollHandle) -> Option<Point<Pixels>>>;

#[derive(Default)]
struct State {
    generation: Cell<u64>,
    active: Cell<bool>,
}

/// Internal scrolling mechanism. Hosts resolve destinations; this owns timing,
/// interpolation, clamping and cancellation without knowing about item selection.
#[derive(Clone, Default)]
pub(crate) struct ScrollMotion(Rc<State>);

impl ScrollMotion {
    pub fn cancel(&self) {
        self.0.generation.set(self.0.generation.get().wrapping_add(1));
        self.0.active.set(false);
    }

    pub fn is_active(&self) -> bool {
        self.0.active.get()
    }

    /// Start during layout, once viewport geometry exists. Resolve each frame so
    /// measured/virtualized destinations can change as estimates become bounds.
    pub fn animate(
        &self,
        scroll: ScrollHandle,
        destination: impl Fn(&ScrollHandle) -> Option<Point<Pixels>> + 'static,
        window: &Window,
        cx: &App,
    ) {
        self.cancel();
        self.0.active.set(true);
        Frame {
            state: Rc::downgrade(&self.0),
            generation: self.0.generation.get(),
            scroll,
            destination: Rc::new(destination),
            view: window.current_view(),
            started: cx.background_executor().now(),
            progress: 0.0,
            settled: 0,
        }
        .schedule(window);
    }
}

struct Frame {
    state: Weak<State>,
    generation: u64,
    scroll: ScrollHandle,
    destination: Destination,
    view: EntityId,
    started: Instant,
    progress: f32,
    settled: u8,
}

impl Frame {
    fn schedule(mut self, window: &Window) {
        window.on_next_frame(move |window, cx| {
            let Some(state) = self.state.upgrade() else { return };
            if state.generation.get() != self.generation {
                return;
            }
            if cx.has_active_drag() || !window.is_window_active() {
                state.active.set(false);
                return;
            }
            let Some(target) = (self.destination)(&self.scroll) else {
                state.active.set(false);
                return;
            };
            if !target.x.as_f32().is_finite() || !target.y.as_f32().is_finite() {
                state.active.set(false);
                return;
            }
            let max = self.scroll.max_offset();
            let target = gpui::point(
                px(target.x.as_f32().clamp(-max.x.as_f32().max(0.0), 0.0)),
                px(target.y.as_f32().clamp(-max.y.as_f32().max(0.0), 0.0)),
            );
            let elapsed = cx.background_executor().now().duration_since(self.started);
            let t = (elapsed.as_secs_f32() / DEFAULT_TRANSITION_DURATION.as_secs_f32()).clamp(0.0, 1.0);
            let progress = ease_out_cubic(t);
            let current = self.scroll.offset();
            // Incremental interpolation preserves layout's anchor corrections.
            let next = if progress >= 1.0 {
                target
            } else {
                current + (target - current) * ((progress - self.progress) / (1.0 - self.progress))
            };
            self.scroll.set_offset(next);
            self.progress = progress;
            let distance = (next.x - current.x).abs() + (next.y - current.y).abs();
            self.settled = if t >= 1.0 && distance <= px(0.1) {
                self.settled + 1
            } else {
                0
            };
            // Allow final layout/measurement to correct the requested destination
            // before releasing it. No recurring policy remains after completion.
            if self.settled >= 2 {
                state.active.set(false);
            } else {
                cx.notify(self.view);
                self.schedule(window);
            }
        });
    }
}
