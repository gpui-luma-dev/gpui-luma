use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Corner, Div, FontWeight, Stateful, Window, anchored, deferred, div, point, prelude::*,
    px, svg,
};
use gpui_luma::controls::context_menu::{ContextMenuRenderModel, ContextMenuTemplate, ContextMenuTemplateHandlers};
use gpui_luma::controls::menu_item::{MenuItem, MenuItemIcon};
use gpui_luma::controls::context_menu::{ContextMenuLook, ContextMenuTheme};
use gpui_luma::controls::floating_menu::FloatingMenuLook;
use lucide_icons::Icon as LucideIcon;

const RADIAL_ITEM_COUNT: usize = 5;
const RADIAL_DIAMETER: f32 = 184.0;
const RADIAL_RADIUS: f32 = RADIAL_DIAMETER * 0.5;
const RADIAL_BUTTON_SIZE: f32 = 42.0;
const RADIAL_BUTTON_RADIUS: f32 = 58.0;
const RADIAL_BACKDROP_DIAMETER: f32 = 164.0;
const RADIAL_BACKDROP_INNER_DIAMETER: f32 = 106.0;
const RADIAL_CENTER_MARKER_SIZE: f32 = 12.0;

type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ContextMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub(in crate::gallery) fn radial_context_menu_template(
    theme: Arc<dyn ContextMenuTheme>,
) -> Arc<dyn ContextMenuTemplate> {
    Arc::new(GalleryRadialContextMenuTemplate { theme })
}

struct GalleryRadialContextMenuTemplate {
    theme: Arc<dyn ContextMenuTheme>,
}

impl ContextMenuTemplate for GalleryRadialContextMenuTemplate {
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
            target_hover: _,
            target_mouse_down: _,
            target_mouse_up: _,
            target_mouse_up_out: _,
            root_mouse_down_out,
            item_hovers,
            item_clicks,
        } = handlers;
        let look = self.theme.resolve(model.state);
        let mut target = render_target(model, &look, target_aux_click);

        if !model.enabled {
            target = target.opacity(0.56);
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
            let radial_menu = render_radial_menu(model, look.floating_menu, item_hovers, item_clicks);
            let overlay = anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(Corner::TopLeft)
                .position(position)
                .offset(point(px(-RADIAL_RADIUS), px(-RADIAL_RADIUS)))
                .child(radial_menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        root
    }
}

fn render_target(
    model: &ContextMenuRenderModel<'_>,
    look: &ContextMenuLook,
    target_aux_click: ContextMenuClickHandler,
) -> Stateful<Div> {
    div()
        .id(format!("{}-target", model.id))
        .w(px(180.0))
        .h(px(128.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .text_color(look.target_foreground)
        .on_aux_click(target_aux_click)
        .child(model.label.clone())
}

fn render_radial_menu(
    model: &ContextMenuRenderModel<'_>,
    look: FloatingMenuLook,
    item_hovers: Vec<ContextMenuHoverHandler>,
    item_clicks: Vec<ContextMenuClickHandler>,
) -> Stateful<Div> {
    let mut item_clicks = item_clicks.into_iter();
    let mut menu = div()
        .id(format!("{}-radial-menu", model.id))
        .relative()
        .size(px(RADIAL_DIAMETER))
        .occlude()
        .child(render_radial_backdrop(&look))
        .child(
            div()
                .absolute()
                .left(px(RADIAL_RADIUS - (RADIAL_CENTER_MARKER_SIZE * 0.5)))
                .top(px(RADIAL_RADIUS - (RADIAL_CENTER_MARKER_SIZE * 0.5)))
                .size(px(RADIAL_CENTER_MARKER_SIZE))
                .rounded(px(RADIAL_CENTER_MARKER_SIZE))
                .bg(with_alpha(look.border, 0.72)),
        );

    for ((index, item), item_hover) in model.items.iter().take(RADIAL_ITEM_COUNT).enumerate().zip(item_hovers) {
        let mut button = render_radial_item(model.id, index, item, &look).on_hover(item_hover);

        if item.is_enabled()
            && item.submenu_items().is_empty()
            && let Some(item_click) = item_clicks.next()
        {
            button = button.on_click(item_click);
        }

        if model.active_path.is_some_and(|active_path| active_path.is_root(index)) {
            button = button.bg(look.item_hover_background).border_color(look.foreground);
        }

        menu = menu.child(button);
    }

    menu
}

fn render_radial_item(
    menu_id: &gpui::SharedString,
    index: usize,
    item: &MenuItem,
    look: &FloatingMenuLook,
) -> Stateful<Div> {
    let (left, top) = radial_item_position(index);
    let foreground = if item.is_enabled() {
        look.foreground
    } else {
        look.item_disabled_foreground
    };

    let mut button = div()
        .id(format!("{}-radial-item-{}", menu_id, item.id()))
        .absolute()
        .left(px(left))
        .top(px(top))
        .size(px(RADIAL_BUTTON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .bg(look.background)
        .border_1()
        .border_color(look.border)
        .rounded(px(RADIAL_BUTTON_SIZE))
        .shadow(look.shadow.clone())
        .text_color(foreground)
        .child(render_item_icon(item.icon_ref(), foreground, look.item_icon_size));

    if item.is_enabled() && item.submenu_items().is_empty() {
        button = button
            .cursor_pointer()
            .hover(move |style| style.bg(look.item_hover_background).border_color(look.foreground));
    } else {
        button = button.opacity(0.56);
    }

    button
}

fn render_radial_backdrop(look: &FloatingMenuLook) -> Stateful<Div> {
    let outer_offset = (RADIAL_DIAMETER - RADIAL_BACKDROP_DIAMETER) * 0.5;
    let inner_offset = (RADIAL_DIAMETER - RADIAL_BACKDROP_INNER_DIAMETER) * 0.5;

    div()
        .id("context-menu-radial-backdrop")
        .absolute()
        .left(px(outer_offset))
        .top(px(outer_offset))
        .size(px(RADIAL_BACKDROP_DIAMETER))
        .rounded(px(RADIAL_BACKDROP_DIAMETER))
        .bg(with_alpha(look.background, 0.58))
        .border_1()
        .border_color(with_alpha(look.border, 0.44))
        .child(
            div()
                .absolute()
                .left(px(inner_offset - outer_offset))
                .top(px(inner_offset - outer_offset))
                .size(px(RADIAL_BACKDROP_INNER_DIAMETER))
                .rounded(px(RADIAL_BACKDROP_INNER_DIAMETER))
                .bg(with_alpha(look.background, 0.32)),
        )
}

fn with_alpha(color: gpui::Hsla, alpha: f32) -> gpui::Hsla {
    gpui::Hsla { a: alpha, ..color }
}

fn radial_item_position(index: usize) -> (f32, f32) {
    let angle = match index {
        0 => -90.0_f32,
        1 => -18.0,
        2 => 54.0,
        3 => 126.0,
        _ => 198.0,
    }
    .to_radians();
    let center = RADIAL_RADIUS - (RADIAL_BUTTON_SIZE * 0.5);

    (center + (angle.cos() * RADIAL_BUTTON_RADIUS), center + (angle.sin() * RADIAL_BUTTON_RADIUS))
}

fn render_item_icon(icon: Option<&MenuItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element()
    } else {
        render_lucide_icon(LucideIcon::Circle, color, size)
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
