use std::sync::{Arc, RwLock};

use gpui::{Bounds, Hsla, Pixels, Window};

use crate::controls::slider::{DomainTrackRenderer, SliderOrientation};

use super::types::ColorSliderDelegate;
use super::track_context::{ColorSliderTrackContext, axis_from_orientation};

pub struct ColorSliderDomainRenderer {
    delegate: Arc<RwLock<Arc<dyn ColorSliderDelegate>>>,
    context: Arc<RwLock<ColorSliderTrackContext>>,
}

impl ColorSliderDomainRenderer {
    pub fn new(delegate: Arc<dyn ColorSliderDelegate>, context: ColorSliderTrackContext) -> Self {
        Self { delegate: Arc::new(RwLock::new(delegate)), context: Arc::new(RwLock::new(context)) }
    }

    pub fn context(&self) -> ColorSliderTrackContext {
        self.context.read().expect("color slider context lock").clone()
    }

    pub fn set_context(&self, context: ColorSliderTrackContext) {
        *self.context.write().expect("color slider context lock") = context;
    }

    pub fn set_delegate(&self, delegate: Arc<dyn ColorSliderDelegate>) {
        *self.delegate.write().expect("color slider delegate lock") = delegate;
    }

    pub fn shared_context(&self) -> Arc<RwLock<ColorSliderTrackContext>> {
        Arc::clone(&self.context)
    }
}

impl DomainTrackRenderer for ColorSliderDomainRenderer {
    fn paint(&self, bounds: Bounds<Pixels>, orientation: SliderOrientation, reversed: bool, window: &mut Window) {
        {
            let mut context = self.context.write().expect("color slider context lock");
            context.axis = axis_from_orientation(orientation);
            context.reversed = reversed;
        }
        let context = self.context.read().expect("color slider context lock");
        let delegate = self.delegate.read().expect("color slider delegate lock").clone();
        delegate.paint_domain_track(&context, bounds, window);
    }

    fn get_color_at_position(&self, position: f32) -> Option<Hsla> {
        let context = self.context.read().expect("color slider context lock");
        let delegate = self.delegate.read().expect("color slider delegate lock").clone();
        Some(delegate.get_color_for_context(&context, position))
    }
}
