use std::cell::Cell;
use std::sync::Arc;

use gpui::{AnyElement, App, MouseButton, Window, div, prelude::*};

use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::control_group::{
    ControlGroupItemLike, ControlGroupRenderModel, ControlGroupTemplate, ControlGroupTemplateHandlers,
};

const DISABLED_OPACITY: f32 = 0.56;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupLayout {
    Vertical,
    Horizontal,
}

pub fn radio_group_buttons_template<T>(
    button_template: Arc<dyn ButtonTemplate<bool>>,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |model, handlers, window, cx| {
        let rows = render_radio_button_rows(model, handlers, &button_template, window, cx);

        match layout {
            RadioGroupLayout::Vertical => {
                div().id(model.id.clone()).flex().flex_col().gap_2().items_start().children(rows)
            }
            RadioGroupLayout::Horizontal => div().id(model.id.clone()).flex().items_center().gap_3().children(rows),
        }
    })
}

pub fn render_radio_button_rows<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
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

        let render_model = ButtonRenderModel {
            id: format!("{}-{}", model.id, item.item.id()).into(),
            data: item.selected,
            content: Arc::new({
                let label = item.item.label().clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            appearance: None,
        };

        let mut button = button_template
            .render(&render_model, window, cx)
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

        if item.enabled {
            button = button.cursor_pointer();
        } else {
            button = button.opacity(DISABLED_OPACITY);
        }

        rows.push(button.into_any_element());
    }

    rows
}
