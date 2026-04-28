mod control;
mod model;
mod template;

pub use control::{SliderControl, SliderDrag, SliderEvent};
pub use model::{SliderBuilder, SliderModel, SliderRenderModel};
pub use template::{
    SliderBoundsHandler, SliderDragMoveHandler, SliderHoverHandler, SliderMouseDownHandler, SliderMouseUpHandler,
    SliderTemplate, SliderTemplateHandlers, ThemedSliderTemplate, default_slider_template,
};

pub use crate::theme::InteractionState as SliderState;

use gpui::{Entity, SharedString};

pub type Slider = Entity<SliderControl>;

pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
    SliderBuilder::new(id)
}
