mod constraints;
mod control;
mod input;
mod layout;
mod model;
mod segments;
mod template;
mod theme;
mod thumbs;

pub use constraints::{clamp_and_snap_value, normalize_intervals, step_allowed_value};
pub use segments::build_track_segments;
pub use control::{Slider2Control, Slider2Drag, Slider2Event};
pub use input::{Slider2InputStrategy, angle_for_percentage, wrap_and_snap, wrap_value};
pub use model::{
    Slider2Builder, Slider2Model, Slider2Orientation, Slider2RenderModel, Slider2ThumbPolicy, Slider2ThumbSize,
    SliderThumbRole, SliderThumbValue, ThumbId, TrackPresentation, TrackSegment, TrackSegmentKind,
};
pub use template::{
    Slider2BoundsHandler, Slider2DragMoveHandler, Slider2HoverHandler, Slider2MouseDownHandler, Slider2MouseUpHandler,
    Slider2Template, Slider2Template as SliderTemplate, Slider2TemplateHandlers, Slider2ThumbMouseDownHandler,
    ThemedAngularDialTemplate, ThemedCircularRingTemplate, ThemedSlider2Template,
    ThemedSlider2Template as ThemedSliderTemplate, default_angular_dial_template, default_circular_ring_template,
    default_slider2_template,
};
pub use theme::{DefaultSliderTheme, SliderLook, SliderTheme, default_slider_theme};

pub use crate::theme::InteractionState as SliderState;

use gpui::{Entity, SharedString};

// Legacy aliases — Slider2 is the canonical engine.
pub type SliderControl = Slider2Control;
pub type SliderDrag = Slider2Drag;
pub type SliderEvent = Slider2Event;
pub type SliderBuilder = Slider2Builder;
pub type SliderModel = Slider2Model;
pub type SliderInputStrategy = Slider2InputStrategy;
pub type SliderOrientation = Slider2Orientation;
pub type SliderRenderModel<'a> = Slider2RenderModel<'a>;
pub type SliderThumbSize = Slider2ThumbSize;
pub type SliderTemplateHandlers = Slider2TemplateHandlers;
pub type SliderBoundsHandler = Slider2BoundsHandler;
pub type SliderHoverHandler = Slider2HoverHandler;
pub type SliderMouseDownHandler = Slider2MouseDownHandler;
pub type SliderMouseUpHandler = Slider2MouseUpHandler;
pub type SliderDragMoveHandler = Slider2DragMoveHandler;
pub type SliderThumbMouseDownHandler = Slider2ThumbMouseDownHandler;

pub type Slider = Entity<SliderControl>;
pub type Slider2 = Slider;

pub fn default_slider_template() -> std::sync::Arc<dyn SliderTemplate> {
    default_slider2_template()
}

pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
    SliderBuilder::new(id)
}
