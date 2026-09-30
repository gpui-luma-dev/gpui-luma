//! Segmented presentation for the SDK radio group; selection stays SDK-owned.

use std::sync::Arc;
use gpui::{IntoElement, div, prelude::*, px};
use luma::controls::control_group::{ControlGroupItemHandlerExt, ControlGroupItemLike};
use luma::controls::radio_group::RadioGroupTemplate;
use crate::{Look, ScaleFamily};

/// Horizontal radio-group template with a shared track and selected segment.
/// Item templates supply content only; colors and geometry belong to the look.
pub fn segmented_radio_template<T: ControlGroupItemLike + 'static>(look: &Look) -> RadioGroupTemplate<T> {
    let look = look.clone();
    Arc::new(move |model, handlers, window, cx| {
        let gray = |step| look.resolve_step(ScaleFamily::Gray, step).hsla();
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let fill = if item.state.pressed {
                    6
                } else if item.selected {
                    5
                } else if item.state.hovered {
                    4
                } else {
                    3
                };
                let border = if item.state.focus_visible {
                    look.resolve_step(ScaleFamily::Color, 8).hsla()
                } else {
                    gray(if item.selected { 6 } else { fill })
                };
                let content = match model.item_template {
                    Some(template) => template(item, window, cx),
                    None => div().child(item.item.label().clone()).into_any_element(),
                };
                div()
                    .id(format!("{}-{}", model.id, item.item.id()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .h(px(32.0))
                    .px(px(12.0))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(border)
                    .bg(gray(fill))
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .text_color(gray(if item.enabled { 12 } else { 8 }))
                    .when(item.enabled, |row| row.cursor_pointer())
                    .child(content)
                    .control_group_item_handlers(handlers)
            })
            .collect::<Vec<_>>();
        div().id(model.id.clone()).flex().items_center().rounded(px(5.0)).bg(gray(3)).children(rows)
    })
}
