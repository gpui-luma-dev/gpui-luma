use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{Bounds, Hsla, Pixels};

use crate::motion::{DEFAULT_TRANSITION_DURATION, VisualTransition};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsIndicatorRect {
    pub left: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsIndicatorPaint {
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub color: Hsla,
}

#[derive(Debug)]
struct TabsIndicatorMotionInner {
    transition: VisualTransition,
    from: TabsIndicatorRect,
    to: TabsIndicatorRect,
    list_bounds: Option<Bounds<Pixels>>,
    pending_item_bounds: Option<Bounds<Pixels>>,
    padding_x: f32,
    height: f32,
    color: Option<Hsla>,
    animated: bool,
    has_geometry: bool,
}

impl TabsIndicatorMotionInner {
    fn new(animated: bool) -> Self {
        Self {
            transition: VisualTransition::new(1.0, Self::duration(animated)),
            from: TabsIndicatorRect::default(),
            to: TabsIndicatorRect::default(),
            list_bounds: None,
            pending_item_bounds: None,
            padding_x: 0.0,
            height: 2.0,
            color: None,
            animated,
            has_geometry: false,
        }
    }

    fn duration(animated: bool) -> Duration {
        if animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        }
    }

    fn set_animated(&mut self, animated: bool) {
        if self.animated == animated {
            return;
        }
        self.animated = animated;
        let progress = self.transition.progress();
        self.transition = VisualTransition::new(progress, Self::duration(animated));
        if !animated {
            self.from = self.to;
            self.transition.snap_to(1.0);
        }
    }

    fn set_metrics(&mut self, padding_x: f32, height: f32, color: Option<Hsla>) {
        self.padding_x = padding_x;
        self.height = height;
        self.color = color;
    }

    fn set_list_bounds(&mut self, bounds: Bounds<Pixels>) {
        let changed = self.list_bounds != Some(bounds);
        self.list_bounds = Some(bounds);
        if changed && let Some(item_bounds) = self.pending_item_bounds {
            self.apply_item_bounds(item_bounds, false);
        }
    }

    fn clear_geometry(&mut self) {
        self.list_bounds = None;
        self.pending_item_bounds = None;
        self.has_geometry = false;
        self.from = TabsIndicatorRect::default();
        self.to = TabsIndicatorRect::default();
        self.transition.snap_to(1.0);
    }

    fn apply_item_bounds(&mut self, item_bounds: Bounds<Pixels>, animate: bool) {
        self.pending_item_bounds = Some(item_bounds);
        let Some(list_bounds) = self.list_bounds else {
            return;
        };
        let next = indicator_rect_from_bounds(item_bounds, list_bounds, self.padding_x);
        self.retarget(next, animate);
    }

    fn retarget(&mut self, next: TabsIndicatorRect, animate: bool) {
        if !self.has_geometry || !animate || !self.animated {
            self.from = next;
            self.to = next;
            self.transition.snap_to(1.0);
            self.has_geometry = true;
            return;
        }

        if (self.to.left - next.left).abs() <= f32::EPSILON && (self.to.width - next.width).abs() <= f32::EPSILON {
            return;
        }

        let current = self.display_rect();
        self.from = current;
        self.to = next;
        self.transition = VisualTransition::new(0.0, Self::duration(self.animated));
        self.transition.set_target(1.0);
        self.has_geometry = true;
    }

    fn display_rect(&self) -> TabsIndicatorRect {
        TabsIndicatorRect {
            left: self.transition.interpolate(self.from.left, self.to.left),
            width: self.transition.interpolate(self.from.width, self.to.width),
        }
    }

    fn sync(&mut self) -> bool {
        self.transition.sync()
    }

    fn is_animating(&self) -> bool {
        self.transition.is_animating()
    }

    fn paint(&self) -> Option<TabsIndicatorPaint> {
        let color = self.color?;
        if !self.has_geometry {
            return None;
        }
        let rect = self.display_rect();
        if rect.width <= f32::EPSILON {
            return None;
        }
        Some(TabsIndicatorPaint { left: rect.left, width: rect.width, height: self.height, color })
    }
}

/// Shared indicator motion state between [`super::TabsControl`] and the list template.
#[derive(Clone, Debug)]
pub struct TabsIndicatorMotion {
    inner: Arc<Mutex<TabsIndicatorMotionInner>>,
}

impl TabsIndicatorMotion {
    pub fn new(animated: bool) -> Self {
        Self { inner: Arc::new(Mutex::new(TabsIndicatorMotionInner::new(animated))) }
    }

    pub fn set_animated(&self, animated: bool) {
        self.inner.lock().expect("tabs indicator motion lock").set_animated(animated);
    }

    pub fn set_metrics(&self, padding_x: f32, height: f32, color: Option<Hsla>) {
        self.inner.lock().expect("tabs indicator motion lock").set_metrics(padding_x, height, color);
    }

    pub fn set_list_bounds(&self, bounds: Bounds<Pixels>) {
        self.inner.lock().expect("tabs indicator motion lock").set_list_bounds(bounds);
    }

    pub fn clear_geometry(&self) {
        self.inner.lock().expect("tabs indicator motion lock").clear_geometry();
    }

    pub fn apply_item_bounds(&self, item_bounds: Bounds<Pixels>, animate: bool) {
        self.inner.lock().expect("tabs indicator motion lock").apply_item_bounds(item_bounds, animate);
    }

    pub fn sync(&self) -> bool {
        self.inner.lock().expect("tabs indicator motion lock").sync()
    }

    pub fn is_animating(&self) -> bool {
        self.inner.lock().expect("tabs indicator motion lock").is_animating()
    }

    pub fn schedule_frame<T>(&self, window: &mut gpui::Window, cx: &mut gpui::Context<T>)
    where
        T: 'static,
    {
        let animating = self.is_animating();
        if !animating {
            return;
        }
        cx.on_next_frame(window, |_, _, cx| {
            cx.notify();
        });
    }

    pub fn paint(&self) -> Option<TabsIndicatorPaint> {
        self.inner.lock().expect("tabs indicator motion lock").paint()
    }

    pub fn display_rect_for_test(&self) -> TabsIndicatorRect {
        self.inner.lock().expect("tabs indicator motion lock").display_rect()
    }
}

pub fn indicator_rect_from_bounds(
    item_bounds: Bounds<Pixels>,
    list_bounds: Bounds<Pixels>,
    padding_x: f32,
) -> TabsIndicatorRect {
    TabsIndicatorRect {
        left: (item_bounds.left() - list_bounds.left()).as_f32() + padding_x,
        width: (item_bounds.size.width.as_f32() - padding_x * 2.0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size, Bounds};

    use super::*;

    #[test]
    fn indicator_rect_insets_by_padding() {
        let list = Bounds::new(point(px(10.0), px(20.0)), size(px(400.0), px(40.0)));
        let item = Bounds::new(point(px(30.0), px(20.0)), size(px(80.0), px(40.0)));
        let rect = indicator_rect_from_bounds(item, list, 8.0);
        assert_eq!(rect.left, 28.0);
        assert_eq!(rect.width, 64.0);
    }

    #[test]
    fn first_geometry_snaps_without_animation() {
        let motion = TabsIndicatorMotion::new(true);
        let list = Bounds::new(point(px(0.0), px(0.0)), size(px(400.0), px(40.0)));
        let item = Bounds::new(point(px(20.0), px(0.0)), size(px(60.0), px(40.0)));
        motion.set_list_bounds(list);
        motion.set_metrics(4.0, 2.0, Some(gpui::hsla(0.0, 0.0, 0.0, 1.0)));
        motion.apply_item_bounds(item, true);
        assert!(!motion.is_animating());
        let paint = motion.paint().expect("paint");
        assert_eq!(paint.left, 24.0);
        assert_eq!(paint.width, 52.0);
    }

    #[test]
    fn retarget_animates_between_rects() {
        let motion = TabsIndicatorMotion::new(true);
        let list = Bounds::new(point(px(0.0), px(0.0)), size(px(400.0), px(40.0)));
        motion.set_list_bounds(list);
        motion.set_metrics(0.0, 2.0, Some(gpui::hsla(0.0, 0.0, 0.0, 1.0)));
        motion.apply_item_bounds(Bounds::new(point(px(0.0), px(0.0)), size(px(40.0), px(40.0))), true);
        motion.apply_item_bounds(Bounds::new(point(px(100.0), px(0.0)), size(px(50.0), px(40.0))), true);
        assert!(motion.is_animating());
        let current = motion.display_rect_for_test();
        assert_eq!(current.left, 0.0);
        assert_eq!(current.width, 40.0);
    }
}
