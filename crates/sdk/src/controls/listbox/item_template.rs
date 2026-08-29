use gpui::{AnyElement, div, px, prelude::*};

use super::ListBoxItem;
use crate::controls::control_group::{
    ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupItemTemplate, make_control_group_item_template,
};
use crate::controls::icon::{SelectionStatusIcons, render_icon_source};

const SELECTION_CHECKMARK_SIZE: f32 = 14.0;

pub fn default_listbox_item_template() -> ControlGroupItemTemplate<ListBoxItem> {
    default_listbox_item_template_with_icons(SelectionStatusIcons::default())
}

pub fn default_listbox_item_template_with_icons(icons: SelectionStatusIcons) -> ControlGroupItemTemplate<ListBoxItem> {
    make_control_group_item_template(move |item: &ControlGroupItemRenderModel<'_, ListBoxItem>, _window, _cx| {
        div()
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(render_selection_checkmark(
                item.selected,
                SELECTION_CHECKMARK_SIZE,
                &icons.selected,
                item.foreground.unwrap_or_default(),
            ))
            .child(div().flex_1().child(ControlGroupItemLike::label(item.item).to_string()))
    })
}

fn render_selection_checkmark(
    selected: bool,
    size: f32,
    icon: &crate::controls::icon::IconSource,
    color: gpui::Hsla,
) -> AnyElement {
    if selected {
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(size))
            .line_height(px(size))
            .child(render_icon_source(icon, color, size))
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}
