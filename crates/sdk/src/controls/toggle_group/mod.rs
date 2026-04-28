mod control;
mod model;
mod template;

pub use control::{ToggleGroupControl, ToggleGroupEvent};
pub use model::{
    ToggleGroupBuilder, ToggleGroupItem, ToggleGroupItemPosition, ToggleGroupModel, ToggleGroupRenderItem,
    ToggleGroupRenderModel, ToggleGroupSelectionMode,
};
pub use template::{
    ThemedToggleGroupTemplate, ToggleGroupClickHandler, ToggleGroupHoverHandler, ToggleGroupMouseDownHandler,
    ToggleGroupMouseUpHandler, ToggleGroupTemplate, ToggleGroupTemplateHandlers, default_toggle_group_template,
};

pub use crate::controls::button_family::{ButtonKind as ToggleGroupKind, ButtonSize as ToggleGroupSize};
pub use crate::controls::state::{CompositeItemState as ToggleGroupItemState, ControlFocusState};

use gpui::{Entity, SharedString};

pub type ToggleGroup = Entity<ToggleGroupControl>;

pub fn new(id: impl Into<SharedString>) -> ToggleGroupBuilder {
    ToggleGroupBuilder::new(id)
}
