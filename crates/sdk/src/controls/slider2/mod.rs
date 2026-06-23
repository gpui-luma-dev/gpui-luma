mod constraints;
mod control;
mod input;
mod layout;
mod model;
mod segments;
mod template;
mod thumbs;

pub use constraints::{clamp_and_snap_value, normalize_intervals, step_allowed_value};
pub use control::{Slider2Drag, Slider2Event};
pub use input::{Slider2InputStrategy, angle_for_percentage, wrap_and_snap, wrap_value};
pub use model::{
    Slider2Builder, Slider2Model, Slider2Orientation, Slider2RenderModel, Slider2ThumbPolicy, Slider2ThumbSize,
    SliderThumbRole, SliderThumbValue, ThumbId, TrackPresentation, TrackSegment, TrackSegmentKind,
};
pub use template::{
    Slider2BoundsHandler, Slider2DragMoveHandler, Slider2HoverHandler, Slider2MouseDownHandler, Slider2MouseUpHandler,
    Slider2Template, Slider2TemplateHandlers, ThemedAngularDialTemplate, ThemedCircularRingTemplate,
    ThemedSlider2Template, default_angular_dial_template, default_circular_ring_template, default_slider2_template,
};

use gpui::{Entity, SharedString};

use self::control::Slider2Control;

pub type Slider2 = Entity<Slider2Control>;

pub fn new(id: impl Into<SharedString>) -> Slider2Builder {
    Slider2Builder::new(id)
}
