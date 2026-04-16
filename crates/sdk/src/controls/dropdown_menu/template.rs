use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, ClickEvent, Div, FontWeight, MouseButton, MouseDownEvent, MouseUpEvent,
    Stateful, Window, deferred, div, px, prelude::*, svg,
};
use lucide_icons::Icon as LucideIcon;

use super::{DropdownMenuItem, DropdownMenuItemIcon, DropdownMenuRenderModel};
use crate::theme::{DropdownMenuTheme, default_dropdown_menu_theme};

pub type DropdownClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type DropdownHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type DropdownMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type DropdownMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct DropdownMenuTemplateHandlers {
    pub trigger_click: DropdownClickHandler,
    pub trigger_hover: DropdownHoverHandler,
    pub trigger_mouse_down: DropdownMouseDownHandler,
    pub trigger_mouse_up: DropdownMouseUpHandler,
    pub trigger_mouse_up_out: DropdownMouseUpHandler,
    pub root_mouse_down_out: DropdownMouseDownHandler,
    pub item_hovers: Vec<DropdownHoverHandler>,
    pub item_clicks: Vec<DropdownClickHandler>,
}

pub trait DropdownMenuTemplate: Send + Sync {
    fn render(
        &self,
        model: &DropdownMenuRenderModel<'_>,
        handlers: DropdownMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedDropdownMenuTemplate {
    theme: Arc<dyn DropdownMenuTheme>,
}

impl ThemedDropdownMenuTemplate {
    pub fn new(theme: Arc<dyn DropdownMenuTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_dropdown_menu_template() -> Arc<dyn DropdownMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DropdownMenuTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| {
            Arc::new(ThemedDropdownMenuTemplate::new(
                default_dropdown_menu_theme(),
            ))
        })
        .clone()
}

impl DropdownMenuTemplate for ThemedDropdownMenuTemplate {
    fn render(
        &self,
        model: &DropdownMenuRenderModel<'_>,
        handlers: DropdownMenuTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let DropdownMenuTemplateHandlers {
            trigger_click,
            trigger_hover,
            trigger_mouse_down,
            trigger_mouse_up,
            trigger_mouse_up_out,
            root_mouse_down_out,
            item_hovers,
            item_clicks,
        } = handlers;
        let appearance = self.theme.resolve(model.state);
        let mut trigger = div()
            .id(format!("{}-trigger", model.id))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(appearance.trigger_gap))
            .px(px(appearance.trigger_padding_x))
            .py(px(appearance.trigger_padding_y))
            .h(px(appearance.trigger_height))
            .min_w(px(appearance.menu_min_width))
            .bg(appearance.trigger_background)
            .text_color(appearance.trigger_foreground)
            .border_1()
            .border_color(appearance.trigger_border)
            .rounded(px(appearance.trigger_radius))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click)
            .child(model.label.clone())
            .child(render_lucide_icon(
                LucideIcon::ChevronDown,
                appearance.trigger_foreground,
                appearance.trigger_icon_size,
            ));

        if model.state.disabled {
            trigger = trigger.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            trigger = trigger.focus_visible(move |style| style.border_color(focus_ring));
        }

        let mut root = div()
            .id(model.id.clone())
            .flex()
            .flex_col()
            .items_stretch()
            .relative()
            .on_mouse_down_out(root_mouse_down_out)
            .child(trigger);

        if model.open {
            let mut menu = div()
                .absolute()
                .top(px(appearance.trigger_height + appearance.menu_offset_y))
                .left(px(0.0))
                .min_w(px(appearance.menu_min_width))
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

            root = root.child(deferred(menu).with_priority(1));
        }

        root
    }
}

fn render_submenu(
    menu_id: &gpui::SharedString,
    item: &DropdownMenuItem,
    appearance: &crate::theme::DropdownMenuAppearance,
    item_clicks: &mut std::vec::IntoIter<DropdownClickHandler>,
    index: usize,
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

    for submenu_item in &item.submenu_items {
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
