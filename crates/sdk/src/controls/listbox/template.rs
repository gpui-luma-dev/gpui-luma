use std::sync::{Arc, OnceLock};

use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*,
};

use super::ListBoxRenderModel;
use crate::controls::listbox::{ListBoxRowAppearance, ListBoxTheme, default_listbox_theme};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner, render_optional_adorner_with_focus_radius};

pub type ListBoxClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ListBoxHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ListBoxMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ListBoxMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ListBoxTemplateHandlers {
    pub item_hovers: Vec<ListBoxHoverHandler>,
    pub item_mouse_downs: Vec<ListBoxMouseDownHandler>,
    pub item_mouse_ups: Vec<ListBoxMouseUpHandler>,
    pub item_mouse_up_outs: Vec<ListBoxMouseUpHandler>,
    pub item_clicks: Vec<ListBoxClickHandler>,
}

pub trait ListBoxTemplate: Send + Sync {
    fn render(
        &self,
        model: &ListBoxRenderModel<'_>,
        handlers: ListBoxTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedListBoxTemplate {
    theme: Arc<dyn ListBoxTheme>,
}

impl ThemedListBoxTemplate {
    pub fn new(theme: Arc<dyn ListBoxTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_listbox_template() -> Arc<dyn ListBoxTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ListBoxTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedListBoxTemplate::new(default_listbox_theme()))).clone()
}

impl ListBoxTemplate for ThemedListBoxTemplate {
    fn render(
        &self,
        model: &ListBoxRenderModel<'_>,
        handlers: ListBoxTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ListBoxTemplateHandlers { item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks } =
            handlers;

        let list_appearance = self.theme.resolve_list(model.enabled, model.focus.focused, model.size);
        let focused_probe_list_appearance = if model.enabled {
            Some(self.theme.resolve_list(model.enabled, true, model.size))
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

            let row_appearance = self.theme.resolve_row(item.selected, item.state.interaction_state(), model.size);
            let focused_probe_row_appearance = if any_item_enabled && !item.state.disabled {
                let mut focused_state = item.state.interaction_state();
                focused_state.focused = true;
                Some(self.theme.resolve_row(item.selected, focused_state, model.size))
            } else {
                None
            };
            let row_oversize_extent = adorner_oversize_extent(row_appearance.adorner).max(
                focused_probe_row_appearance
                    .as_ref()
                    .map(|probe| adorner_oversize_extent(probe.adorner))
                    .unwrap_or(0.0),
            );
            let row = if model.item_button_template.is_some() {
                let content =
                    super::item_template::render_listbox_row_content(model, item.content_model.clone(), _window, _cx);
                div()
                    .id(ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("item-{}", item.id).into()))
                    .relative()
                    .w_full()
                    .min_h(px(row_appearance.height))
                    .flex()
                    .items_center()
                    .px(px(row_appearance.padding_x))
                    .py(px(row_appearance.padding_y))
                    .rounded(px(row_appearance.radius))
                    .bg(row_appearance.background)
                    .text_color(row_appearance.label_color)
                    .child(content)
            } else {
                render_listbox_row_visual(
                    ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("item-{}", item.id).into()),
                    item.state,
                    (model.content)(&item.content_model, _cx),
                    row_appearance,
                )
            }
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

            let row = if !item.state.disabled {
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
