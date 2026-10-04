use std::sync::Arc;
use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use super::TooltipRenderModel;

/// Presentation-only renderer; lifecycle and positioning are managed by the SDK.
pub type TooltipTemplate = Arc<dyn Fn(&TooltipRenderModel) -> AnyElement + Send + Sync>;

pub fn bubble(model: &TooltipRenderModel) -> AnyElement {
    div()
        .block_mouse_except_scroll()
        .flex()
        .items_center()
        .gap(px(16.0))
        .max_w(px(model.max_width))
        .px(px(model.padding))
        .py(px(model.padding_y))
        .rounded(px(model.radius))
        .shadow(model.shadow.clone())
        .bg(model.background)
        .text_color(model.foreground)
        .text_size(px(model.text_size))
        .line_height(px(model.line_height))
        .child(div().min_w_0().child(model.text.clone()))
        .when_some(model.shortcut.clone(), |root, shortcut| {
            root.child(div().flex_shrink_0().opacity(0.65).text_size(px(12.0)).child(shortcut))
        })
        .into_any_element()
}
