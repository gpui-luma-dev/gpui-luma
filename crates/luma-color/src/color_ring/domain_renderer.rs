use std::sync::{Arc, RwLock};

use gpui::{Bounds, Hsla, Image, Pixels, Window, px, size};

use luma::controls::slider::{DomainTrackRenderer, SliderOrientation};

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
        self.context.read().expect("color ring context lock").clone()
    }

    pub fn set_context(&self, context: ColorRingTrackContext) {
        *self.context.write().expect("color ring context lock") = context;
    }

    pub fn set_delegate(&self, delegate: Arc<dyn ColorRingTrackDelegate>) {
        *self.delegate.write().expect("color ring delegate lock") = delegate;
    }
}

impl DomainTrackRenderer for ColorRingDomainRenderer {
    fn paint(&self, bounds: Bounds<Pixels>, _orientation: SliderOrientation, reversed: bool, window: &mut Window) {
        {
            let mut context = self.context.write().expect("color ring context lock");
            context.reversed = reversed;
        }
        let context = self.context.read().expect("color ring context lock");
        let delegate = self.delegate.read().expect("color ring delegate lock").clone();
        delegate.paint_domain_track(&context, bounds, window);
    }

    fn get_color_at_position(&self, position: f32) -> Option<Hsla> {
        let context = self.context.read().expect("color ring context lock");
        let delegate = self.delegate.read().expect("color ring delegate lock").clone();
        Some(delegate.get_color_for_context(&context, position))
    }

    fn raster_image(
        &self,
        bounds: Bounds<Pixels>,
        _orientation: SliderOrientation,
        reversed: bool,
    ) -> Option<Arc<Image>> {
        {
            let mut context = self.context.write().expect("color ring context lock");
            context.reversed = reversed;
        }
        let context = self.context.read().expect("color ring context lock");
        let delegate = self.delegate.read().expect("color ring delegate lock").clone();
        let image_size = if bounds.size.width > px(0.0) && bounds.size.height > px(0.0) {
            bounds.size
        } else {
            let side = context.size_px().max(1.0);
            size(px(side), px(side))
        };
        delegate.raster_cached_image(&context, image_size)
    }
}
