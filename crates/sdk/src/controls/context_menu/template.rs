use std::sync::{Arc, OnceLock};

use gpui::{
    App, Bounds, ClickEvent, Corner, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Stateful, Window,
    anchored, deferred, div, px, prelude::*,
};

use super::ContextMenuRenderModel;
use crate::controls::floating_menu::render_floating_menu;
use crate::controls::context_menu::{ContextMenuTheme, default_context_menu_theme};

pub type ContextMenuBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ContextMenuMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct ContextMenuTemplateHandlers {
    pub target_bounds: ContextMenuBoundsHandler,
    pub target_aux_click: ContextMenuClickHandler,
    pub target_hover: ContextMenuHoverHandler,
    pub target_mouse_down: ContextMenuMouseDownHandler,
    pub target_mouse_up: ContextMenuMouseUpHandler,
    pub target_mouse_up_out: ContextMenuMouseUpHandler,
    pub root_mouse_down_out: ContextMenuMouseDownHandler,
    pub item_hovers: Vec<ContextMenuHoverHandler>,
    pub item_clicks: Vec<ContextMenuClickHandler>,
}

pub trait ContextMenuTemplate: Send + Sync {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedContextMenuTemplate {
    theme: Arc<dyn ContextMenuTheme>,
}

impl ThemedContextMenuTemplate {
    pub fn new(theme: Arc<dyn ContextMenuTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_context_menu_template() -> Arc<dyn ContextMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ContextMenuTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedContextMenuTemplate::new(default_context_menu_theme())))
        .clone()
}

impl ContextMenuTemplate for ThemedContextMenuTemplate {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ContextMenuTemplateHandlers {
            target_bounds,
            target_aux_click,
            target_hover,
            target_mouse_down,
            target_mouse_up,
            target_mouse_up_out,
            root_mouse_down_out,
            item_hovers,
            item_clicks,
        } = handlers;
        let look = self.theme.resolve(model.state);
        let mut target = div()
            .id(format!("{}-target", model.id))
            .flex()
            .items_center()
            .justify_center()
            .min_w(px(look.target_min_width))
            .px(px(look.target_padding_x))
            .py(px(look.target_padding_y))
            .bg(look.target_background)
            .text_color(look.target_foreground)
            .border_1()
            .border_color(look.target_border)
            .rounded(px(look.target_radius))
            .text_size(px(look.target_typography.size))
            .line_height(px(look.target_typography.line_height))
            .font_weight(look.target_typography.weight)
            .cursor_pointer()
            .on_hover(target_hover)
            .on_mouse_down(MouseButton::Right, target_mouse_down)
            .on_mouse_up(MouseButton::Right, target_mouse_up)
            .on_mouse_up_out(MouseButton::Right, target_mouse_up_out)
            .on_aux_click(target_aux_click)
            .child(model.label.clone());

        if !model.enabled {
            target = target.opacity(0.56);
        }

        if let Some(focus_ring) = look.focus_ring {
            target = target.border_1().border_color(focus_ring);
        }

        let mut root = div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    target_bounds(bounds, window, cx);
                }
            })
            .id(model.id.clone())
            .relative()
            .on_mouse_down_out(root_mouse_down_out)
            .child(target);

        if let Some(position) = model.menu_position {
            let menu = render_floating_menu(
                model.id,
                model.items,
                model.open_submenu,
                model.active_path,
                look.floating_menu,
                item_hovers,
                item_clicks,
            );
            let overlay = anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(Corner::TopLeft)
                .position(position)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        root
    }
}
