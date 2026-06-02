use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, FontWeight, MouseDownEvent, MouseUpEvent, Stateful, Window, div, px, prelude::*,
};
use lucide_icons::Icon as LucideIcon;

use super::{AccordionRenderModel, AccordionTheme, default_accordion_theme};
use crate::theme::{ControlSize, LayoutCacheKey, LumaLayoutCacheExt};

pub type AccordionHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type AccordionMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type AccordionMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type AccordionClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type AccordionTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

#[derive(Default)]
pub struct AccordionTemplateHandlers {
    pub trigger_hovers: Vec<AccordionHoverHandler>,
    pub trigger_mouse_downs: Vec<AccordionMouseDownHandler>,
    pub trigger_mouse_ups: Vec<AccordionMouseUpHandler>,
    pub trigger_clicks: Vec<AccordionClickHandler>,
}

pub trait AccordionTemplate: Send + Sync {
    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedAccordionTemplate {
    theme: Arc<dyn AccordionTheme>,
    modifiers: Vec<AccordionTemplateModifier>,
}

impl ThemedAccordionTemplate {
    pub fn new(theme: Arc<dyn AccordionTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &AccordionRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedAccordionTemplate {
    base: Arc<dyn AccordionTemplate>,
    modifiers: Vec<AccordionTemplateModifier>,
}

impl ModifiedAccordionTemplate {
    fn new(base: Arc<dyn AccordionTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: AccordionTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &AccordionRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }

    fn into_arc(self) -> Arc<dyn AccordionTemplate> {
        Arc::new(self)
    }
}

impl AccordionTemplate for ModifiedAccordionTemplate {
    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_accordion_template() -> Arc<dyn AccordionTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn AccordionTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedAccordionTemplate::new(default_accordion_theme()))).clone()
}

pub fn accordion_template_with_modifier<F>(
    template: Arc<dyn AccordionTemplate>,
    modifier: F,
) -> Arc<dyn AccordionTemplate>
where
    F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedAccordionTemplate::new(template).with_modifier(Box::new(modifier)).into_arc()
}

impl AccordionTemplate for ThemedAccordionTemplate {
    fn render(
        &self,
        model: &AccordionRenderModel<'_>,
        handlers: AccordionTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| super::theme::AccordionScale::compute(ControlSize::Md, metrics, scale_factor),
        );

        let AccordionTemplateHandlers { trigger_hovers, trigger_mouse_downs, trigger_mouse_ups, trigger_clicks } =
            handlers;

        let mut trigger_hovers = trigger_hovers.into_iter();
        let mut trigger_mouse_downs = trigger_mouse_downs.into_iter();
        let mut trigger_mouse_ups = trigger_mouse_ups.into_iter();
        let mut trigger_clicks = trigger_clicks.into_iter();

        let mut root = div().id(model.id.clone()).flex().flex_col().w_full().gap(px(scale.item_gap));

        for item in &model.items {
            let Some(hover_handler) = trigger_hovers.next() else {
                break;
            };
            let Some(down_handler) = trigger_mouse_downs.next() else {
                break;
            };
            let Some(up_handler) = trigger_mouse_ups.next() else {
                break;
            };
            let Some(click_handler) = trigger_clicks.next() else {
                break;
            };

            let trigger_palette = self.theme.resolve_trigger(item.state.interaction_state());
            let content_palette = self.theme.resolve_content(item.expanded);

            let mut trigger = div()
                .id(format!("{}-trigger", item.id))
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .min_h(px(scale.trigger_height))
                .px(px(scale.padding_x))
                .py(px(scale.padding_y))
                .rounded(px(scale.radius))
                .text_color(trigger_palette.foreground)
                .text_size(px(trigger_palette.typography.size))
                .line_height(px(trigger_palette.typography.line_height))
                .font_family(trigger_palette.font_family.clone())
                .font_weight(trigger_palette.typography.weight)
                .cursor_pointer();

            if let Some(background) = trigger_palette.background {
                trigger = trigger.bg(background);
            }

            if item.enabled {
                trigger = trigger
                    .on_hover(hover_handler)
                    .on_mouse_down(gpui::MouseButton::Left, down_handler)
                    .on_mouse_up(gpui::MouseButton::Left, up_handler)
                    .on_click(click_handler);
            } else {
                trigger = trigger.opacity(0.56);
            }

            let trigger_content = match &item.trigger.custom_element {
                Some(renderer) => renderer(),
                None => {
                    let mut row = div().flex().items_center().gap(px(scale.inner_gap));
                    if let Some(icon) = item.trigger.icon {
                        row = row.child(render_icon(icon, trigger_palette.icon_color, scale.icon_size));
                    }
                    row = row.child(item.trigger.label.clone().unwrap_or_default());
                    row.into_any_element()
                }
            };

            let chevron_icon = if item.expanded {
                LucideIcon::ChevronDown
            } else {
                LucideIcon::ChevronRight
            };
            let chevron = render_icon(chevron_icon, trigger_palette.chevron_color, scale.chevron_size);

            let trigger_el = trigger.child(trigger_content).child(chevron);

            let content_el = if item.expanded {
                let mut content = div()
                    .id(format!("{}-content", item.id))
                    .w_full()
                    .px(px(scale.padding_x))
                    .py(px(scale.content_padding_y))
                    .text_color(content_palette.foreground);

                if let Some(background) = content_palette.background {
                    content = content.bg(background);
                }

                if let Some(renderer) = &item.content.element {
                    content = content.child(renderer());
                }

                Some(content)
            } else {
                None
            };

            let mut item_container = div()
                .id(format!("{}-item", item.id))
                .flex()
                .flex_col()
                .w_full()
                .border_b_1()
                .border_color(trigger_palette.border_color)
                .child(trigger_el);

            if let Some(cel) = content_el {
                item_container = item_container.child(cel);
            }

            root = root.child(item_container);
        }

        self.apply_modifiers(root, model)
    }
}

fn render_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
