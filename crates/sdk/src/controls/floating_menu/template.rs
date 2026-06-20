use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, FontWeight, SharedString, Stateful, Window, div, px, prelude::*, svg};
use lucide_icons::Icon as LucideIcon;

use crate::controls::menu_item::{MenuItem, MenuItemIcon};
use crate::controls::state::MenuPath;
use crate::controls::floating_menu::FloatingMenuLook;

pub type FloatingMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type FloatingMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub struct FloatingMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [MenuItem],
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub look: FloatingMenuLook,
}

pub struct FloatingMenuTemplateHandlers {
    pub item_hovers: Vec<FloatingMenuHoverHandler>,
    pub item_clicks: Vec<FloatingMenuClickHandler>,
}

pub trait FloatingMenuTemplate: Send + Sync {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div>;
}

pub struct ThemedFloatingMenuTemplate;

impl FloatingMenuTemplate for ThemedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let FloatingMenuTemplateHandlers { item_hovers, item_clicks } = handlers;
        let look = model.look.clone();

        let mut menu = div()
            .id(format!("{}-menu", model.id))
            .relative()
            .min_w(px(look.min_width))
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .occlude();

        let mut item_clicks = item_clicks.into_iter();
        let mut submenu = None;

        for ((index, item), item_hover) in model.items.iter().enumerate().zip(item_hovers) {
            let enabled = item.is_enabled();
            let color = if enabled {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let mut row = div()
                .id(format!("{}-item-{}", model.id, item.id()))
                .flex()
                .items_center()
                .gap(px(look.item_gap))
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(render_item_icon(item.icon_ref(), look.item_icon_size))
                .child(div().flex_1().child(item.label_text().clone()));

            if enabled {
                row = row
                    .cursor_pointer()
                    .on_hover(item_hover)
                    .hover({
                        let hover_background = look.item_hover_background;
                        let hover_foreground = look.item_hover_foreground;
                        move |style| style.bg(hover_background).text_color(hover_foreground)
                    })
                    .child(render_submenu_affordance(!item.submenu_items().is_empty(), look.item_icon_size));

                if model.active_path.is_some_and(|active_path| active_path.is_root(index)) {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if item.submenu_items().is_empty() {
                    if let Some(item_click) = item_clicks.next() {
                        row = row.on_click(item_click);
                    }
                } else if model.open_submenu == Some(index) {
                    submenu = Some(render_floating_submenu(
                        model.id,
                        item,
                        &look,
                        &mut item_clicks,
                        index,
                        model.active_path,
                    ));
                }
            } else {
                row = row
                    .opacity(0.56)
                    .child(render_submenu_affordance(!item.submenu_items().is_empty(), look.item_icon_size));
            }

            menu = menu.child(row);
        }

        if let Some(submenu) = submenu {
            menu = menu.child(submenu);
        }

        menu
    }
}

pub fn default_floating_menu_template() -> Arc<dyn FloatingMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn FloatingMenuTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedFloatingMenuTemplate)).clone()
}

pub fn render_floating_menu(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    render_floating_menu_with_template(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_template(
    template: Arc<dyn FloatingMenuTemplate>,
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    template.render(
        &FloatingMenuRenderModel { id, items, open_submenu, active_path, look },
        FloatingMenuTemplateHandlers { item_hovers, item_clicks },
    )
}

fn render_floating_submenu(
    menu_id: &SharedString,
    item: &MenuItem,
    look: &FloatingMenuLook,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{menu_id}-submenu-{}", item.id()))
        .absolute()
        .top(px(look.padding + (index as f32 * look.item_height)))
        .left(px(look.min_width + look.submenu_offset_x))
        .min_w(px(look.min_width))
        .p(px(look.padding))
        .bg(look.background)
        .border_1()
        .border_color(look.border)
        .rounded(px(look.radius))
        .shadow(look.shadow.clone())
        .occlude();

    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let enabled = submenu_item.is_enabled();
        let color = if enabled {
            look.foreground
        } else {
            look.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{menu_id}-submenu-item-{}", submenu_item.id()))
            .flex()
            .items_center()
            .gap(px(look.item_gap))
            .min_h(px(look.item_height))
            .px(px(look.item_padding_x))
            .rounded(px(look.item_radius))
            .text_color(color)
            .text_size(px(look.item_typography.size))
            .line_height(px(look.item_typography.line_height))
            .font_weight(look.item_typography.weight)
            .child(render_item_icon(submenu_item.icon_ref(), look.item_icon_size))
            .child(div().flex_1().child(submenu_item.label_text().clone()));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(item_click) = item_clicks.next() {
                row = row
                    .cursor_pointer()
                    .hover({
                        let hover_background = look.item_hover_background;
                        let hover_foreground = look.item_hover_foreground;
                        move |style| style.bg(hover_background).text_color(hover_foreground)
                    })
                    .on_click(item_click);
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index)) {
                row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
            }
        } else if !enabled {
            row = row.opacity(0.56);
        }

        submenu = submenu.child(row);
    }

    submenu
}

fn render_item_icon(icon: Option<&MenuItemIcon>, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, size)
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(has_submenu: bool, size: f32) -> AnyElement {
    if has_submenu {
        render_lucide_icon(LucideIcon::ChevronRight, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .child(char::from(icon).to_string())
        .into_any_element()
}
