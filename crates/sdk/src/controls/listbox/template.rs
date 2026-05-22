use std::sync::{Arc, OnceLock};

use gpui::{App, Div, ElementId, MouseButton, Stateful, Window, div, px, prelude::*};

use super::ListBoxItem;
use crate::controls::control_group::{
    ControlGroupItemLike, ControlGroupRenderModel, ControlGroupTemplate, ControlGroupTemplateHandlers,
};
use crate::controls::listbox::{ListBoxRowAppearance, ListBoxTheme, default_listbox_theme};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner, render_optional_adorner_with_focus_radius};
use crate::theme::ControlSize;

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
        let ControlGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;

        let list_appearance = self.theme.resolve_list(model.enabled, model.focus.focused, self.size);
        let focused_probe_list_appearance = if model.enabled {
            Some(self.theme.resolve_list(model.enabled, true, self.size))
        } else {
            None
        };
        let list_oversize_extent = adorner_oversize_extent(list_appearance.adorner).max(
            focused_probe_list_appearance
                .as_ref()
                .map(|probe| adorner_oversize_extent(probe.adorner))
                .unwrap_or(0.0),
        );

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(list_appearance.row_gap))
            .overflow_hidden()
            .px(px(list_appearance.padding_x))
            .py(px(list_appearance.padding_y))
            .rounded(px(list_appearance.radius))
            .bg(list_appearance.background)
            .border_1()
            .border_color(list_appearance.border);

        if let Some(adorner) = render_optional_adorner(list_appearance.adorner, list_appearance.radius) {
            root = root.child(adorner);
        }

        let mut item_hovers = item_hovers.into_iter();
        let mut item_mouse_downs = item_mouse_downs.into_iter();
        let mut item_mouse_ups = item_mouse_ups.into_iter();
        let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
        let mut item_clicks = item_clicks.into_iter();

        let any_item_enabled = model.items.iter().any(|item| item.enabled);

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

            let row_appearance = self.theme.resolve_row(item.selected, item.state.interaction_state(), self.size);
            let focused_probe_row_appearance = if any_item_enabled && !item.state.disabled {
                let mut focused_state = item.state.interaction_state();
                focused_state.focused = true;
                Some(self.theme.resolve_row(item.selected, focused_state, self.size))
            } else {
                None
            };
            let row_oversize_extent = adorner_oversize_extent(row_appearance.adorner).max(
                focused_probe_row_appearance
                    .as_ref()
                    .map(|probe| adorner_oversize_extent(probe.adorner))
                    .unwrap_or(0.0),
            );

            let content = if let Some(item_template) = model.item_template {
                item_template(item, window, cx)
            } else {
                div().child(ControlGroupItemLike::label(item.item).to_string()).into_any_element()
            };

            let row = render_listbox_row_visual(
                ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("item-{}", item.item.id()).into()),
                item.state,
                content,
                row_appearance,
            )
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

            let row = if item.enabled {
                row.cursor_pointer()
            } else {
                row.opacity(0.56)
            };

            let row = if row_oversize_extent > 0.0 {
                div().relative().p(px(row_oversize_extent)).child(row).into_any_element()
            } else {
                row.into_any_element()
            };

            root = root.child(row);
        }

        if list_oversize_extent > 0.0 {
            div().id(format!("{}-list-slot", model.id)).relative().p(px(list_oversize_extent)).child(root)
        } else {
            root
        }
    }
}

pub fn default_listbox_template() -> ControlGroupTemplate<ListBoxItem> {
    listbox_template_with_theme(default_listbox_theme())
}

pub fn listbox_template_with_theme(theme: Arc<dyn ListBoxTheme>) -> ControlGroupTemplate<ListBoxItem> {
    let themed = Arc::new(ThemedListBoxTemplate::new(theme));
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

fn render_listbox_row_visual(
    id: ElementId,
    state: crate::controls::state::CompositeItemState,
    content: gpui::AnyElement,
    appearance: ListBoxRowAppearance,
) -> Stateful<Div> {
    let mut root = div()
        .id(id)
        .relative()
        .w_full()
        .min_h(px(appearance.height))
        .flex()
        .items_center()
        .px(px(appearance.padding_x))
        .py(px(appearance.padding_y))
        .bg(appearance.background)
        .text_color(appearance.label_color)
        .text_size(px(appearance.label_typography.size))
        .line_height(px(appearance.label_typography.line_height))
        .font_weight(appearance.label_typography.weight)
        .child(content);

    if let Some(adorner) = render_optional_adorner_with_focus_radius(appearance.adorner, appearance.radius) {
        root = root.child(adorner);
    }

    if state.disabled {
        root = root.opacity(0.56);
    }

    root
}
