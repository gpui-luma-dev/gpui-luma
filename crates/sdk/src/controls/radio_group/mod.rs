mod themed_template;

use gpui::{Entity, SharedString};

use crate::controls::control_group::{
    self as control_group, ControlGroupBuilder, ControlGroupControl, ControlGroupEvent, ControlGroupItem,
    ControlGroupItemLike, ControlGroupRenderModel, ControlGroupTemplate, ControlGroupTemplateHandlers,
    ControlSelectionMode,
};

pub use themed_template::{
    RadioGroupLayout, radio_group_button_item_element_template, radio_group_buttons_template,
    render_radio_button_option, render_radio_button_rows,
};

// 1. Semantic Type Aliases
// This makes the idea of a "Radio Group" visible in function signatures and state
pub type RadioGroup<T> = Entity<ControlGroupControl<T>>;
pub type RadioGroupEvent = ControlGroupEvent;
pub type RadioGroupBuilder<T> = ControlGroupBuilder<T>;
pub type RadioGroupItem = ControlGroupItem;
pub type RadioGroupRenderModel<'a, T> = ControlGroupRenderModel<'a, T>;
pub type RadioGroupTemplate<T> = ControlGroupTemplate<T>;
pub type RadioGroupTemplateHandlers = ControlGroupTemplateHandlers;
pub type SelectionMode = ControlSelectionMode;
pub use crate::controls::control_group::ControlGroupItemLike as RadioGroupItemLike;

// 2. The Semantic Constructor
// We return a ControlGroupBuilder pre-configured for radio behavior.
pub fn new<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::new(id)
        // A Radio Group inherently requires exactly one item to be selected
        .single_required()
        .template(radio_group_container_template())
}

// Example of how you might wrap a horizontal variant if you wanted
pub fn horizontal<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    new(id).horizontal()
}

pub fn radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::vertical_group_template()
}

pub fn horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::horizontal_group_template()
}

pub fn default_radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_container_template()
}

pub fn default_horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    horizontal_group_container_template()
}
