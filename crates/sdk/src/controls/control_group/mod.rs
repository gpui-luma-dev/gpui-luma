mod control;
mod model;
mod template;

use std::sync::Arc;

pub use control::{ControlGroupControl, ControlGroupEvent};
pub use model::{
    ControlGroupBuilder, ControlGroupItem, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupModel,
    ControlGroupRenderModel, ControlGroupStateMode, ControlSelectionMode,
};
pub use template::{
    ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemTemplate, ControlGroupMouseDownHandler,
    ControlGroupMouseUpHandler, ControlGroupTemplate, ControlGroupTemplateHandlers, default_control_group_template,
    make_control_group_item_template,
};

pub use crate::controls::state::{CompositeItemState as ControlGroupItemState, ControlFocusState};

use gpui::{Entity, InteractiveElement, SharedString, div, prelude::*, px};
use template::render_control_group_items;

pub type ControlGroup<T> = Entity<ControlGroupControl<T>>;

pub fn new<T>(id: impl Into<SharedString>) -> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    ControlGroupBuilder::new(id)
}

pub fn vertical_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(|model, handlers, _window, cx| {
        div()
            .id(model.id.clone())
            .flex()
            .flex_col()
            .gap_2()
            .items_start()
            .children(render_control_group_items(model, handlers, cx))
    })
}

pub fn horizontal_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(|model, handlers, _window, cx| {
        div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .gap(px(12.0))
            .children(render_control_group_items(model, handlers, cx))
    })
}
