use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Corner, Div, FontWeight, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    Point, Size, Stateful, Window, anchored, deferred, div, point, px, prelude::*,
};
use lucide_icons::Icon as LucideIcon;

use super::{PopupMenuPlacement, PopupMenuRenderModel};
use crate::controls::floating_menu::render_floating_menu;
use crate::controls::popup_menu::{PopupMenuAppearance, PopupMenuTheme, default_popup_menu_theme};

pub type PopupMenuBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type PopupMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type PopupMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type PopupMenuMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type PopupMenuMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct PopupMenuTemplateHandlers {
    pub trigger_bounds: PopupMenuBoundsHandler,
    pub trigger_click: PopupMenuClickHandler,
    pub trigger_hover: PopupMenuHoverHandler,
    pub trigger_mouse_down: PopupMenuMouseDownHandler,
    pub trigger_mouse_up: PopupMenuMouseUpHandler,
    pub trigger_mouse_up_out: PopupMenuMouseUpHandler,
    pub root_mouse_down_out: PopupMenuMouseDownHandler,
    pub item_hovers: Vec<PopupMenuHoverHandler>,
    pub item_clicks: Vec<PopupMenuClickHandler>,
}

pub trait PopupMenuTemplate: Send + Sync {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedPopupMenuTemplate {
    theme: Arc<dyn PopupMenuTheme>,
}

impl ThemedPopupMenuTemplate {
    pub fn new(theme: Arc<dyn PopupMenuTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_popup_menu_template() -> Arc<dyn PopupMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn PopupMenuTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedPopupMenuTemplate::new(default_popup_menu_theme()))).clone()
}

impl PopupMenuTemplate for ThemedPopupMenuTemplate {
    fn render(
        &self,
        model: &PopupMenuRenderModel<'_>,
        handlers: PopupMenuTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let PopupMenuTemplateHandlers {
            trigger_bounds,
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
            .min_w(px(appearance.floating_menu.min_width))
            .bg(appearance.trigger_background)
            .text_color(appearance.trigger_foreground)
            .border_1()
            .border_color(appearance.trigger_border)
            .rounded(px(appearance.trigger_radius))
            .text_size(px(appearance.trigger_typography.size))
            .line_height(px(appearance.trigger_typography.line_height))
            .font_weight(appearance.trigger_typography.weight)
            .cursor_pointer()
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click)
            .child(model.label.clone())
            .child(render_lucide_icon(
                if model.open {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                },
                appearance.trigger_foreground,
                appearance.trigger_icon_size,
            ));

        if model.state.disabled {
            trigger = trigger.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            trigger = trigger.border_1().border_color(focus_ring);
        }

        let mut root = div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    trigger_bounds(bounds, window, cx);
                }
            })
            .id(model.id.clone())
            .flex()
            .flex_col()
            .items_stretch()
            .relative()
            .on_mouse_down_out(root_mouse_down_out)
            .child(trigger);

        if model.open {
            let placement = resolve_popup_menu_placement(
                model.trigger_bounds,
                model.placement,
                &appearance,
                model.items.len(),
                window.viewport_size(),
            );
            let menu = render_floating_menu(
                model.id,
                model.items,
                model.open_submenu,
                model.active_path,
                appearance.floating_menu,
                item_hovers,
                item_clicks,
            );
            let overlay = anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(placement.anchor)
                .position(placement.position)
                .offset(placement.offset)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        root
    }
}

#[derive(Clone, Copy, Debug)]
struct ResolvedPopupMenuPlacement {
    anchor: Corner,
    position: Point<Pixels>,
    offset: Point<Pixels>,
}

fn resolve_popup_menu_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: PopupMenuPlacement,
    appearance: &PopupMenuAppearance,
    item_count: usize,
    viewport_size: Size<Pixels>,
) -> ResolvedPopupMenuPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(
            point(px(0.0), px(0.0)),
            Size { width: px(appearance.floating_menu.min_width), height: px(appearance.trigger_height) },
        )
    });
    let menu_size = estimated_menu_size(appearance, item_count, trigger_bounds.size.width);
    let offset_y = px(appearance.menu_offset_y);
    let resolved = match placement {
        PopupMenuPlacement::Smart => {
            let viewport_bottom = viewport_size.height - px(8.0);
            if trigger_bounds.bottom() + offset_y + menu_size.height <= viewport_bottom {
                PopupMenuPlacement::BelowStart
            } else {
                PopupMenuPlacement::AboveStart
            }
        }
        placement => placement,
    };

    match resolved {
        PopupMenuPlacement::Smart | PopupMenuPlacement::BelowStart => ResolvedPopupMenuPlacement {
            anchor: Corner::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
        },
        PopupMenuPlacement::AboveStart => ResolvedPopupMenuPlacement {
            anchor: Corner::BottomLeft,
            position: point(trigger_bounds.left(), trigger_bounds.top()),
            offset: point(px(0.0), -offset_y),
        },
        PopupMenuPlacement::CenteredOnTrigger => ResolvedPopupMenuPlacement {
            anchor: Corner::TopLeft,
            position: trigger_bounds.center(),
            offset: point(-(menu_size.width * 0.5), -(menu_size.height * 0.5)),
        },
    }
}

fn estimated_menu_size(appearance: &PopupMenuAppearance, item_count: usize, trigger_width: Pixels) -> Size<Pixels> {
    let menu_min_width = px(appearance.floating_menu.min_width);
    Size {
        width: if trigger_width > menu_min_width {
            trigger_width
        } else {
            menu_min_width
        },
        height: px(appearance.floating_menu.padding * 2.0) + px(appearance.floating_menu.item_height) * item_count,
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

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::theme::InteractionState;

    fn appearance() -> PopupMenuAppearance {
        default_popup_menu_theme().resolve(InteractionState::default())
    }

    #[test]
    fn smart_placement_uses_below_when_it_fits() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &appearance,
            3,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::TopLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.bottom()));
        assert_eq!(placement.offset, point(px(0.0), px(appearance.menu_offset_y)));
    }

    #[test]
    fn smart_placement_uses_above_when_below_is_constrained() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(12.0), px(300.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::Smart,
            &appearance,
            4,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::BottomLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.top()));
        assert_eq!(placement.offset, point(px(0.0), -px(appearance.menu_offset_y)));
    }

    #[test]
    fn explicit_above_ignores_available_space() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::AboveStart,
            &appearance,
            3,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::BottomLeft);
        assert_eq!(placement.position, point(trigger.left(), trigger.top()));
    }

    #[test]
    fn centered_placement_anchors_to_trigger_center() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(40.0), px(80.0)), size(px(160.0), px(32.0)));
        let menu_size = estimated_menu_size(&appearance, 5, trigger.size.width);
        let placement = resolve_popup_menu_placement(
            Some(trigger),
            PopupMenuPlacement::CenteredOnTrigger,
            &appearance,
            5,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::TopLeft);
        assert_eq!(placement.position, trigger.center());
        assert_eq!(placement.offset, point(-(menu_size.width * 0.5), -(menu_size.height * 0.5)));
    }
}
