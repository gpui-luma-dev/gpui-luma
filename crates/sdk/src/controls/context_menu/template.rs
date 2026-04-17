use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Corner, Div, FontWeight, MouseButton, MouseDownEvent,
    MouseUpEvent, Pixels, Stateful, Window, anchored, deferred, div, px, prelude::*, svg,
};
use lucide_icons::Icon as LucideIcon;

use super::{ContextMenuRenderModel, DropdownMenuItem, DropdownMenuItemIcon};
use crate::controls::state::{MenuPath, focus_debug_border};
use crate::theme::{ContextMenuAppearance, ContextMenuTheme, default_context_menu_theme};

pub type ContextMenuBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type ContextMenuMouseDownHandler =
    Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
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
        let appearance = self.theme.resolve(model.state);
        let mut target = div()
            .id(format!("{}-target", model.id))
            .flex()
            .items_center()
            .justify_center()
            .min_w(px(appearance.target_min_width))
            .px(px(appearance.target_padding_x))
            .py(px(appearance.target_padding_y))
            .bg(appearance.target_background)
            .text_color(appearance.target_foreground)
            .border_1()
            .border_color(appearance.target_border)
            .rounded(px(appearance.target_radius))
            .font_weight(FontWeight::MEDIUM)
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

        if model.focus.focused {
            target = target.border_1().border_color(focus_debug_border());
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
            let menu = render_menu(model, appearance, item_hovers, item_clicks);
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

fn render_menu(
    model: &ContextMenuRenderModel<'_>,
    appearance: ContextMenuAppearance,
    item_hovers: Vec<ContextMenuHoverHandler>,
    item_clicks: Vec<ContextMenuClickHandler>,
) -> Stateful<Div> {
    let mut menu = div()
        .id(format!("{}-menu", model.id))
        .min_w(px(appearance.menu_min_width))
        .relative()
        .p(px(appearance.menu_padding))
        .bg(appearance.menu_background)
        .border_1()
        .border_color(appearance.menu_border)
        .rounded(px(appearance.menu_radius))
        .shadow_sm()
        .occlude();
    let mut item_clicks = item_clicks.into_iter();
    let mut submenu = None;

    for ((index, item), item_hover) in model.items.iter().enumerate().zip(item_hovers) {
        let mut row = div()
            .id(format!("{}-item-{}", model.id, item.id))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(if item.enabled {
                appearance.item_foreground
            } else {
                appearance.item_disabled_foreground
            })
            .child(render_item_icon(
                item.icon.as_ref(),
                if item.enabled {
                    appearance.item_foreground
                } else {
                    appearance.item_disabled_foreground
                },
                appearance.item_icon_size,
            ))
            .child(div().flex_1().child(item.label.clone()));

        if item.enabled {
            row = row
                .cursor_pointer()
                .on_hover(item_hover)
                .hover(move |style| style.bg(appearance.item_hover_background))
                .child(render_submenu_affordance(
                    !item.submenu_items.is_empty(),
                    appearance.item_foreground,
                    appearance.item_icon_size,
                ));

            if model
                .active_path
                .is_some_and(|active_path| active_path.is_root(index))
            {
                row = row.bg(appearance.item_hover_background);
            }

            if item.submenu_items.is_empty() {
                if let Some(item_click) = item_clicks.next() {
                    row = row.on_click(item_click);
                }
            } else if model.open_submenu == Some(index) {
                submenu = Some(render_submenu(
                    model.id,
                    item,
                    &appearance,
                    &mut item_clicks,
                    index,
                    model.active_path,
                ));
            }
        } else {
            row = row.opacity(0.56).child(render_submenu_affordance(
                !item.submenu_items.is_empty(),
                appearance.item_disabled_foreground,
                appearance.item_icon_size,
            ));
        }

        menu = menu.child(row);
    }

    if let Some(submenu) = submenu {
        menu = menu.child(submenu);
    }

    menu
}

fn render_submenu(
    menu_id: &gpui::SharedString,
    item: &DropdownMenuItem,
    appearance: &ContextMenuAppearance,
    item_clicks: &mut std::vec::IntoIter<ContextMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{}-submenu-{}", menu_id, item.id))
        .absolute()
        .top(px(
            appearance.menu_padding + (index as f32 * appearance.item_height)
        ))
        .left(px(appearance.menu_min_width + appearance.submenu_offset_x))
        .min_w(px(appearance.menu_min_width))
        .p(px(appearance.menu_padding))
        .bg(appearance.menu_background)
        .border_1()
        .border_color(appearance.menu_border)
        .rounded(px(appearance.menu_radius))
        .shadow_sm()
        .occlude();

    for (submenu_index, submenu_item) in item.submenu_items.iter().enumerate() {
        let mut row = div()
            .id(format!("{}-submenu-item-{}", menu_id, submenu_item.id))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(if submenu_item.enabled {
                appearance.item_foreground
            } else {
                appearance.item_disabled_foreground
            })
            .child(render_item_icon(
                submenu_item.icon.as_ref(),
                if submenu_item.enabled {
                    appearance.item_foreground
                } else {
                    appearance.item_disabled_foreground
                },
                appearance.item_icon_size,
            ))
            .child(div().flex_1().child(submenu_item.label.clone()));

        if submenu_item.enabled && submenu_item.submenu_items.is_empty() {
            if let Some(item_click) = item_clicks.next() {
                row = row
                    .cursor_pointer()
                    .hover(move |style| style.bg(appearance.item_hover_background))
                    .on_click(item_click);
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index)) {
                row = row.bg(appearance.item_hover_background);
            }
        } else if !submenu_item.enabled {
            row = row.opacity(0.56);
        }

        submenu = submenu.child(row);
    }

    submenu
}

fn render_item_icon(
    icon: Option<&DropdownMenuItemIcon>,
    color: gpui::Hsla,
    size: f32,
) -> AnyElement {
    if let Some(icon) = icon.and_then(DropdownMenuItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else if let Some(path) = icon.and_then(DropdownMenuItemIcon::svg_path) {
        svg()
            .external_path(path.clone())
            .size(px(size))
            .text_color(color)
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(has_submenu: bool, color: gpui::Hsla, size: f32) -> AnyElement {
    if has_submenu {
        render_lucide_icon(LucideIcon::ChevronRight, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
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
