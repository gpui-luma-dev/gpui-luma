mod control;
mod model;
mod template;
mod theme;

pub use control::{SliderDrag, SliderEvent};
pub use model::{SliderBuilder, SliderInputStrategy, SliderModel, SliderOrientation, SliderRenderModel};
pub use template::{
    SliderBoundsHandler, SliderDragMoveHandler, SliderHoverHandler, SliderMouseDownHandler, SliderMouseUpHandler,
    SliderTemplate, SliderTemplateHandlers, ThemedSliderTemplate, default_slider_template,
};

pub use theme::{DefaultSliderTheme, SliderLook, SliderTheme, default_slider_theme};

pub use crate::theme::InteractionState as SliderState;

use gpui::{Entity, SharedString};

use self::control::SliderControl;

pub type Slider = Entity<SliderControl>;

pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
    SliderBuilder::new(id)
}
