use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Corner, Div, FontWeight, Stateful, Window, anchored, deferred, div, px, prelude::*,
    svg,
};
use gpui_luma::controls::context_menu::{
    ContextMenuRenderModel, ContextMenuTemplate, ContextMenuTemplateHandlers, MenuItem, MenuItemIcon, MenuPath,
};
use gpui_luma::theme::{ContextMenuAppearance, ContextMenuTheme, default_context_menu_theme};
use lucide_icons::Icon as LucideIcon;

type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ContextMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub(in crate::gallery) fn gallery_context_menu_template() -> Arc<dyn ContextMenuTemplate> {
    Arc::new(GalleryContextMenuTemplate { theme: default_context_menu_theme() })
}

struct GalleryContextMenuTemplate {
    theme: Arc<dyn ContextMenuTheme>,
}

impl ContextMenuTemplate for GalleryContextMenuTemplate {
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
        let appearance = self.theme.resolve(model.state);
        let mut target = render_gallery_target(model, &appearance, target_aux_click);

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

pub(super) fn render_gallery_target(
    model: &ContextMenuRenderModel<'_>,
    appearance: &ContextMenuAppearance,
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
        .text_color(appearance.target_foreground)
        .on_aux_click(target_aux_click)
        .child(model.label.clone())
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
            .id(format!("{}-item-{}", model.id, item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(if item.is_enabled() {
                appearance.item_foreground
            } else {
                appearance.item_disabled_foreground
            })
            .child(render_item_icon(
                item.icon_ref(),
                if item.is_enabled() {
                    appearance.item_foreground
                } else {
                    appearance.item_disabled_foreground
                },
                appearance.item_icon_size,
            ))
            .child(div().flex_1().child(item.label_text().clone()));

        if item.is_enabled() {
            row = row
                .cursor_pointer()
                .on_hover(item_hover)
                .hover(move |style| style.bg(appearance.item_hover_background))
                .child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    appearance.item_foreground,
                    appearance.item_icon_size,
                ));

            if model.active_path.is_some_and(|active_path| active_path.is_root(index)) {
                row = row.bg(appearance.item_hover_background);
            }

            if item.submenu_items().is_empty() {
                if let Some(item_click) = item_clicks.next() {
                    row = row.on_click(item_click);
                }
            } else if model.open_submenu == Some(index) {
                submenu = Some(render_submenu(model.id, item, &appearance, &mut item_clicks, index, model.active_path));
            }
        } else {
            row = row.opacity(0.56).child(render_submenu_affordance(
                !item.submenu_items().is_empty(),
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
    item: &MenuItem,
    appearance: &ContextMenuAppearance,
    item_clicks: &mut std::vec::IntoIter<ContextMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{}-submenu-{}", menu_id, item.id()))
        .absolute()
        .top(px(appearance.menu_padding + (index as f32 * appearance.item_height)))
        .left(px(appearance.menu_min_width + appearance.submenu_offset_x))
        .min_w(px(appearance.menu_min_width))
        .p(px(appearance.menu_padding))
        .bg(appearance.menu_background)
        .border_1()
        .border_color(appearance.menu_border)
        .rounded(px(appearance.menu_radius))
        .shadow_sm()
        .occlude();

    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let mut row = div()
            .id(format!("{}-submenu-item-{}", menu_id, submenu_item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(if submenu_item.is_enabled() {
                appearance.item_foreground
            } else {
                appearance.item_disabled_foreground
            })
            .child(render_item_icon(
                submenu_item.icon_ref(),
                if submenu_item.is_enabled() {
                    appearance.item_foreground
                } else {
                    appearance.item_disabled_foreground
                },
                appearance.item_icon_size,
            ))
            .child(div().flex_1().child(submenu_item.label_text().clone()));

        if submenu_item.is_enabled() && submenu_item.submenu_items().is_empty() {
            if let Some(item_click) = item_clicks.next() {
                row = row
                    .cursor_pointer()
                    .hover(move |style| style.bg(appearance.item_hover_background))
                    .on_click(item_click);
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index)) {
                row = row.bg(appearance.item_hover_background);
            }
        } else if !submenu_item.is_enabled() {
            row = row.opacity(0.56);
        }

        submenu = submenu.child(row);
    }

    submenu
}

fn render_item_icon(icon: Option<&MenuItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element()
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
