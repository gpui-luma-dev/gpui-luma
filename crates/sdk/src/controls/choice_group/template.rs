use std::cell::Cell;
use std::sync::{Arc, OnceLock};

use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*,
};

use super::{ChoiceGroupItemContentModel, ChoiceGroupItemPosition, ChoiceGroupLayout, ChoiceGroupRenderModel};
use crate::controls::button_family::ButtonKind;
use crate::controls::choice_group::ChoiceGroupItemButtonRenderModel;
use crate::theme::{ButtonVariant, ChoiceGroupItemAppearance, ChoiceGroupTheme, default_choice_group_theme};

const DISABLED_OPACITY: f32 = 0.56;

pub type ChoiceGroupClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ChoiceGroupHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ChoiceGroupMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ChoiceGroupMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ChoiceGroupTemplateHandlers {
    pub item_hovers: Vec<ChoiceGroupHoverHandler>,
    pub item_mouse_downs: Vec<ChoiceGroupMouseDownHandler>,
    pub item_mouse_ups: Vec<ChoiceGroupMouseUpHandler>,
    pub item_mouse_up_outs: Vec<ChoiceGroupMouseUpHandler>,
    pub item_clicks: Vec<ChoiceGroupClickHandler>,
}

pub type ChoiceGroupModifier =
    Box<dyn for<'a> Fn(Stateful<Div>, &ChoiceGroupRenderModel<'a>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct ModifiedChoiceGroupTemplate {
    base: Arc<dyn ChoiceGroupTemplate>,
    modifiers: Vec<ChoiceGroupModifier>,
}

impl ModifiedChoiceGroupTemplate {
    pub fn new(base: Arc<dyn ChoiceGroupTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &ChoiceGroupRenderModel<'a>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ChoiceGroupRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl ChoiceGroupTemplate for ModifiedChoiceGroupTemplate {
    fn render(
        &self,
        model: &ChoiceGroupRenderModel<'_>,
        handlers: ChoiceGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn template_with_modifier<F>(template: Arc<dyn ChoiceGroupTemplate>, modifier: F) -> Arc<dyn ChoiceGroupTemplate>
where
    F: for<'a> Fn(Stateful<Div>, &ChoiceGroupRenderModel<'a>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedChoiceGroupTemplate::new(template).with_modifier(modifier))
}

struct ChoiceGroupItemVisualModel {
    id: ElementId,
    position: ChoiceGroupItemPosition,
    layout: ChoiceGroupLayout,
    state: crate::controls::state::CompositeItemState,
    content: gpui::AnyElement,
    icon_button_like: bool,
}

pub trait ChoiceGroupTemplate: Send + Sync {
    fn render(
        &self,
        model: &ChoiceGroupRenderModel<'_>,
        handlers: ChoiceGroupTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedChoiceGroupTemplate {
    theme: Arc<dyn ChoiceGroupTheme>,
    modifiers: Vec<ChoiceGroupModifier>,
}

impl ThemedChoiceGroupTemplate {
    pub fn new(theme: Arc<dyn ChoiceGroupTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &ChoiceGroupRenderModel<'a>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ChoiceGroupRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_choice_group_template() -> Arc<dyn ChoiceGroupTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ChoiceGroupTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedChoiceGroupTemplate::new(default_choice_group_theme())))
        .clone()
}

impl ChoiceGroupTemplate for ThemedChoiceGroupTemplate {
    fn render(
        &self,
        model: &ChoiceGroupRenderModel<'_>,
        handlers: ChoiceGroupTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ChoiceGroupTemplateHandlers {
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
            .overflow_hidden()
            .rounded(px(list_appearance.radius))
            .bg(list_appearance.background)
            .border_1()
            .border_color(list_appearance.border);

        root = match model.layout {
            ChoiceGroupLayout::Horizontal => root.items_center(),
            ChoiceGroupLayout::Vertical => root.flex_col().items_stretch(),
        };

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

            if model.item_button_template.is_some() {
                let control = div()
                    .id(ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("item-{}", item.id).into()))
                    .child(render_choice_group_item_content(model, item.content_model.clone(), _window, _cx))
                    .on_hover(item_hover)
                    .on_mouse_down(MouseButton::Left, item_mouse_down)
                    .on_mouse_up(MouseButton::Left, item_mouse_up)
                    .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
                    .on_click(item_click);

                root = root.child(control);
            } else {
                let appearance = self.theme.resolve_item(
                    button_variant(model.kind),
                    item.selected,
                    item.state.interaction_state(),
                    model.size,
                );

                let icon_button_like =
                    model.kind == ButtonKind::Ghost && matches!(model.layout, ChoiceGroupLayout::Horizontal);

                let mut control = render_choice_group_item_visual(
                    ChoiceGroupItemVisualModel {
                        id: ElementId::NamedChild(
                            Arc::new(model.id.clone().into()),
                            format!("item-{}", item.id).into(),
                        ),
                        position: item.position,
                        layout: model.layout,
                        state: item.state,
                        content: render_choice_group_item_content(model, item.content_model.clone(), _window, _cx),
                        icon_button_like,
                    },
                    appearance,
                )
                .on_hover(item_hover)
                .on_mouse_down(MouseButton::Left, item_mouse_down)
                .on_mouse_up(MouseButton::Left, item_mouse_up)
                .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
                .on_click(item_click);

                if !item.state.disabled {
                    control = control.cursor_pointer();
                }

                root = root.child(control);
            }
        }

        self.apply_modifiers(root, model)
    }
}

fn render_choice_group_item_content(
    model: &ChoiceGroupRenderModel<'_>,
    content_model: ChoiceGroupItemContentModel,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    if let Some(item_button_template) = model.item_button_template.as_ref() {
        let presenter = model.content.clone();
        let content_model_for_presenter = content_model.clone();

        let button_model = ChoiceGroupItemButtonRenderModel {
            id: format!("{}-{}", model.id, content_model.item_id).into(),
            data: content_model.selected,
            content: Arc::new(move |_, cx| presenter(&content_model_for_presenter, cx)),
            kind: model.kind,
            size: model.size,
            state: content_model.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
        };

        item_button_template.render(&button_model, window, cx).into_any_element()
    } else {
        (model.content)(&content_model, cx)
    }
}

fn render_choice_group_item_visual(
    model: ChoiceGroupItemVisualModel,
    appearance: ChoiceGroupItemAppearance,
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
        .child(model.content);

    let icon_button_diameter = appearance.height * 0.75;

    if model.icon_button_like {
        root = root.min_h(px(icon_button_diameter)).size(px(icon_button_diameter)).p_0().rounded_full();
    } else {
        root = apply_item_divider(root, model.layout, model.position, appearance.divider);
        root = apply_item_radius(root, model.position, appearance.radius);
    }

    if let Some(focus_ring) = appearance.focus_ring {
        let (focus_position, focus_radius) = if model.icon_button_like {
            (ChoiceGroupItemPosition::Only, icon_button_diameter / 2.0)
        } else {
            (model.position, appearance.radius)
        };
        root = root.child(render_focus_ring(focus_ring, focus_position, focus_radius));
    }

    if model.state.disabled {
        root = root.opacity(DISABLED_OPACITY);
    }

    root
}

fn apply_item_divider(
    mut root: Stateful<Div>,
    layout: ChoiceGroupLayout,
    position: ChoiceGroupItemPosition,
    divider: gpui::Hsla,
) -> Stateful<Div> {
    let first = matches!(position, ChoiceGroupItemPosition::First | ChoiceGroupItemPosition::Only);
    if first {
        return root;
    }

    root = match layout {
        ChoiceGroupLayout::Horizontal => root.border_l_1(),
        ChoiceGroupLayout::Vertical => root.border_t_1(),
    };

    root.border_color(divider)
}

fn apply_item_radius(root: Stateful<Div>, position: ChoiceGroupItemPosition, radius: f32) -> Stateful<Div> {
    let radius = px(radius);

    match position {
        ChoiceGroupItemPosition::Only => root.rounded(radius),
        ChoiceGroupItemPosition::First => root.rounded_tl(radius).rounded_bl(radius),
        ChoiceGroupItemPosition::Middle => root,
        ChoiceGroupItemPosition::Last => root.rounded_tr(radius).rounded_br(radius),
    }
}

fn render_focus_ring(color: gpui::Hsla, position: ChoiceGroupItemPosition, radius: f32) -> Div {
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

fn apply_focus_ring_radius(root: Div, position: ChoiceGroupItemPosition, radius: f32) -> Div {
    let radius = px(radius);

    match position {
        ChoiceGroupItemPosition::Only => root.rounded(radius),
        ChoiceGroupItemPosition::First => root.rounded_tl(radius).rounded_bl(radius),
        ChoiceGroupItemPosition::Middle => root,
        ChoiceGroupItemPosition::Last => root.rounded_tr(radius).rounded_br(radius),
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Standard => ButtonVariant::Standard,
        ButtonKind::Ghost => ButtonVariant::Ghost,
        ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}
