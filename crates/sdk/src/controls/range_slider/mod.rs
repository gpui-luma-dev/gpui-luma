mod constraints;
mod control;
mod model;
mod template;

pub use constraints::{
    ContinuousConstraintStrategy, IntervalConstraintStrategy, RangeInterval, SliderConstraintStrategy,
    clamp_and_snap_value, normalize_intervals, step_allowed_value,
};
pub use control::{RangeSliderControl, RangeSliderDrag, RangeSliderEvent};
pub use model::{
    RangeSliderBuilder, RangeSliderModel, RangeSliderRenderModel, RangeSliderSegmentKind, RangeSliderTrackSegment,
};
pub use template::{
    RangeSliderBoundsHandler, RangeSliderDragMoveHandler, RangeSliderHoverHandler, RangeSliderMouseDownHandler,
    RangeSliderMouseUpHandler, RangeSliderTemplate, RangeSliderTemplateHandlers, ThemedRangeSliderTemplate,
    default_range_slider_template,
};

pub use crate::controls::slider::{
    DefaultSliderTheme, SliderLook, SliderOrientation, SliderState, SliderTheme, SliderThumbSize, default_slider_theme,
};

use gpui::{Entity, SharedString};

pub type RangeSlider = Entity<RangeSliderControl>;

pub fn new(id: impl Into<SharedString>) -> RangeSliderBuilder {
    RangeSliderBuilder::new(id)
}
