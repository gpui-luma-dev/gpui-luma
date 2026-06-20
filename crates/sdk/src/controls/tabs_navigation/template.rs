use std::sync::{Arc, OnceLock};

use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, MouseDownEvent, MouseUpEvent, SharedString, Stateful, TextRun,
    Window, div, font, px, prelude::*,
};

use super::{TabsNavigationRenderModel, model::TabsNavigationWidthMode};
use crate::controls::tabs_navigation::{TabsNavigationItemLook, TabsNavigationTheme, default_tabs_navigation_theme};

pub type TabsNavigationClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type TabsNavigationHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TabsNavigationMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TabsNavigationMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct TabsNavigationTemplateHandlers {
    pub item_hovers: Vec<TabsNavigationHoverHandler>,
    pub item_mouse_downs: Vec<TabsNavigationMouseDownHandler>,
    pub item_mouse_ups: Vec<TabsNavigationMouseUpHandler>,
    pub item_mouse_up_outs: Vec<TabsNavigationMouseUpHandler>,
    pub item_clicks: Vec<TabsNavigationClickHandler>,
}

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
}

impl ThemedTabsNavigationTemplate {
    pub fn new(theme: Arc<dyn TabsNavigationTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_tabs_navigation_template() -> Arc<dyn TabsNavigationTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TabsNavigationTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedTabsNavigationTemplate::new(default_tabs_navigation_theme())))
        .clone()
}

impl TabsNavigationTemplate for ThemedTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let TabsNavigationTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;
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

            let look = self.theme.resolve_item(item.active, item.state.interaction_state(), model.size);
            let mut tab = render_tabs_navigation_item_visual(
                TabsNavigationItemVisualModel {
                    id: ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("tab-{}", item.id).into()),
                    label: item.label,
                    state: item.state,
                },
                look,
            )
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            root = root.child(tab);
        }

        root
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
        let line =
            window
                .text_system()
                .shape_line(item.label.clone(), px(look.label_typography.size), &[run], None);
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
