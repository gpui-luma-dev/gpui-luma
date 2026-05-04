use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Corner, Div, FontWeight, MouseButton, MouseDownEvent, MouseUpEvent, Pixels,
    Point, SharedString, Size, Stateful, Window, anchored, deferred, div, point, px, prelude::*, svg,
};
use gpui::Hsla;
use lucide_icons::Icon as LucideIcon;

use super::{SelectorPlacement, SelectorRenderModel};
use crate::controls::selector::model::{
    SelectorItemRenderModel, SelectorItem, SelectorItemIcon, SelectorItemLike,
};
use crate::controls::state::MenuPath;
use crate::theme::{FloatingMenuAppearance, SelectorAppearance, SelectorTheme, default_selector_theme};

pub type SelectorBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SelectorClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectorMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

pub struct SelectorTemplateHandlers {
    pub trigger_bounds: SelectorBoundsHandler,
    pub trigger_click: SelectorClickHandler,
    pub trigger_hover: SelectorHoverHandler,
    pub trigger_mouse_down: SelectorMouseDownHandler,
    pub trigger_mouse_up: SelectorMouseUpHandler,
    pub trigger_mouse_up_out: SelectorMouseUpHandler,
    pub root_mouse_down_out: SelectorMouseDownHandler,
    pub item_hovers: Vec<SelectorHoverHandler>,
    pub item_clicks: Vec<SelectorClickHandler>,
}

pub trait SelectorTemplate<T = SelectorItem>: Send + Sync
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSelectorTemplate {
    theme: Arc<dyn SelectorTheme>,
}

impl ThemedSelectorTemplate {
    pub fn new(theme: Arc<dyn SelectorTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_selector_template<T>() -> Arc<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(ThemedSelectorTemplate::new(default_selector_theme()))
}

impl<T> SelectorTemplate<T> for ThemedSelectorTemplate
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectorTemplateHandlers {
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
        let trigger_content = render_trigger_content(model, &appearance, cx);
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
            .child(div().flex().items_center().gap(px(appearance.trigger_gap)).child(trigger_content))
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
            let placement = resolve_selector_placement(
                model.trigger_bounds,
                model.placement,
                &appearance,
                model.items.len(),
                window.viewport_size(),
            );
            let menu = render_selector_menu(model, appearance.floating_menu, item_hovers, item_clicks, cx);
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
struct ResolvedSelectorPlacement {
    anchor: Corner,
    position: Point<Pixels>,
    offset: Point<Pixels>,
}

fn resolve_selector_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: SelectorPlacement,
    appearance: &SelectorAppearance,
    item_count: usize,
    viewport_size: Size<Pixels>,
) -> ResolvedSelectorPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(
            point(px(0.0), px(0.0)),
            Size { width: px(appearance.floating_menu.min_width), height: px(appearance.trigger_height) },
        )
    });
    let menu_size = estimated_menu_size(appearance, item_count, trigger_bounds.size.width);
    let offset_y = px(appearance.menu_offset_y);
    let resolved = match placement {
        SelectorPlacement::Smart => {
            let viewport_bottom = viewport_size.height - px(8.0);
            if trigger_bounds.bottom() + offset_y + menu_size.height <= viewport_bottom {
                SelectorPlacement::BelowStart
            } else {
                SelectorPlacement::AboveStart
            }
        }
        placement => placement,
    };

    match resolved {
        SelectorPlacement::Smart | SelectorPlacement::BelowStart => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
        },
        SelectorPlacement::AboveStart => ResolvedSelectorPlacement {
            anchor: Corner::BottomLeft,
            position: point(trigger_bounds.left(), trigger_bounds.top()),
            offset: point(px(0.0), -offset_y),
        },
        SelectorPlacement::CenteredOnTrigger => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: trigger_bounds.center(),
            offset: point(-(menu_size.width * 0.5), -(menu_size.height * 0.5)),
        },
        SelectorPlacement::OverlayOnTrigger => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: trigger_bounds.origin,
            offset: point(px(0.0), px(0.0)),
        },
    }
}

fn estimated_menu_size(appearance: &SelectorAppearance, item_count: usize, trigger_width: Pixels) -> Size<Pixels> {
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

fn render_trigger_content<T>(
    model: &SelectorRenderModel<'_, T>,
    appearance: &SelectorAppearance,
    cx: &mut App,
) -> AnyElement
where
    T: SelectorItemLike + 'static,
{
    if let (Some(selected_index), Some(item_template)) = (model.selected_index, model.item_template)
        && let Some(item) = model.items.get(selected_index)
    {
        let active = model.active_path.is_some_and(|path| path.is_root(selected_index));
        let item_model = SelectorItemRenderModel {
            selector_id: model.id,
            item,
            index: selected_index,
            selected: true,
            active,
            open: model.open,
            enabled: model.enabled,
        };
        return item_template(&item_model, cx);
    }

    div()
        .flex()
        .items_center()
        .gap(px(appearance.trigger_gap))
        .child(render_selected_item_icon(
            model.selected_icon,
            appearance.trigger_foreground,
            appearance.trigger_icon_size,
        ))
        .child(model.label.clone())
        .into_any_element()
}

fn render_selector_menu<T>(
    model: &SelectorRenderModel<'_, T>,
    appearance: FloatingMenuAppearance,
    item_hovers: Vec<SelectorHoverHandler>,
    item_clicks: Vec<SelectorClickHandler>,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectorItemLike + 'static,
{
    let mut menu = div()
        .id(format!("{}-menu", model.id))
        .relative()
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .occlude();

    let mut clicks = item_clicks.into_iter();

    for ((index, item), hover) in model.items.iter().enumerate().zip(item_hovers) {
        menu = menu.child(render_selector_row(
            model.id,
            model.id,
            item,
            index,
            model.selected_index,
            model.active_path,
            model.open,
            model.enabled,
            model.item_template,
            &appearance,
            hover,
            clicks.next(),
            cx,
        ));
    }

    menu
}

fn render_selector_row<T>(
    menu_id: &str,
    selector_id: &SharedString,
    item: &T,
    index: usize,
    selected_index: Option<usize>,
    active_path: Option<MenuPath>,
    open: bool,
    enabled: bool,
    item_template: Option<&crate::controls::selector::model::SelectorItemTemplate<T>>,
    appearance: &FloatingMenuAppearance,
    hover: SelectorHoverHandler,
    click: Option<SelectorClickHandler>,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectorItemLike + 'static,
{
    let color = appearance.foreground;
    let selected = selected_index == Some(index);
    let active = active_path.is_some_and(|path| path.is_root(index));
    let content = if let Some(item_template) = item_template {
        let item_model = SelectorItemRenderModel { selector_id, item, index, selected, active, open, enabled };
        item_template(&item_model, cx)
    } else {
        div()
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .child(render_selected_item_icon(item.icon(), color, appearance.item_icon_size))
            .child(div().flex_1().child(item.label_text().clone()))
            .into_any_element()
    };

    let mut row = div()
        .id(format!("{}-item-{}", menu_id, item.id()))
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
        .child(div().flex_1().child(content))
        .child(render_selection_checkmark(selected, color, appearance.item_icon_size));

    row = row.cursor_pointer().on_hover(hover).hover({
        let hover_background = appearance.item_hover_background;
        move |style| style.bg(hover_background)
    });

    if active {
        row = row.bg(appearance.item_hover_background);
    }

    if let Some(click) = click {
        row = row.on_click(click);
    }

    row
}

fn render_selection_checkmark(selected: bool, color: Hsla, size: f32) -> AnyElement {
    if selected {
        render_lucide_icon(LucideIcon::Check, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_selected_item_icon(icon: Option<&SelectorItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(SelectorItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else if let Some(path) = icon.and_then(SelectorItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element()
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

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::theme::InteractionState;

    fn appearance() -> SelectorAppearance {
        default_selector_theme().resolve(InteractionState::default())
    }

    #[test]
    fn smart_placement_uses_below_when_it_fits() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let placement = resolve_selector_placement(
            Some(trigger),
            SelectorPlacement::Smart,
            &appearance,
            3,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::TopLeft);
        assert_eq!(placement.position, point(px(12.0), px(112.0)));
    }

    #[test]
    fn overlay_placement_anchors_to_trigger_origin() {
        let appearance = appearance();
        let trigger = Bounds::new(point(px(30.0), px(70.0)), size(px(150.0), px(30.0)));
        let placement = resolve_selector_placement(
            Some(trigger),
            SelectorPlacement::OverlayOnTrigger,
            &appearance,
            6,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(placement.anchor, Corner::TopLeft);
        assert_eq!(placement.position, point(px(30.0), px(70.0)));
        assert_eq!(placement.offset, point(px(0.0), px(0.0)));
    }
}
