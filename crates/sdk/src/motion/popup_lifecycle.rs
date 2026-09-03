use gpui::{Bounds, Pixels, Point, Window};

use super::overlay_presence::OverlayPresence;

/// Shared open/close state for popup-like controls.
///
/// The trigger remains part of the popup's interactive surface: an outside
/// click only dismisses the popup when it is outside both the popup content and
/// its trigger. The content host owns the actual outside-click boundary; this
/// type supplies the trigger exclusion used by that boundary.
#[derive(Clone, Copy, Debug)]
pub struct PopupLifecycle {
    open: bool,
    presence: OverlayPresence,
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl PopupLifecycle {
    pub fn new(animated: bool) -> Self {
        Self { open: false, presence: OverlayPresence::new(false, animated), trigger_bounds: None }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn presence(&self) -> OverlayPresence {
        self.presence
    }

    pub fn trigger_bounds(&self) -> Option<Bounds<Pixels>> {
        self.trigger_bounds
    }

    pub fn set_trigger_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.trigger_bounds = Some(bounds);
    }

    pub fn open(&mut self) -> bool {
        if self.open {
            return false;
        }
        self.open = true;
        self.presence.set_open(true);
        true
    }

    pub fn close(&mut self) -> bool {
        if !self.open && !self.presence.is_animating() {
            return false;
        }
        self.open = false;
        self.presence.set_open(false);
        true
    }

    pub fn toggle(&mut self) -> bool {
        if self.open { self.close() } else { self.open() }
    }

    /// Dismisses when a pointer is outside the trigger.
    ///
    /// The caller invokes this from the popup content's outside-click handler,
    /// so being outside the content is already established by the event host.
    pub fn dismiss_from_outside_click(&mut self, position: Point<Pixels>) -> bool {
        if self.is_inside_trigger(position) {
            return false;
        }
        self.close()
    }

    pub fn is_inside_trigger(&self, position: Point<Pixels>) -> bool {
        self.trigger_bounds.is_some_and(|bounds| bounds.contains(&position))
    }

    pub fn handle_escape(&mut self) -> bool {
        self.close()
    }

    pub fn sync(&mut self) -> bool {
        self.presence.sync()
    }

    pub fn schedule_frame<T>(&self, window: &mut Window, cx: &mut gpui::Context<T>)
    where
        T: 'static,
    {
        self.presence.schedule_frame(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    #[test]
    fn trigger_click_is_not_an_outside_dismissal() {
        let mut lifecycle = PopupLifecycle::new(false);
        lifecycle.set_trigger_bounds(Bounds { origin: point(px(10.0), px(10.0)), size: size(px(40.0), px(24.0)) });
        lifecycle.open();

        assert!(!lifecycle.dismiss_from_outside_click(point(px(20.0), px(20.0))));
        assert!(lifecycle.is_open());
        assert!(lifecycle.dismiss_from_outside_click(point(px(100.0), px(100.0))));
        assert!(!lifecycle.is_open());
    }

    #[test]
    fn toggle_closes_an_open_popup() {
        let mut lifecycle = PopupLifecycle::new(false);
        assert!(lifecycle.toggle());
        assert!(lifecycle.is_open());
        assert!(lifecycle.toggle());
        assert!(!lifecycle.is_open());
    }
}
