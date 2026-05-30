use gpui::{AnyElement, Stateful, div, prelude::*, px};

use super::theme::ListViewRowAppearance;
use crate::theme::adorner::render_optional_adorner_with_focus_radius;

pub(crate) fn render_list_view_row(
    id: impl Into<gpui::ElementId>,
    content: AnyElement,
    appearance: ListViewRowAppearance,
    enabled: bool,
    show_top_divider: bool,
    is_custom: bool,
) -> Stateful<gpui::Div> {
    let mut row = div().id(id).relative().w_full();

    if is_custom {
        row = row.child(div().flex_1().min_w(px(0.0)).child(content));
    } else {
        row = row
            .min_h(px(appearance.min_height))
            .flex()
            .items_center()
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .bg(appearance.background)
            .text_color(appearance.label_color)
            .text_size(px(appearance.label_typography.size))
            .line_height(px(appearance.label_typography.line_height))
            .font_weight(appearance.label_typography.weight)
            .child(div().flex_1().min_w(px(0.0)).child(content));
    }

    if show_top_divider {
        row = row.border_t_1().border_color(appearance.divider);
    }

    if enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(0.56);
    }

    if let Some(adorner) = render_optional_adorner_with_focus_radius(appearance.adorner, appearance.radius) {
        row = row.child(adorner);
    }

    row
}
