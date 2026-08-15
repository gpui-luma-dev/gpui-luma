use std::sync::OnceLock;

use gpui::{AnyElement, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use super::ListBoxItem;
use crate::controls::control_group::{
    ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupItemTemplate, make_control_group_item_template,
};

const SELECTION_CHECKMARK_SIZE: f32 = 14.0;

pub fn default_listbox_item_template() -> ControlGroupItemTemplate<ListBoxItem> {
    static TEMPLATE: OnceLock<ControlGroupItemTemplate<ListBoxItem>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| {
            make_control_group_item_template(|item: &ControlGroupItemRenderModel<'_, ListBoxItem>, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(render_selection_checkmark(item.selected, SELECTION_CHECKMARK_SIZE))
                    .child(div().flex_1().child(ControlGroupItemLike::label(item.item).to_string()))
            })
        })
        .clone()
}

fn render_selection_checkmark(selected: bool, size: f32) -> AnyElement {
    if selected {
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(size))
            .line_height(px(size))
            .child(crate::controls::icon::lucide_glyph(LucideIcon::Check))
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}
