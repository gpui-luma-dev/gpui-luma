use std::sync::{Arc, RwLock};

use gpui::{Bounds, Hsla, Image, Pixels, Window};

use luma::controls::slider::{DomainTrackRenderer, SliderOrientation};

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
        self.context.read().expect("color arc context lock").clone()
    }

    pub fn set_context(&self, context: ColorArcTrackContext) {
        *self.context.write().expect("color arc context lock") = context;
    }

    pub fn set_delegate(&self, delegate: Arc<dyn ColorArcDelegate>) {
        *self.delegate.write().expect("color arc delegate lock") = delegate;
    }

    pub fn shared_context(&self) -> Arc<RwLock<ColorArcTrackContext>> {
        Arc::clone(&self.context)
    }
}

impl DomainTrackRenderer for ColorArcDomainRenderer {
    fn paint(&self, bounds: Bounds<Pixels>, _orientation: SliderOrientation, reversed: bool, window: &mut Window) {
        {
            let mut context = self.context.write().expect("color arc context lock");
            context.reversed = reversed;
        }
        let context = self.context.read().expect("color arc context lock");
        let delegate = self.delegate.read().expect("color arc delegate lock").clone();
        delegate.paint_domain_track(&context, bounds, window);
    }

    fn get_color_at_position(&self, position: f32) -> Option<Hsla> {
        let context = self.context.read().expect("color arc context lock");
        let delegate = self.delegate.read().expect("color arc delegate lock").clone();
        Some(delegate.get_color_for_context(&context, position))
    }

    fn raster_image(
        &self,
        bounds: Bounds<Pixels>,
        _orientation: SliderOrientation,
        reversed: bool,
    ) -> Option<Arc<Image>> {
        {
            let mut context = self.context.write().expect("color arc context lock");
            context.reversed = reversed;
        }
        let context = self.context.read().expect("color arc context lock");
        let delegate = self.delegate.read().expect("color arc delegate lock").clone();
        let image_size = if bounds.size.width > gpui::px(0.0) && bounds.size.height > gpui::px(0.0) {
            bounds.size
        } else {
            let side = context.dial_size_px().max(1.0);
            gpui::size(gpui::px(side), gpui::px(side))
        };
        delegate.raster_cached_image(&context, image_size)
    }
}
