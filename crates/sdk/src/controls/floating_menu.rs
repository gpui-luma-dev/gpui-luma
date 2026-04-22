use gpui::{AnyElement, App, ClickEvent, Div, FontWeight, Stateful, Window, div, px, prelude::*, svg};
use lucide_icons::Icon as LucideIcon;

use crate::controls::menu_item::{MenuItem, MenuItemIcon};
use crate::controls::state::MenuPath;
use crate::theme::FloatingMenuAppearance;

pub(crate) type FloatingMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub(crate) type FloatingMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub(crate) fn render_floating_menu(
    id: &gpui::SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    appearance: FloatingMenuAppearance,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    let mut menu = div()
        .id(format!("{id}-menu"))
        .relative()
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .occlude();
    let mut item_clicks = item_clicks.into_iter();
    let mut submenu = None;

    for ((index, item), item_hover) in items.iter().enumerate().zip(item_hovers) {
        let enabled = item.is_enabled();
        let color = if enabled {
            appearance.foreground
        } else {
            appearance.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{id}-item-{}", item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(color)
            .text_size(px(appearance.item_typography.size))
            .line_height(px(appearance.item_typography.line_height))
            .font_weight(appearance.item_typography.weight)
            .child(render_item_icon(item.icon_ref(), color, appearance.item_icon_size))
            .child(div().flex_1().child(item.label_text().clone()));

        if enabled {
            row = row
                .cursor_pointer()
                .on_hover(item_hover)
                .hover({
                    let hover_background = appearance.item_hover_background;
                    move |style| style.bg(hover_background)
                })
                .child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    appearance.foreground,
                    appearance.item_icon_size,
                ));

            if active_path.is_some_and(|active_path| active_path.is_root(index)) {
                row = row.bg(appearance.item_hover_background);
            }

            if item.submenu_items().is_empty() {
                if let Some(item_click) = item_clicks.next() {
                    row = row.on_click(item_click);
                }
            } else if open_submenu == Some(index) {
                submenu = Some(render_floating_submenu(id, item, &appearance, &mut item_clicks, index, active_path));
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

fn render_floating_submenu(
    menu_id: &gpui::SharedString,
    item: &MenuItem,
    appearance: &FloatingMenuAppearance,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{menu_id}-submenu-{}", item.id()))
        .absolute()
        .top(px(appearance.padding + (index as f32 * appearance.item_height)))
        .left(px(appearance.min_width + appearance.submenu_offset_x))
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .occlude();

    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let enabled = submenu_item.is_enabled();
        let color = if enabled {
            appearance.foreground
        } else {
            appearance.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{menu_id}-submenu-item-{}", submenu_item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(color)
            .text_size(px(appearance.item_typography.size))
            .line_height(px(appearance.item_typography.line_height))
            .font_weight(appearance.item_typography.weight)
            .child(render_item_icon(submenu_item.icon_ref(), color, appearance.item_icon_size))
            .child(div().flex_1().child(submenu_item.label_text().clone()));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(item_click) = item_clicks.next() {
                row = row
                    .cursor_pointer()
                    .hover({
                        let hover_background = appearance.item_hover_background;
                        move |style| style.bg(hover_background)
                    })
                    .on_click(item_click);
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index)) {
                row = row.bg(appearance.item_hover_background);
            }
        } else if !enabled {
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
