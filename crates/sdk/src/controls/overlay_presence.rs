use std::time::Duration;

use gpui::{Context, Pixels, Point, Size, Window, point};

use crate::animation::{DEFAULT_TRANSITION_DURATION, VisualTransition};

/// Minimum scale factor for overlay enter/exit (Framer / Shadcn `zoom-in-95`).
pub const OVERLAY_ENTER_SCALE_MIN: f32 = 0.95;

/// Frame-driven open/close presence for overlays (opacity + layout-scale).
///
/// Keeps the host mounted while the close animation runs, then reports
/// [`should_paint`](Self::should_paint) as `false` once settled closed.
#[derive(Clone, Copy, Debug)]
pub struct OverlayPresence {
    transition: VisualTransition,
    logical_open: bool,
    animated: bool,
}

impl Default for OverlayPresence {
    fn default() -> Self {
        Self::new(false, true)
    }
}

impl OverlayPresence {
    pub fn new(open: bool, animated: bool) -> Self {
        let duration = Self::duration(animated);
        let target = if open { 1.0 } else { 0.0 };
        Self { transition: VisualTransition::new(target, duration), logical_open: open, animated }
    }

    pub fn set_animated(&mut self, animated: bool) {
        if self.animated == animated {
            return;
        }
        self.animated = animated;
        let progress = self.transition.progress();
        self.transition = VisualTransition::new(progress, Self::duration(animated));
        if !animated {
            let target = if self.logical_open { 1.0 } else { 0.0 };
            self.transition.snap_to(target);
        }
    }

    pub fn is_animated(&self) -> bool {
        self.animated
    }

    pub fn is_logical_open(&self) -> bool {
        self.logical_open
    }

    pub fn set_open(&mut self, open: bool) {
        self.logical_open = open;
        let target = if open { 1.0 } else { 0.0 };
        if self.animated {
            self.transition.set_target(target);
        } else {
            self.transition.snap_to(target);
        }
    }

    pub fn snap_open(&mut self, open: bool) {
        self.logical_open = open;
        self.transition.snap_to(if open { 1.0 } else { 0.0 });
    }

    pub fn sync(&mut self) -> bool {
        self.transition.sync()
    }

    pub fn schedule_frame<T>(&self, window: &mut Window, cx: &mut Context<T>)
    where
        T: 'static,
    {
        self.transition.schedule_frame(window, cx);
    }

    pub fn progress(&self) -> f32 {
        self.transition.progress()
    }

    pub fn is_animating(&self) -> bool {
        self.transition.is_animating()
    }

    pub fn should_paint(&self) -> bool {
        self.logical_open || self.transition.progress() > f32::EPSILON || self.transition.is_animating()
    }

    pub fn opacity(&self) -> f32 {
        self.progress().clamp(0.0, 1.0)
    }

    pub fn scale(&self) -> f32 {
        overlay_enter_scale(self.progress())
    }

    /// Compensates placement offset so layout-scale grows from the placement origin
    /// (top edge / start), approximating `transform-origin` without CSS scale.
    pub fn adjust_offset(&self, offset: Point<Pixels>, content_size: Size<Pixels>) -> Point<Pixels> {
        overlay_enter_offset(offset, content_size, self.scale())
    }

    pub fn scaled_size(&self, content_size: Size<Pixels>) -> Size<Pixels> {
        let scale = self.scale();
        Size { width: content_size.width * scale, height: content_size.height * scale }
    }

    fn duration(animated: bool) -> Duration {
        if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        }
    }
}

pub fn overlay_enter_scale(progress: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    OVERLAY_ENTER_SCALE_MIN + ((1.0 - OVERLAY_ENTER_SCALE_MIN) * t)
}

pub fn overlay_enter_offset(offset: Point<Pixels>, content_size: Size<Pixels>, scale: f32) -> Point<Pixels> {
    let scale = scale.clamp(OVERLAY_ENTER_SCALE_MIN, 1.0);
    let dx = content_size.width * ((1.0 - scale) * 0.5);
    let dy = content_size.height * ((1.0 - scale) * 0.5);
    point(offset.x + dx, offset.y + dy)
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::*;

    #[test]
    fn closed_endpoints_map_to_min_scale_and_zero_opacity() {
        let presence = OverlayPresence::new(false, true);
        assert!(!presence.should_paint());
        assert_eq!(presence.opacity(), 0.0);
        assert!((presence.scale() - OVERLAY_ENTER_SCALE_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn open_endpoints_map_to_full_scale_and_opacity() {
        let presence = OverlayPresence::new(true, true);
        assert!(presence.should_paint());
        assert_eq!(presence.opacity(), 1.0);
        assert!((presence.scale() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_open_retargets_and_keeps_mounted_while_closing() {
        let mut presence = OverlayPresence::new(true, true);
        presence.set_open(false);
        assert!(!presence.is_logical_open());
        assert!(presence.should_paint());
        assert!(presence.is_animating());
    }

    #[test]
    fn snap_open_settles_without_animation() {
        let mut presence = OverlayPresence::new(false, true);
        presence.snap_open(true);
        assert!(presence.is_logical_open());
        assert!(!presence.is_animating());
        assert_eq!(presence.opacity(), 1.0);
    }

    #[test]
    fn unanimated_set_open_snaps() {
        let mut presence = OverlayPresence::new(false, false);
        presence.set_open(true);
        assert!(!presence.is_animating());
        assert_eq!(presence.opacity(), 1.0);
        assert!((presence.scale() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn adjust_offset_compensates_toward_center_from_origin() {
        let offset = point(px(0.0), px(4.0));
        let size = Size { width: px(100.0), height: px(40.0) };
        let adjusted = overlay_enter_offset(offset, size, OVERLAY_ENTER_SCALE_MIN);
        assert!((adjusted.x.as_f32() - 2.5).abs() < 0.001);
        assert!((adjusted.y.as_f32() - 5.0).abs() < 0.001);
    }
}
