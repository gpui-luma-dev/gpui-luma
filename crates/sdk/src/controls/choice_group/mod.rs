mod control;
mod model;
mod template;
mod theme;

pub use control::ChoiceGroupEvent;
pub use model::{
    ChoiceGroupBuilder, ChoiceGroupContent, ChoiceGroupItem, ChoiceGroupItemButtonRenderModel,
    ChoiceGroupItemButtonTemplate, ChoiceGroupItemContentButtonRenderModel, ChoiceGroupItemContentButtonTemplate,
    ChoiceGroupItemContentModel, ChoiceGroupItemPosition, ChoiceGroupLayout, ChoiceGroupModel, ChoiceGroupRenderItem,
    ChoiceGroupRenderModel, ChoiceGroupSelectionMode, ChoiceGroupStateMode, ChoiceGroupStylePreset,
};
pub use template::{
    ChoiceGroupClickHandler, ChoiceGroupHoverHandler, ChoiceGroupModifier, ChoiceGroupMouseDownHandler,
    ChoiceGroupMouseUpHandler, ChoiceGroupTemplate, ChoiceGroupTemplateHandlers, ModifiedChoiceGroupTemplate,
    ThemedChoiceGroupTemplate, default_choice_group_template, template_with_modifier,
};
pub use theme::{
    DefaultChoiceGroupTheme, CHOICE_GROUP_THEME_USAGE, ChoiceGroupItemAppearance, ChoiceGroupListAppearance,
    ChoiceGroupTheme, default_choice_group_theme,
};

pub use crate::controls::button_family::{ButtonKind as ChoiceGroupKind, ButtonSize as ChoiceGroupSize};
pub use crate::controls::state::{CompositeItemState as ChoiceGroupItemState, ControlFocusState};

use gpui::{Entity, SharedString};

use self::control::ChoiceGroupControl;

pub type ChoiceGroup = Entity<ChoiceGroupControl>;

pub fn new(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id)
}

pub fn single(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id).single()
}

pub fn multiple(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id).multiple()
}

pub fn toolbar_icons(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id).toolbar_icons()
}

pub fn toolbar_icons_multiple(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id).toolbar_icons_multiple()
}

pub fn single_select(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
    ChoiceGroupBuilder::new(id).single_select()
}
