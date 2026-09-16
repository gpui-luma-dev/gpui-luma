use std::sync::{Arc, RwLock};

use gpui::{Bounds, Hsla, Image, Pixels, Window, px, size};

use luma::controls::slider::{DomainTrackRenderer, SliderOrientation};
use luma::infra::lock;

use super::track_context::ColorRingTrackContext;
use super::types::ColorRingTrackDelegate;

pub struct ColorRingDomainRenderer {
    delegate: Arc<RwLock<Arc<dyn ColorRingTrackDelegate>>>,
    context: Arc<RwLock<ColorRingTrackContext>>,
}

impl ColorRingDomainRenderer {
    pub fn new(delegate: Arc<dyn ColorRingTrackDelegate>, context: ColorRingTrackContext) -> Self {
        Self { delegate: Arc::new(RwLock::new(delegate)), context: Arc::new(RwLock::new(context)) }
    }

    pub fn context(&self) -> ColorRingTrackContext {
        lock::read(&self.context).clone()
    }

    pub fn set_context(&self, context: ColorRingTrackContext) {
        *lock::write(&self.context) = context;
    }

    pub fn set_delegate(&self, delegate: Arc<dyn ColorRingTrackDelegate>) {
        *lock::write(&self.delegate) = delegate;
    }
}

impl DomainTrackRenderer for ColorRingDomainRenderer {
    fn paint(&self, bounds: Bounds<Pixels>, _orientation: SliderOrientation, reversed: bool, window: &mut Window) {
        {
            let mut context = lock::write(&self.context);
            context.reversed = reversed;
        }
        let context = lock::read(&self.context);
        let delegate = lock::read(&self.delegate).clone();
        delegate.paint_domain_track(&context, bounds, window);
    }

    fn get_color_at_position(&self, position: f32) -> Option<Hsla> {
        let context = lock::read(&self.context);
        let delegate = lock::read(&self.delegate).clone();
        Some(delegate.get_color_for_context(&context, position))
    }

    fn raster_image(
        &self,
        bounds: Bounds<Pixels>,
        _orientation: SliderOrientation,
        reversed: bool,
    ) -> Option<Arc<Image>> {
        {
            let mut context = lock::write(&self.context);
            context.reversed = reversed;
        }
        let context = lock::read(&self.context);
        let delegate = lock::read(&self.delegate).clone();
        let image_size = if bounds.size.width > px(0.0) && bounds.size.height > px(0.0) {
            bounds.size
        } else {
            let side = context.size_px().max(1.0);
            size(px(side), px(side))
        };
        delegate.raster_cached_image(&context, image_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::hsla;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    struct StubDelegate;

    impl ColorRingTrackDelegate for StubDelegate {
        fn paint_domain_track(&self, _context: &ColorRingTrackContext, _bounds: Bounds<Pixels>, _window: &mut Window) {}

        fn get_color_for_context(&self, _context: &ColorRingTrackContext, _position: f32) -> Hsla {
            hsla(0.5, 0.5, 0.5, 1.0)
        }
    }

    #[test]
    fn poisoned_locks_do_not_abort_color_lookup() {
        let renderer = ColorRingDomainRenderer::new(Arc::new(StubDelegate), ColorRingTrackContext::default());
        let expected = renderer.get_color_at_position(0.75);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = renderer.context.write().unwrap();
            panic!("poison");
        }));
        assert_eq!(renderer.get_color_at_position(0.75), expected);
    }
}
