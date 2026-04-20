use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, FontWeight, MouseButton, MouseDownEvent, MouseUpEvent, Stateful, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon as LucideIcon;

use super::{
    NavButtonRenderModel, NavItemState, NavLabelRenderModel, NavNodeItemRenderModel, NavNodeRenderModel, NavRenderItem,
    NavViewRenderModel,
};
use crate::controls::state::focus_debug_border;
use crate::theme::ThemeTokens;

pub type NavClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type NavHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type NavMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type NavMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct NavButtonTemplateHandlers {
    pub hover: NavHoverHandler,
    pub mouse_down: NavMouseDownHandler,
    pub mouse_up: NavMouseUpHandler,
    pub mouse_up_out: NavMouseUpHandler,
    pub click: NavClickHandler,
}

pub struct NavNodeTemplateHandlers {
    pub hover: NavHoverHandler,
    pub mouse_down: NavMouseDownHandler,
    pub mouse_up: NavMouseUpHandler,
    pub mouse_up_out: NavMouseUpHandler,
    pub click: NavClickHandler,
}

pub struct NavNodeItemTemplateHandlers {
    pub hover: NavHoverHandler,
    pub mouse_down: NavMouseDownHandler,
    pub mouse_up: NavMouseUpHandler,
    pub mouse_up_out: NavMouseUpHandler,
    pub click: NavClickHandler,
}

pub enum NavRenderItemTemplateHandlers {
    Button(NavButtonTemplateHandlers),
    Label,
    Node { node: NavNodeTemplateHandlers, children: Vec<NavNodeItemTemplateHandlers> },
}

pub struct NavViewTemplateHandlers {
    pub items: Vec<NavRenderItemTemplateHandlers>,
    pub bottom_buttons: Vec<NavButtonTemplateHandlers>,
}

pub trait NavViewTemplate: Send + Sync {
    fn render(
        &self,
        model: &NavViewRenderModel<'_>,
        handlers: NavViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub trait NavItemTemplate: Send + Sync {
    fn render_button(
        &self,
        button: &NavButtonRenderModel<'_>,
        handlers: NavButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_label(&self, label: &NavLabelRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;

    fn render_node(
        &self,
        node: &NavNodeRenderModel<'_>,
        handlers: NavNodeTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_node_item(
        &self,
        item: &NavNodeItemRenderModel<'_>,
        handlers: NavNodeItemTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedNavViewTemplate;

impl ThemedNavViewTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemedNavViewTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_nav_view_template() -> Arc<dyn NavViewTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn NavViewTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedNavViewTemplate)).clone()
}

impl NavViewTemplate for ThemedNavViewTemplate {
    fn render(
        &self,
        model: &NavViewRenderModel<'_>,
        handlers: NavViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let NavViewTemplateHandlers { items, bottom_buttons } = handlers;
        let mut item_handlers = items.into_iter();
        let mut bottom_buttons = bottom_buttons.into_iter();
        let mut main = div().flex().flex_col().gap(px(2.0)).flex_1().min_h(px(0.0));

        for item in &model.items {
            match (item, item_handlers.next()) {
                (NavRenderItem::Button(button), Some(NavRenderItemTemplateHandlers::Button(handlers))) => {
                    main = main.child(model.item_template.render_button(button, handlers, window, cx));
                }
                (NavRenderItem::Label(label), Some(NavRenderItemTemplateHandlers::Label)) => {
                    main = main.child(model.item_template.render_label(label, window, cx));
                }
                (
                    NavRenderItem::Node(node),
                    Some(NavRenderItemTemplateHandlers::Node { node: node_handlers, children }),
                ) => {
                    main = main.child(model.item_template.render_node(node, node_handlers, window, cx));

                    if node.expanded {
                        let mut child_handlers = children.into_iter();
                        for child in &node.children {
                            if let Some(handlers) = child_handlers.next() {
                                main = main.child(model.item_template.render_node_item(child, handlers, window, cx));
                            }
                        }
                    }
                }
                _ => {
                    debug_assert!(false, "nav render model and handlers are out of sync");
                }
            }
        }

        let mut root = div().id(model.id.clone()).size_full().flex().flex_col().gap(px(8.0)).p(px(8.0)).child(main);

        if !model.bottom_items.is_empty() {
            let mut bottom = div().flex().flex_col().gap(px(2.0)).pt(px(8.0));

            for button in &model.bottom_items {
                if let Some(handlers) = bottom_buttons.next() {
                    bottom = bottom.child(model.item_template.render_button(button, handlers, window, cx));
                }
            }

            root = root.child(bottom);
        }

        root
    }
}

pub struct ThemedNavItemTemplate {
    tokens: ThemeTokens,
}

impl ThemedNavItemTemplate {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl Default for ThemedNavItemTemplate {
    fn default() -> Self {
        Self::new(ThemeTokens::default())
    }
}

pub fn default_nav_item_template() -> Arc<dyn NavItemTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn NavItemTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedNavItemTemplate::default())).clone()
}

impl NavItemTemplate for ThemedNavItemTemplate {
    fn render_button(
        &self,
        button: &NavButtonRenderModel<'_>,
        handlers: NavButtonTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        self.render_action_row(
            ActionRow {
                id: button.id,
                label: button.label,
                depth: button.depth,
                enabled: button.enabled,
                state: button.state,
                disclosure: None,
            },
            RowHandlers::Button(handlers),
        )
    }

    fn render_label(&self, label: &NavLabelRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let colors = &self.tokens.colors;

        div()
            .id(format!("nav-label-{}", label.label))
            .min_h(px(20.0))
            .px(px(depth_padding(label.depth)))
            .pt(px(8.0))
            .text_size(px(11.0))
            .line_height(px(14.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(colors.text_disabled)
            .child(label.label.clone())
    }

    fn render_node(
        &self,
        node: &NavNodeRenderModel<'_>,
        handlers: NavNodeTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        self.render_action_row(
            ActionRow {
                id: node.id,
                label: node.label,
                depth: node.depth,
                enabled: node.enabled,
                state: node.state,
                disclosure: Some(if node.expanded {
                    LucideIcon::ChevronDown
                } else {
                    LucideIcon::ChevronRight
                }),
            },
            RowHandlers::Node(handlers),
        )
    }

    fn render_node_item(
        &self,
        item: &NavNodeItemRenderModel<'_>,
        handlers: NavNodeItemTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        self.render_action_row(
            ActionRow {
                id: item.id,
                label: item.label,
                depth: item.depth,
                enabled: item.enabled,
                state: item.state,
                disclosure: None,
            },
            RowHandlers::NodeItem(handlers),
        )
    }
}

impl ThemedNavItemTemplate {
    fn render_action_row(&self, model: ActionRow<'_>, handlers: RowHandlers) -> Stateful<Div> {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics.md;
        let icon_size = 16.0;
        let background = if model.state.selected && model.state.pressed {
            Some(colors.selected_pressed)
        } else if model.state.selected && model.state.hovered {
            Some(colors.selected_hover)
        } else if model.state.selected {
            Some(colors.selected)
        } else if model.state.pressed {
            Some(colors.surface_pressed)
        } else if model.state.hovered || model.state.active {
            Some(colors.surface_hover)
        } else {
            None
        };
        let foreground = if !model.enabled {
            colors.text_disabled
        } else if model.state.selected {
            colors.text_inverse
        } else {
            colors.text
        };
        let mut row = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .gap(px(metrics.gap))
            .min_h(px(30.0))
            .px(px(depth_padding(model.depth)))
            .rounded(px(metrics.radius))
            .text_size(px(13.0))
            .line_height(px(18.0))
            .text_color(foreground)
            .child(match model.disclosure {
                Some(icon) => render_lucide_icon(icon, foreground, icon_size),
                None if model.depth > 0 => div().size(px(icon_size)).into_any_element(),
                None => div().size(px(0.0)).into_any_element(),
            })
            .child(div().flex_1().child(model.label.clone()));

        if let Some(background) = background {
            row = row.bg(background);
        }

        if model.enabled {
            row = apply_handlers(row.cursor_pointer(), handlers);
        } else {
            row = row.opacity(0.56);
        }

        if model.state.focus_visible {
            row = row.border_1().border_color(focus_debug_border());
        }

        row
    }
}

struct ActionRow<'a> {
    id: &'a gpui::SharedString,
    label: &'a gpui::SharedString,
    depth: usize,
    enabled: bool,
    state: NavItemState,
    disclosure: Option<LucideIcon>,
}

enum RowHandlers {
    Button(NavButtonTemplateHandlers),
    Node(NavNodeTemplateHandlers),
    NodeItem(NavNodeItemTemplateHandlers),
}

fn apply_handlers(row: Stateful<Div>, handlers: RowHandlers) -> Stateful<Div> {
    match handlers {
        RowHandlers::Button(handlers) => row
            .on_hover(handlers.hover)
            .on_mouse_down(MouseButton::Left, handlers.mouse_down)
            .on_mouse_up(MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click),
        RowHandlers::Node(handlers) => row
            .on_hover(handlers.hover)
            .on_mouse_down(MouseButton::Left, handlers.mouse_down)
            .on_mouse_up(MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click),
        RowHandlers::NodeItem(handlers) => row
            .on_hover(handlers.hover)
            .on_mouse_down(MouseButton::Left, handlers.mouse_down)
            .on_mouse_up(MouseButton::Left, handlers.mouse_up)
            .on_mouse_up_out(MouseButton::Left, handlers.mouse_up_out)
            .on_click(handlers.click),
    }
}

fn depth_padding(depth: usize) -> f32 {
    8.0 + (depth as f32 * 18.0)
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
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
