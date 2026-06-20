mod button_item_template;
mod toggle_button_item_template;
mod control;
mod model;
mod template;
mod theme;
mod themed_template;

pub use control::{ControlGroupControl, ControlGroupEvent};
pub use model::{
    ControlGroupBuilder, ControlGroupChromeModel, ControlGroupItem, ControlGroupItemLike, ControlGroupItemRenderModel,
    ControlGroupLayout, ControlGroupModel, ControlGroupRenderModel, ControlGroupStateMode, ControlSelectionMode,
};
pub use button_item_template::button_item_template;
pub use toggle_button_item_template::toggle_button_item_template;
pub use template::{
    ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemTemplate, ControlGroupMouseDownHandler,
    ControlGroupMouseUpHandler, ControlGroupTemplate, ControlGroupTemplateHandlers, default_control_group_template,
    control_group_template_with_theme, make_control_group_item_template, shared_control_group_template,
    template_with_modifier,
};
pub use theme::{DefaultControlGroupTheme, ControlGroupListLook, ControlGroupTheme, default_control_group_theme};
pub use themed_template::ThemedControlGroupTemplate;

pub use crate::controls::state::{CompositeItemState as ControlGroupItemState, ControlFocusState};

use gpui::{Entity, SharedString};

pub type ControlGroup<T> = Entity<ControlGroupControl<T>>;

pub fn new<T>(id: impl Into<SharedString>) -> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    ControlGroupBuilder::new(id)
}

/// Themed template; set layout via [`ControlGroupBuilder::horizontal`] or [`ControlGroupBuilder::vertical`].
pub fn vertical_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    shared_control_group_template()
}

pub fn horizontal_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    shared_control_group_template()
}
