use std::cell::Cell;
use std::sync::Arc;

use gpui::{AnyElement, App, Window, div, prelude::*};

use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::control_group::{
    ControlGroupItemElementTemplate, ControlGroupItemHandlerExt, ControlGroupItemLike, ControlGroupItemRenderModel,
    ControlGroupItemTemplate, ControlGroupLayout, ControlGroupRenderModel, ControlGroupTemplate,
    ControlGroupTemplateHandlers,
};
use crate::controls::radio_button::RadioButtonData;

const DISABLED_OPACITY: f32 = 0.56;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupLayout {
    Vertical,
    Horizontal,
}

pub fn radio_group_buttons_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
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

pub fn radio_group_button_item_element_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |item, item_template, window, cx| {
        render_radio_button_option(item, item_template, &button_template, window, cx)
    })
}

pub fn render_radio_button_option<T>(
    item: &ControlGroupItemRenderModel<'_, T>,
    item_template: Option<&ControlGroupItemTemplate<T>>,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div>
where
    T: ControlGroupItemLike + 'static,
{
    if let Some(item_template) = item_template {
        let indicator_render_model = ButtonRenderModel {
            id: format!("{}-{}-indicator", item.group_id, item.item.id()).into(),
            data: RadioButtonData::new(item.selected),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::Icon,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            elevation: false,
            compact: true,
            look: None,
            ..Default::default()
        };
        let indicator = button_template.render(&indicator_render_model, window, cx);
        let content = item_template(item, window, cx);

        div()
            .id(format!("{}-item-{}", item.group_id, item.item.id()))
            .flex()
            .items_center()
            .gap_2()
            .child(indicator)
            .child(div().flex_1().min_w_0().child(content))
    } else {
        let render_model = ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: RadioButtonData::new(item.selected),
            content: Arc::new({
                let label = item.item.label().clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            elevation: true,
            compact: false,
            look: None,
            ..Default::default()
        };

        button_template.render(&render_model, window, cx)
    }
}

pub fn render_radio_button_rows<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    let mut rows = Vec::with_capacity(model.items.len());

    for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
        let mut row = render_radio_button_option(item, model.item_template, button_template, window, cx)
            .control_group_item_handlers(item_handlers);

        if model.layout == ControlGroupLayout::Horizontal {
            row = row.flex_1().h_full();
        }

        if item.enabled {
            row = row.cursor_pointer();
        } else {
            row = row.opacity(DISABLED_OPACITY);
        }

        rows.push(row.into_any_element());
    }

    rows
}
