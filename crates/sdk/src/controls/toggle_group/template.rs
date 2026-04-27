use std::sync::{Arc, OnceLock};

use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, MouseDownEvent, MouseUpEvent, SharedString, Stateful, Window, div,
    px, prelude::*,
};

use super::{ToggleGroupItemPosition, ToggleGroupRenderModel};
use crate::controls::button_family::ButtonKind;
use crate::theme::{ButtonVariant, ToggleGroupItemAppearance, ToggleGroupTheme, default_toggle_group_theme};

pub type ToggleGroupClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ToggleGroupHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ToggleGroupMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ToggleGroupMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ToggleGroupTemplateHandlers {
    pub item_hovers: Vec<ToggleGroupHoverHandler>,
    pub item_mouse_downs: Vec<ToggleGroupMouseDownHandler>,
    pub item_mouse_ups: Vec<ToggleGroupMouseUpHandler>,
    pub item_mouse_up_outs: Vec<ToggleGroupMouseUpHandler>,
    pub item_clicks: Vec<ToggleGroupClickHandler>,
}

struct ToggleGroupItemVisualModel<'a> {
    id: ElementId,
    label: &'a SharedString,
    position: ToggleGroupItemPosition,
    state: crate::controls::state::CompositeItemState,
}

pub trait ToggleGroupTemplate: Send + Sync {
    fn render(
        &self,
        model: &ToggleGroupRenderModel<'_>,
        handlers: ToggleGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedToggleGroupTemplate {
    theme: Arc<dyn ToggleGroupTheme>,
}

impl ThemedToggleGroupTemplate {
    pub fn new(theme: Arc<dyn ToggleGroupTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_toggle_group_template() -> Arc<dyn ToggleGroupTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToggleGroupTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedToggleGroupTemplate::new(default_toggle_group_theme())))
        .clone()
}

impl ToggleGroupTemplate for ThemedToggleGroupTemplate {
    fn render(
        &self,
        model: &ToggleGroupRenderModel<'_>,
        handlers: ToggleGroupTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ToggleGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;
        let list_appearance = self.theme.resolve_list(model.enabled, model.size);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .overflow_hidden()
            .rounded(px(list_appearance.radius))
            .bg(list_appearance.background)
            .border_1()
            .border_color(list_appearance.border);

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

            let appearance = self.theme.resolve_item(
                button_variant(model.kind),
                item.selected,
                item.state.interaction_state(),
                model.size,
            );
            let mut button = render_toggle_group_item_visual(
                ToggleGroupItemVisualModel {
                    id: ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("item-{}", item.id).into()),
                    label: item.label,
                    position: item.position,
                    state: item.state,
                },
                appearance,
            )
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

            if !item.state.disabled {
                button = button.cursor_pointer();
            }

            root = root.child(button);
        }

        root
    }
}

fn render_toggle_group_item_visual(
    model: ToggleGroupItemVisualModel<'_>,
    appearance: ToggleGroupItemAppearance,
) -> Stateful<Div> {
    let mut root = div()
        .id(model.id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .min_h(px(appearance.height))
        .px(px(appearance.padding_x))
        .py(px(appearance.padding_y))
        .bg(appearance.background)
        .text_color(appearance.label_color)
        .text_size(px(appearance.label_typography.size))
        .line_height(px(appearance.label_typography.line_height))
        .font_weight(appearance.label_typography.weight)
        .child(model.label.clone());

    if model.position != ToggleGroupItemPosition::First && model.position != ToggleGroupItemPosition::Only {
        root = root.border_l_1().border_color(appearance.divider);
    }

    root = apply_item_radius(root, model.position, appearance.radius);

    if let Some(focus_ring) = appearance.focus_ring {
        root = root.child(render_focus_ring(focus_ring, model.position, appearance.radius));
    }

    if model.state.disabled {
        root = root.opacity(0.56);
    }

    root
}

fn apply_item_radius(root: Stateful<Div>, position: ToggleGroupItemPosition, radius: f32) -> Stateful<Div> {
    let radius = px(radius);

    match position {
        ToggleGroupItemPosition::Only => root.rounded(radius),
        ToggleGroupItemPosition::First => root.rounded_tl(radius).rounded_bl(radius),
        ToggleGroupItemPosition::Middle => root,
        ToggleGroupItemPosition::Last => root.rounded_tr(radius).rounded_br(radius),
    }
}

fn render_focus_ring(color: gpui::Hsla, position: ToggleGroupItemPosition, radius: f32) -> Div {
    let inset = 1.0;
    let radius = (radius - inset).max(0.0);
    let ring = div()
        .absolute()
        .top(px(inset))
        .right(px(inset))
        .bottom(px(inset))
        .left(px(inset))
        .border_1()
        .border_color(color);

    apply_focus_ring_radius(ring, position, radius)
}

fn apply_focus_ring_radius(root: Div, position: ToggleGroupItemPosition, radius: f32) -> Div {
    let radius = px(radius);

    match position {
        ToggleGroupItemPosition::Only => root.rounded(radius),
        ToggleGroupItemPosition::First => root.rounded_tl(radius).rounded_bl(radius),
        ToggleGroupItemPosition::Middle => root,
        ToggleGroupItemPosition::Last => root.rounded_tr(radius).rounded_br(radius),
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Standard => ButtonVariant::Standard,
        ButtonKind::Ghost => ButtonVariant::Ghost,
        ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}
