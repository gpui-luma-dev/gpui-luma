use std::sync::Arc;

use gpui::{AnyElement, App, Window, div, prelude::*, px};

use super::model::{ControlGroupItemLike, ControlGroupItemRenderModel};
use super::template::{ControlGroupItemElementTemplate, ControlGroupTemplate, render_control_group_item_elements};
use super::theme::{ControlGroupItemVisualContext, ControlGroupTheme};
use crate::theme::{ControlSize, LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};
use crate::theme::adorner::render_optional_adorner_with_focus_radius;

pub type MenuChoiceRowContentFn<T> = Arc<
    dyn for<'a> Fn(
            &'a ControlGroupItemRenderModel<'a, T>,
            &ControlGroupItemVisualContext,
            &mut Window,
            &mut App,
        ) -> AnyElement
        + Send
        + Sync,
>;

pub fn menu_choice_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(|model, handlers, window, cx| {
        div()
            .id(model.id.clone())
            .w_full()
            .flex()
            .flex_col()
            .children(render_control_group_item_elements(model, handlers, window, cx).into_elements())
    })
}

pub fn menu_choice_row_item_element_template<T>(
    theme: Arc<dyn ControlGroupTheme>,
    size: ControlSize,
    row_height: f32,
    row_radius: f32,
    content: MenuChoiceRowContentFn<T>,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |item, _item_template, window, cx| {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(size, metrics, scale_factor),
        );
        let mut visual = theme.resolve_item_visual(
            item.selected,
            item.state.interaction_state(),
            size,
            &scale,
            row_height,
            row_radius,
        );
        if item.state.focus_visible {
            visual.adorner = theme.resolve_item_adorner(item.selected, item.state.interaction_state(), size);
        }

        let row_content = content(item, &visual, window, cx);

        let mut row = div()
            .id(format!("{}-item-{}", item.group_id, item.item.id()))
            .relative()
            .w_full()
            .h(px(row_height))
            .flex()
            .items_center()
            .rounded(px(visual.radius))
            .bg(visual.background)
            .text_color(visual.foreground)
            .font_family(visual.font_family.clone())
            .text_size(px(visual.typography.size))
            .line_height(px(visual.typography.line_height))
            .font_weight(visual.typography.weight)
            .child(row_content);

        if item.state.disabled {
            row = row.opacity(0.56);
        } else {
            row = row.cursor_pointer();
        }

        if let Some(adorner) = render_optional_adorner_with_focus_radius(visual.adorner, visual.radius) {
            row = row.child(adorner);
        }

        row
    })
}

pub fn configure_menu_choice_group<T>(
    template: ControlGroupTemplate<T>,
    theme: Arc<dyn ControlGroupTheme>,
    size: ControlSize,
    row_height: f32,
    row_radius: f32,
    content: MenuChoiceRowContentFn<T>,
) -> (ControlGroupTemplate<T>, ControlGroupItemElementTemplate<T>)
where
    T: ControlGroupItemLike + 'static,
{
    (template, menu_choice_row_item_element_template(theme, size, row_height, row_radius, content))
}
