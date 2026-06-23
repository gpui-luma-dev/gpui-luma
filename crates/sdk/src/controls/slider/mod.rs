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
pub use control::{SliderControl, SliderDrag, SliderEvent};
pub use input::{SliderInputStrategy, angle_for_percentage, wrap_and_snap, wrap_value};
pub use model::{
    SliderBuilder, SliderModel, SliderOrientation, SliderRenderModel, SliderThumbPolicy, SliderThumbSize,
    SliderThumbRole, SliderThumbValue, ThumbId, TrackPresentation, TrackSegment, TrackSegmentKind,
};
pub use template::{
    SliderBoundsHandler, SliderDragMoveHandler, SliderHoverHandler, SliderMouseDownHandler, SliderMouseUpHandler,
    SliderTemplate, SliderTemplateHandlers, SliderThumbMouseDownHandler, ThemedAngularDialTemplate,
    ThemedCircularRingTemplate, ThemedSliderTemplate, default_angular_dial_template, default_circular_ring_template,
    default_slider_template,
};
pub use theme::{DefaultSliderTheme, SliderLook, SliderTheme, default_slider_theme};

pub use crate::theme::InteractionState as SliderState;

use gpui::{Entity, SharedString};

pub type Slider = Entity<SliderControl>;

pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
    SliderBuilder::new(id)
}
