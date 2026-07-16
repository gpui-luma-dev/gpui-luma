use std::sync::{Arc, OnceLock};

use gpui::{App, Div, ElementId, SharedString, Stateful, TextRun, Window, div, font, px, prelude::*};

use super::{TabsNavigationItem, TabsNavigationRenderItem, TabsNavigationRenderModel, model::TabsNavigationWidthMode};
use crate::controls::control_group::{
    ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemHandlerExt, ControlGroupMouseDownHandler,
    ControlGroupMouseUpHandler, ControlGroupRenderModel, ControlGroupTemplate, ControlGroupTemplateHandlers,
};
use crate::controls::tabs_navigation::{TabsNavigationItemLook, TabsNavigationTheme, default_tabs_navigation_theme};

pub type TabsNavigationClickHandler = ControlGroupClickHandler;
pub type TabsNavigationHoverHandler = ControlGroupHoverHandler;
pub type TabsNavigationMouseDownHandler = ControlGroupMouseDownHandler;
pub type TabsNavigationMouseUpHandler = ControlGroupMouseUpHandler;
pub type TabsNavigationTemplateHandlers = ControlGroupTemplateHandlers;

pub type TabsNavigationTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

struct TabsNavigationItemVisualModel<'a> {
    id: ElementId,
    label: &'a SharedString,
    state: crate::controls::state::CompositeItemState,
}

pub trait TabsNavigationTemplate: Send + Sync {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTabsNavigationTemplate {
    theme: Arc<dyn TabsNavigationTheme>,
    modifiers: Vec<TabsNavigationTemplateModifier>,
}

impl ThemedTabsNavigationTemplate {
    pub fn new(theme: Arc<dyn TabsNavigationTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TabsNavigationRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedTabsNavigationTemplate {
    base: Arc<dyn TabsNavigationTemplate>,
    modifiers: Vec<TabsNavigationTemplateModifier>,
}

impl ModifiedTabsNavigationTemplate {
    fn new(base: Arc<dyn TabsNavigationTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TabsNavigationTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TabsNavigationRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_tabs_navigation_template() -> Arc<dyn TabsNavigationTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TabsNavigationTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedTabsNavigationTemplate::new(default_tabs_navigation_theme())))
        .clone()
}

pub(super) fn template_with_modifier<F>(
    template: Arc<dyn TabsNavigationTemplate>,
    modifier: F,
) -> Arc<dyn TabsNavigationTemplate>
where
    F: Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTabsNavigationTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl TabsNavigationTemplate for ModifiedTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl TabsNavigationTemplate for ThemedTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, model.size);
        let uniform_width = resolve_uniform_tab_width(model, self.theme.as_ref(), window);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(list_look.gap))
            .p(px(list_look.padding))
            .rounded(px(list_look.radius));

        if let Some(background) = list_look.background {
            root = root.bg(background);
        }

        if let Some(border) = list_look.border {
            root = root.border_1().border_color(border);
        }

        for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
            let look = self.theme.resolve_item(item.active, item.state.interaction_state(), model.size);
            let mut tab = render_tabs_navigation_item_visual(
                TabsNavigationItemVisualModel {
                    id: ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("tab-{}", item.id).into()),
                    label: item.label,
                    state: item.state,
                },
                look,
            )
            .control_group_item_handlers(item_handlers);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            root = root.child(tab);
        }

        self.apply_modifiers(root, model)
    }
}

fn resolve_uniform_tab_width(
    model: &TabsNavigationRenderModel<'_>,
    theme: &dyn TabsNavigationTheme,
    window: &mut Window,
) -> Option<f32> {
    if model.width_mode != TabsNavigationWidthMode::Uniform {
        return None;
    }

    let font_family = theme.font_family();
    let mut max_width = 0.0_f32;

    for item in &model.items {
        let look = theme.resolve_item(item.active, item.state.interaction_state(), model.size);
        let run = TextRun {
            len: item.label.len(),
            font: {
                let mut font = font(font_family.clone());
                font.weight = look.label_typography.weight;
                font
            },
            color: look.label_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(item.label.clone(), px(look.label_typography.size), &[run], None);
        let width = line.x_for_index(item.label.len()).as_f32() + look.padding_x * 2.0;
        max_width = max_width.max(width);
    }

    Some(max_width)
}

fn render_tabs_navigation_item_visual(
    model: TabsNavigationItemVisualModel<'_>,
    look: TabsNavigationItemLook,
) -> Stateful<Div> {
    let mut root = div()
        .id(model.id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .min_h(px(look.height))
        .px(px(look.padding_x))
        .rounded(px(look.radius))
        .text_color(look.label_color)
        .text_size(px(look.label_typography.size))
        .line_height(px(look.label_typography.line_height))
        .font_weight(look.label_typography.weight)
        .child(model.label.clone());

    if let Some(indicator) = look.indicator {
        root = root.child(
            div()
                .absolute()
                .left(px(look.padding_x))
                .right(px(look.padding_x))
                .bottom(px(0.0))
                .h(px(look.indicator_height))
                .rounded(px(look.indicator_height))
                .bg(indicator),
        );
    }

    if model.state.disabled {
        root = root.opacity(0.56);
    }

    root
}

pub(crate) fn tabs_navigation_control_group_template(
    size: crate::theme::ControlSize,
    width_mode: TabsNavigationWidthMode,
    template: Arc<dyn TabsNavigationTemplate>,
) -> ControlGroupTemplate<TabsNavigationItem> {
    Arc::new(move |model, handlers, window, cx| {
        let tabs_model = tabs_navigation_render_model(model, size, width_mode);
        template.render(&tabs_model, handlers, window, cx)
    })
}

fn tabs_navigation_render_model<'a>(
    model: &'a ControlGroupRenderModel<'a, TabsNavigationItem>,
    size: crate::theme::ControlSize,
    width_mode: TabsNavigationWidthMode,
) -> TabsNavigationRenderModel<'a> {
    TabsNavigationRenderModel {
        id: model.id,
        size,
        width_mode,
        items: model
            .items
            .iter()
            .map(|item| TabsNavigationRenderItem {
                id: item.item.id(),
                label: item.item.label_text(),
                active: item.selected,
                enabled: item.enabled,
                state: item.state,
            })
            .collect(),
        active_id: model.selected_ids.first(),
        enabled: model.enabled,
        focus: model.focus,
    }
}
