use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div,
    prelude::*,
};

use super::model::{ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupRenderModel, ControlSelectionMode};

pub type ControlGroupClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ControlGroupHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ControlGroupMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ControlGroupMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ControlGroupTemplateHandlers {
    pub item_hovers: Vec<ControlGroupHoverHandler>,
    pub item_mouse_downs: Vec<ControlGroupMouseDownHandler>,
    pub item_mouse_ups: Vec<ControlGroupMouseUpHandler>,
    pub item_mouse_up_outs: Vec<ControlGroupMouseUpHandler>,
    pub item_clicks: Vec<ControlGroupClickHandler>,
}

pub type ControlGroupItemTemplate<T> =
    Arc<dyn for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_control_group_item_template<T, F, E>(template: F) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

pub type ControlGroupTemplate<T> = Arc<
    dyn for<'a> Fn(
            &ControlGroupRenderModel<'a, T>,
            ControlGroupTemplateHandlers,
            &mut Window,
            &mut App,
        ) -> Stateful<Div>
        + Send
        + Sync
        + 'static,
>;

pub fn default_control_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(default_control_group_template_impl::<T>)
}

fn default_control_group_template_impl<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    _window: &mut Window,
    cx: &mut App,
) -> Stateful<Div>
where
    T: ControlGroupItemLike + 'static,
{
    div()
        .id(model.id.clone())
        .flex()
        .flex_col()
        .items_start()
        .children(render_control_group_items(model, handlers, cx))
}

fn default_item_content<T>(model: &ControlGroupItemRenderModel<'_, T>) -> AnyElement
where
    T: ControlGroupItemLike + 'static,
{
    let active_marker = if model.state.focus_visible {
        ">"
    } else if model.active {
        "*"
    } else {
        " "
    };

    let selection_marker = match model.selection_mode {
        ControlSelectionMode::SingleRequired | ControlSelectionMode::SingleAllowNone => {
            if model.selected {
                "(*)"
            } else {
                "( )"
            }
        }
        ControlSelectionMode::Multiple => {
            if model.selected {
                "[x]"
            } else {
                "[ ]"
            }
        }
    };

    div().child(format!("{active_marker} {selection_marker} {}", model.item.label())).into_any_element()
}

pub(crate) fn render_control_group_items<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    cx: &mut App,
) -> Vec<AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    let ControlGroupTemplateHandlers { item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks } =
        handlers;

    let mut item_hovers = item_hovers.into_iter();
    let mut item_mouse_downs = item_mouse_downs.into_iter();
    let mut item_mouse_ups = item_mouse_ups.into_iter();
    let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
    let mut item_clicks = item_clicks.into_iter();
    let mut rows = Vec::with_capacity(model.items.len());

    for item in &model.items {
        let Some(item_hover) = item_hovers.next() else {
            break;
        };
        let Some(item_mouse_down) = item_mouse_downs.next() else {
            break;
        };
        let Some(item_mouse_up) = item_mouse_ups.next() else {
            break;
        };
        let Some(item_mouse_up_out) = item_mouse_up_outs.next() else {
            break;
        };
        let Some(item_click) = item_clicks.next() else {
            break;
        };

        let content = if let Some(item_template) = model.item_template {
            item_template(item, cx)
        } else {
            default_item_content(item)
        };

        let mut row = div()
            .id(format!("{}-item-{}", model.id, item.item.id()))
            .flex()
            .child(content)
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

        if item.enabled {
            row = row.cursor_pointer();
        } else {
            row = row.opacity(0.56);
        }

        rows.push(row.into_any_element());
    }

    rows
}
