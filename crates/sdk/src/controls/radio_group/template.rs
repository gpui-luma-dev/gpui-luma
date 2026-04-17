use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FontWeight, MouseButton, MouseDownEvent,
    MouseUpEvent, SharedString, Stateful, Window, div, px, prelude::*,
};

use super::RadioGroupRenderModel;
use crate::theme::{RadioGroupItemAppearance, RadioGroupTheme, default_radio_group_theme};

pub type RadioGroupClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type RadioGroupHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type RadioGroupMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type RadioGroupMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct RadioGroupTemplateHandlers {
    pub item_hovers: Vec<RadioGroupHoverHandler>,
    pub item_mouse_downs: Vec<RadioGroupMouseDownHandler>,
    pub item_mouse_ups: Vec<RadioGroupMouseUpHandler>,
    pub item_mouse_up_outs: Vec<RadioGroupMouseUpHandler>,
    pub item_clicks: Vec<RadioGroupClickHandler>,
}

struct RadioGroupItemVisualModel<'a> {
    id: ElementId,
    label: &'a SharedString,
    selected: bool,
    enabled: bool,
}

pub trait RadioGroupTemplate: Send + Sync {
    fn render(
        &self,
        model: &RadioGroupRenderModel<'_>,
        handlers: RadioGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedRadioGroupTemplate {
    theme: Arc<dyn RadioGroupTheme>,
}

impl ThemedRadioGroupTemplate {
    pub fn new(theme: Arc<dyn RadioGroupTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_radio_group_template() -> Arc<dyn RadioGroupTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn RadioGroupTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedRadioGroupTemplate::new(default_radio_group_theme())))
        .clone()
}

impl RadioGroupTemplate for ThemedRadioGroupTemplate {
    fn render(
        &self,
        model: &RadioGroupRenderModel<'_>,
        handlers: RadioGroupTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let RadioGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;

        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .gap(px(12.0));

        let mut item_hovers = item_hovers.into_iter();
        let mut item_mouse_downs = item_mouse_downs.into_iter();
        let mut item_mouse_ups = item_mouse_ups.into_iter();
        let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
        let mut item_clicks = item_clicks.into_iter();

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

            let appearance = self.theme.resolve_item(item.selected, item.state);
            let mut row = render_radio_group_item_visual(
                RadioGroupItemVisualModel {
                    id: ElementId::NamedChild(
                        Arc::new(model.id.clone().into()),
                        format!("item-{}", item.id).into(),
                    ),
                    label: item.label,
                    selected: item.selected,
                    enabled: item.enabled,
                },
                appearance,
            )
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

            if item.enabled {
                row = row.cursor_pointer();
            }

            root = root.child(row);
        }

        root
    }
}

fn render_radio_group_item_visual(
    model: RadioGroupItemVisualModel<'_>,
    appearance: RadioGroupItemAppearance,
) -> Stateful<Div> {
    let indicator = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(appearance.indicator_size))
        .bg(appearance.indicator_background)
        .border_1()
        .border_color(appearance.indicator_border)
        .rounded(px(appearance.indicator_size))
        .child(render_dot(
            model.selected,
            appearance.dot_size,
            appearance.dot_color,
        ));

    let mut root = div()
        .id(model.id)
        .flex()
        .items_center()
        .gap(px(appearance.gap))
        .min_h(px(appearance.height))
        .px(px(appearance.control_padding_x))
        .py(px(appearance.control_padding_y))
        .text_color(appearance.label_color)
        .font_weight(FontWeight::MEDIUM)
        .rounded(px(appearance.control_radius))
        .child(indicator)
        .child(model.label.clone());

    if let Some(background) = appearance.control_background {
        root = root.bg(background);
    }

    if let Some(border) = appearance.control_border {
        root = root.border_1().border_color(border);
    }

    if !model.enabled {
        root = root.opacity(0.56);
    }

    if let Some(focus_ring) = appearance.focus_ring {
        root = root.focus_visible(move |style| style.border_1().border_color(focus_ring));
    }

    root
}

fn render_dot(selected: bool, size: f32, color: gpui::Hsla) -> AnyElement {
    if selected {
        div()
            .size(px(size))
            .bg(color)
            .rounded(px(size))
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}
