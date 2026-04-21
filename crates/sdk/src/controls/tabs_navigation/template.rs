use std::sync::{Arc, OnceLock};

use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, MouseDownEvent, MouseUpEvent, SharedString, Stateful, Window, div,
    px, prelude::*,
};

use super::TabsNavigationRenderModel;
use crate::controls::state::focus_debug_border;
use crate::theme::{TabsNavigationItemAppearance, TabsNavigationTheme, default_tabs_navigation_theme};

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
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let TabsNavigationTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;
        let list_appearance = self.theme.resolve_list(model.enabled);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(list_appearance.gap))
            .p(px(list_appearance.padding))
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

            let appearance = self.theme.resolve_item(item.active, item.state.interaction_state());
            let mut tab = render_tabs_navigation_item_visual(
                TabsNavigationItemVisualModel {
                    id: ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("tab-{}", item.id).into()),
                    label: item.label,
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
                tab = tab.cursor_pointer();
            }

            root = root.child(tab);
        }

        root
    }
}

fn render_focus_ring(color: gpui::Hsla, radius: f32) -> Div {
    div().absolute().size_full().border_1().border_color(color).rounded(px(radius))
}

fn render_tabs_navigation_item_visual(
    model: TabsNavigationItemVisualModel<'_>,
    appearance: TabsNavigationItemAppearance,
) -> Stateful<Div> {
    let mut root = div()
        .id(model.id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .min_h(px(appearance.height))
        .px(px(appearance.padding_x))
        .rounded(px(appearance.radius))
        .text_color(appearance.label_color)
        .text_size(px(appearance.label_typography.size))
        .line_height(px(appearance.label_typography.line_height))
        .font_weight(appearance.label_typography.weight)
        .child(model.label.clone());

    if let Some(background) = appearance.background {
        root = root.bg(background);
    }

    if let Some(indicator) = appearance.indicator {
        root = root.child(
            div()
                .absolute()
                .left(px(appearance.padding_x * 0.5))
                .right(px(appearance.padding_x * 0.5))
                .bottom(px(3.0))
                .h(px(appearance.indicator_height))
                .rounded(px(appearance.indicator_height))
                .bg(indicator),
        );
    }

    if let Some(focus_ring) = appearance.focus_ring {
        root = root.child(render_focus_ring(focus_ring, appearance.radius));
    } else if model.state.active && model.state.focus_visible {
        root = root.child(render_focus_ring(focus_debug_border(), appearance.radius));
    }

    if model.state.disabled {
        root = root.opacity(0.56);
    }

    root
}
