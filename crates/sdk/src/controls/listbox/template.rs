use std::sync::{Arc, OnceLock};

use gpui::{App, Div, ElementId, Stateful, Window, div, px, prelude::*};

use super::ListBoxItem;
use crate::controls::control_group::{
    ControlGroupItemElementTemplate, ControlGroupItemLike, ControlGroupLayout, ControlGroupRenderModel,
    ControlGroupTemplate, ControlGroupTemplateHandlers, render_control_group_item_elements,
};
use crate::controls::listbox::{ListBoxRowLook, ListBoxTheme, default_listbox_theme};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner, render_optional_adorner_with_focus_radius};
use crate::theme::{ControlSize, LayoutCacheKey, ListRowScale, LumaLayoutCacheExt};

pub struct ThemedListBoxTemplate {
    theme: Arc<dyn ListBoxTheme>,
    size: ControlSize,
}

impl ThemedListBoxTemplate {
    pub fn new(theme: Arc<dyn ListBoxTheme>) -> Self {
        Self { theme, size: ControlSize::Md }
    }

    pub fn with_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    fn render(
        &self,
        model: &ControlGroupRenderModel<'_, ListBoxItem>,
        handlers: ControlGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, model.focus.focused, self.size);
        let focused_probe_list_look = if model.enabled {
            Some(self.theme.resolve_list(model.enabled, true, self.size))
        } else {
            None
        };
        let list_oversize_extent = adorner_oversize_extent(list_look.adorner)
            .max(focused_probe_list_look.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .gap(px(list_look.row_gap))
            .overflow_hidden()
            .px(px(list_look.padding_x))
            .py(px(list_look.padding_y))
            .rounded(px(list_look.radius))
            .bg(list_look.background)
            .border_1()
            .border_color(list_look.border);

        if let Some(adorner) = render_optional_adorner(list_look.adorner, list_look.radius) {
            root = root.child(adorner);
        }

        root = match model.layout {
            ControlGroupLayout::Horizontal => root.flex_row().items_center(),
            ControlGroupLayout::Vertical => root.flex_col(),
        };

        let default_item_element_template = self.row_item_element_template();
        let row_model = ControlGroupRenderModel {
            id: model.id,
            items: model.items.clone(),
            selected_ids: model.selected_ids,
            active_id: model.active_id,
            selection_mode: model.selection_mode,
            state_mode: model.state_mode,
            enabled: model.enabled,
            layout: model.layout,
            focus: model.focus,
            focus_strategy: model.focus_strategy,
            item_template: model.item_template,
            item_element_template: model.item_element_template.or(Some(&default_item_element_template)),
        };

        root = root.children(render_control_group_item_elements(&row_model, handlers, window, cx).into_elements());

        if list_oversize_extent > 0.0 {
            div().id(format!("{}-list-slot", model.id)).relative().p(px(list_oversize_extent)).child(root)
        } else {
            root
        }
    }

    fn row_item_element_template(&self) -> ControlGroupItemElementTemplate<ListBoxItem> {
        listbox_row_item_element_template_with_size(self.theme.clone(), self.size)
    }
}

pub fn default_listbox_template() -> ControlGroupTemplate<ListBoxItem> {
    listbox_template_with_theme(default_listbox_theme())
}

pub fn listbox_template_with_theme(theme: Arc<dyn ListBoxTheme>) -> ControlGroupTemplate<ListBoxItem> {
    listbox_template_with_theme_and_size(theme, ControlSize::Md)
}

pub fn listbox_template_with_theme_and_size(
    theme: Arc<dyn ListBoxTheme>,
    size: ControlSize,
) -> ControlGroupTemplate<ListBoxItem> {
    let themed = Arc::new(ThemedListBoxTemplate::new(theme).with_size(size));
    Arc::new(move |model, handlers, window, cx| themed.render(model, handlers, window, cx))
}

fn default_themed_listbox_template() -> Arc<ThemedListBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<ThemedListBoxTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedListBoxTemplate::new(default_listbox_theme()))).clone()
}

pub fn shared_listbox_template() -> ControlGroupTemplate<ListBoxItem> {
    let themed = default_themed_listbox_template();
    Arc::new(move |model, handlers, window, cx| themed.render(model, handlers, window, cx))
}

pub fn default_listbox_row_item_element_template() -> ControlGroupItemElementTemplate<ListBoxItem> {
    static TEMPLATE: OnceLock<ControlGroupItemElementTemplate<ListBoxItem>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| listbox_row_item_element_template_with_theme(default_listbox_theme()))
        .clone()
}

pub fn listbox_row_item_element_template_with_theme(
    theme: Arc<dyn ListBoxTheme>,
) -> ControlGroupItemElementTemplate<ListBoxItem> {
    listbox_row_item_element_template_with_size(theme, ControlSize::Md)
}

fn listbox_row_item_element_template_with_size(
    theme: Arc<dyn ListBoxTheme>,
    size: ControlSize,
) -> ControlGroupItemElementTemplate<ListBoxItem> {
    Arc::new(move |item, item_template, window, cx| {
        let scale_factor = window.scale_factor();
        let row_scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| ListRowScale::compute(size, metrics, scale_factor),
        );
        let row_look = theme.resolve_row_look(item.selected, item.state.interaction_state(), size, &row_scale);
        let focused_probe_row_look = if !item.state.disabled {
            let mut focused_state = item.state.interaction_state();
            focused_state.focused = true;
            Some(theme.resolve_row_look(item.selected, focused_state, size, &row_scale))
        } else {
            None
        };
        let row_oversize_extent = adorner_oversize_extent(row_look.adorner)
            .max(focused_probe_row_look.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));

        let content = if let Some(item_template) = item_template {
            item_template(item, window, cx)
        } else {
            div().child(ControlGroupItemLike::label(item.item).to_string()).into_any_element()
        };

        let row = render_listbox_row_visual(
            ElementId::NamedChild(Arc::new(item.group_id.clone().into()), format!("item-{}", item.item.id()).into()),
            item.state,
            content,
            row_look,
        );

        if row_oversize_extent > 0.0 {
            div()
                .id(ElementId::NamedChild(
                    Arc::new(item.group_id.clone().into()),
                    format!("item-slot-{}", item.item.id()).into(),
                ))
                .relative()
                .p(px(row_oversize_extent))
                .child(row)
        } else {
            row
        }
    })
}

fn render_listbox_row_visual(
    id: ElementId,
    state: crate::controls::state::CompositeItemState,
    content: gpui::AnyElement,
    look: ListBoxRowLook,
) -> Stateful<Div> {
    let mut root = div()
        .id(id)
        .relative()
        .w_full()
        .min_h(px(look.height))
        .flex()
        .items_center()
        .px(px(look.padding_x))
        .py(px(look.padding_y))
        .bg(look.background)
        .text_color(look.label_color)
        .text_size(px(look.label_typography.size))
        .line_height(px(look.label_typography.line_height))
        .font_weight(look.label_typography.weight)
        .child(div().w_full().mt(px(look.label_baseline_shift)).child(content));

    if let Some(adorner) = render_optional_adorner_with_focus_radius(look.adorner, look.radius) {
        root = root.child(adorner);
    }

    if state.disabled {
        root = root.opacity(0.56);
    }

    root
}
