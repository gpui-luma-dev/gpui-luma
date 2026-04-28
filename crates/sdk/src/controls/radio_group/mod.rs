mod control;
mod model;
mod template;

pub use control::{RadioGroupControl, RadioGroupEvent};
pub use model::{RadioGroupBuilder, RadioGroupItem, RadioGroupModel, RadioGroupRenderItem, RadioGroupRenderModel};
pub use template::{
    RadioGroupClickHandler, RadioGroupHoverHandler, RadioGroupMouseDownHandler, RadioGroupMouseUpHandler,
    RadioGroupTemplate, RadioGroupTemplateHandlers, ThemedRadioGroupTemplate, default_radio_group_template,
};

pub use crate::controls::state::{CompositeItemState as RadioGroupItemState, ControlFocusState};

use gpui::{Entity, SharedString};

pub type RadioGroup = Entity<RadioGroupControl>;

pub fn new(id: impl Into<SharedString>) -> RadioGroupBuilder {
    RadioGroupBuilder::new(id)
}
