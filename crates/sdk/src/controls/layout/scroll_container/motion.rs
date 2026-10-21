//! Frame-driven offset animation shared by scroll containers and item adapters.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use web_time::Instant;
use gpui::{App, EntityId, Pixels, Point, ScrollHandle, Window, px};
use crate::motion::{Transition, MotionState, RunningMotion};

type Destination = Rc<dyn Fn(&ScrollHandle) -> Option<Point<Pixels>>>;

#[derive(Default)]
struct State {
    generation: Cell<u64>,
    active: Cell<bool>,
    definition: RefCell<Transition>,
    samples: Cell<[MotionState; 2]>,
}

/// Internal scrolling mechanism. Hosts resolve destinations; this owns timing,
/// interpolation, clamping and cancellation without knowing about item selection.
#[derive(Clone, Default)]
pub(crate) struct ScrollMotion(Rc<State>);

impl ScrollMotion {
    pub fn configure(&self, definition: Transition) {
        *self.0.definition.borrow_mut() = definition;
    }

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
        let samples = if self.is_active() {
            self.0.samples.get()
        } else {
            [MotionState::default(); 2]
        };
        self.cancel();
        self.0.active.set(true);
        Frame {
            state: Rc::downgrade(&self.0),
            generation: self.0.generation.get(),
            scroll,
            destination: Rc::new(destination),
            view: window.current_view(),
            started: cx.background_executor().now(),
            began: cx.background_executor().now(),
            definition: self.0.definition.borrow().clone(),
            channels: None,
            target: None,
            samples,
            last: None,
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
    began: Instant,
    definition: Transition,
    channels: Option<[Box<dyn RunningMotion>; 2]>,
    target: Option<Point<Pixels>>,
    samples: [MotionState; 2],
    last: Option<Point<Pixels>>,
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
            let current = self.scroll.offset();
            let now = cx.background_executor().now();
            // Rebase on native layout/anchor corrections and changing estimates.
            // Each trajectory is in pixel units and retains its sampled velocity.
            if self.target != Some(target) || self.last.is_some_and(|last| last != current) {
                self.samples[0].value = current.x.as_f32();
                self.samples[1].value = current.y.as_f32();
                self.channels = Some(if self.channels.is_some() {
                    self.started = now;
                    let elapsed = now.duration_since(self.began);
                    [
                        self.definition.retarget(self.samples[0], target.x.as_f32(), elapsed),
                        self.definition.retarget(self.samples[1], target.y.as_f32(), elapsed),
                    ]
                } else {
                    [
                        self.definition.start(self.samples[0], target.x.as_f32()),
                        self.definition.start(self.samples[1], target.y.as_f32()),
                    ]
                });
                self.target = Some(target);
            }
            let elapsed = now.duration_since(self.started);
            let Some(channels) = &self.channels else {
                state.active.set(false);
                return;
            };
            let x = channels[0].sample(elapsed);
            let y = channels[1].sample(elapsed);
            self.samples = [x.state, y.state];
            state.samples.set(self.samples);
            if !x.state.value.is_finite()
                || !y.state.value.is_finite()
                || !x.state.velocity.is_finite()
                || !y.state.velocity.is_finite()
            {
                self.scroll.set_offset(target);
                state.active.set(false);
                return;
            }
            let next = gpui::point(
                if x.complete {
                    target.x
                } else {
                    px(x.state.value.clamp(-max.x.as_f32().max(0.0), 0.0))
                },
                if y.complete {
                    target.y
                } else {
                    px(y.state.value.clamp(-max.y.as_f32().max(0.0), 0.0))
                },
            );
            self.scroll.set_offset(next);
            self.last = Some(next);
            let distance = (next.x - current.x).abs() + (next.y - current.y).abs();
            self.settled = if x.complete && y.complete && distance <= px(0.1) {
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
