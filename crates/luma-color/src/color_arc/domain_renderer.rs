use std::sync::{Arc, RwLock};

use gpui::{Bounds, Hsla, Image, Pixels, Window};

use luma::controls::slider::{DomainTrackRenderer, SliderOrientation};
use luma::infra::lock;

use super::track_context::ColorArcTrackContext;
use super::types::ColorArcDelegate;

pub struct ColorArcDomainRenderer {
    delegate: Arc<RwLock<Arc<dyn ColorArcDelegate>>>,
    context: Arc<RwLock<ColorArcTrackContext>>,
}

impl ColorArcDomainRenderer {
    pub fn new(delegate: Arc<dyn ColorArcDelegate>, context: ColorArcTrackContext) -> Self {
        Self { delegate: Arc::new(RwLock::new(delegate)), context: Arc::new(RwLock::new(context)) }
    }

    pub fn context(&self) -> ColorArcTrackContext {
        lock::read(&self.context).clone()
    }

    pub fn set_context(&self, context: ColorArcTrackContext) {
        *lock::write(&self.context) = context;
    }

    pub fn set_delegate(&self, delegate: Arc<dyn ColorArcDelegate>) {
        *lock::write(&self.delegate) = delegate;
    }

    pub fn shared_context(&self) -> Arc<RwLock<ColorArcTrackContext>> {
        Arc::clone(&self.context)
    }
}

impl DomainTrackRenderer for ColorArcDomainRenderer {
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
        let image_size = if bounds.size.width > gpui::px(0.0) && bounds.size.height > gpui::px(0.0) {
            bounds.size
        } else {
            let side = context.dial_size_px().max(1.0);
            gpui::size(gpui::px(side), gpui::px(side))
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

    impl ColorArcDelegate for StubDelegate {
        fn paint_domain_track(&self, _context: &ColorArcTrackContext, _bounds: Bounds<Pixels>, _window: &mut Window) {}

        fn get_color_for_context(&self, _context: &ColorArcTrackContext, _position: f32) -> Hsla {
            hsla(0.1, 0.2, 0.3, 1.0)
        }
    }

    #[test]
    fn poisoned_locks_do_not_abort_color_lookup() {
        let renderer = ColorArcDomainRenderer::new(Arc::new(StubDelegate), ColorArcTrackContext::default());
        let expected = renderer.get_color_at_position(0.25);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = renderer.context.write().unwrap();
            panic!("poison");
        }));
        assert_eq!(renderer.get_color_at_position(0.25), expected);
    }
}
