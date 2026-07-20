use std::sync::Arc;

use gpui::{
    App, Bounds, ClickEvent, Corner, Div, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Size, Stateful,
    Window, anchored, deferred, div, point, px, prelude::*,
};
use lucide_icons::Icon as LucideIcon;

use super::{SelectorPlacement, SelectorRenderModel};
use super::item_template::render_item_content;

use crate::controls::button_family::button_family_effective_border;
use crate::controls::choice_indicator_layout::reserve_shadow_extent;
use crate::controls::color::style::ElementExt;
use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::{
    SelectorItem, SelectorItemLike, SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers,
    default_selector_items_template,
};
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

use super::theme::{SelectorLook, SelectorTheme, default_selector_theme};

pub type SelectorBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SelectorClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectorMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type SelectorItemHoverHandler = Arc<dyn Fn(usize, &bool, &mut Window, &mut App) + 'static>;
pub type SelectorItemMouseDownHandler = Arc<dyn Fn(usize, &MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SelectorItemClickHandler = Arc<dyn Fn(usize, &ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorTemplateModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct SelectorTemplateHandlers {
    pub trigger_bounds: SelectorBoundsHandler,
    pub trigger_click: SelectorClickHandler,
    pub trigger_hover: SelectorHoverHandler,
    pub trigger_mouse_down: SelectorMouseDownHandler,
    pub trigger_mouse_up: SelectorMouseUpHandler,
    pub trigger_mouse_up_out: SelectorMouseUpHandler,
    pub root_mouse_down_out: SelectorMouseDownHandler,
    pub on_item_hover: SelectorItemHoverHandler,
    pub on_item_mouse_down: SelectorItemMouseDownHandler,
    pub on_item_click: SelectorItemClickHandler,
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

impl Default for SelectorTemplateHandlers {
    fn default() -> Self {
        Self {
            trigger_bounds: Box::new(noop_bounds),
            trigger_click: Box::new(noop_click),
            trigger_hover: Box::new(noop_hover),
            trigger_mouse_down: Box::new(noop_mouse_down),
            trigger_mouse_up: Box::new(noop_mouse_up),
            trigger_mouse_up_out: Box::new(noop_mouse_up),
            root_mouse_down_out: Box::new(noop_mouse_down),
            on_item_hover: Arc::new(|_, _, _, _| {}),
            on_item_mouse_down: Arc::new(|_, _, _, _| {}),
            on_item_click: Arc::new(|_, _, _, _| {}),
        }
    }
}

pub trait SelectorTemplate<T = SelectorItem>: Send + Sync
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, cx: &mut App) -> SelectorLook;

    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSelectorTemplate<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    theme: Arc<dyn SelectorTheme>,
    items_template: Arc<dyn SelectorItemsTemplate<T>>,
    modifiers: Vec<SelectorTemplateModifier<T>>,
}

struct ModifiedSelectorTemplate<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    base: Arc<dyn SelectorTemplate<T>>,
    modifiers: Vec<SelectorTemplateModifier<T>>,
}

impl<T> ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new(theme: Arc<dyn SelectorTheme>, items_template: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        Self { theme, items_template, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

impl<T> ModifiedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn new(base: Arc<dyn SelectorTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: SelectorTemplateModifier<T>) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }

    fn into_arc(self) -> Arc<dyn SelectorTemplate<T>> {
        Arc::new(self)
    }
}

pub fn default_selector_template<T>() -> Arc<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(ThemedSelectorTemplate::new(default_selector_theme(), default_selector_items_template::<T>()))
}

pub(super) fn modified_selector_template<T, F>(
    template: Arc<dyn SelectorTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectorTemplate<T>>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectorRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedSelectorTemplate::new(template).with_modifier(Box::new(modifier)).into_arc()
}

impl<T> SelectorTemplate<T> for ModifiedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, cx: &mut App) -> SelectorLook {
        self.base.resolve_look(model, window, cx)
    }

    fn render(
        &self,
        model: &SelectorRenderModel<'_, T>,
        handlers: SelectorTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl<T> SelectorTemplate<T> for ThemedSelectorTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn resolve_look(&self, model: &SelectorRenderModel<'_, T>, window: &Window, cx: &mut App) -> SelectorLook {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(model.size, metrics, scale_factor),
        );
        self.theme
            .resolve_look(model.trigger_style, model.state, model.size, &scale, model.without_elevation)
    }

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
            on_item_hover,
            on_item_mouse_down,
            on_item_click,
        } = handlers;
        let scale_factor = window.scale_factor();
        let look = self.resolve_look(model, window, cx);
        let border = button_family_effective_border(look.trigger_border);
        let trigger_content = render_item_content(model, &look, cx);
        let mut root = div()
            .id(model.id.clone())
            .flex()
            .items_center()
            .justify_between()
            .gap(px(look.trigger_gap))
            .px(px(look.trigger_padding_x))
            .py(px(look.trigger_padding_y))
            .h(px(look.trigger_height))
            .w_full()
            .bg(look.trigger_background)
            .rounded(px(look.trigger_radius))
            .cursor_pointer()
            .relative()
            .on_prepaint(move |bounds, window, cx| {
                trigger_bounds(&bounds, window, cx);
            })
            .on_hover(trigger_hover)
            .on_mouse_down(MouseButton::Left, trigger_mouse_down)
            .on_mouse_up(MouseButton::Left, trigger_mouse_up)
            .on_mouse_up_out(MouseButton::Left, trigger_mouse_up_out)
            .on_click(trigger_click)
            .on_mouse_down_out(root_mouse_down_out)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(look.trigger_gap))
                    .min_w(px(0.0))
                    .flex_1()
                    .text_color(look.trigger_foreground)
                    .text_size(px(look.trigger_typography.size))
                    .line_height(px(look.trigger_typography.line_height))
                    .font_weight(look.trigger_typography.weight)
                    .child(trigger_content),
            )
            .child(div().flex().items_center().justify_center().flex_shrink_0().text_color(look.trigger_icon).child(
                lucide_icon(
                    if model.open {
                        LucideIcon::ChevronUp
                    } else {
                        LucideIcon::ChevronDown
                    },
                    look.trigger_icon,
                    look.trigger_icon_size,
                ),
            ));

        if border.a > 0.0 {
            root = root.border_1().border_color(border);
        }

        let paint_shadow = !model.state.disabled
            && !model.without_elevation
            && look.trigger_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty());
        if paint_shadow && let Some(shadows) = look.trigger_shadow.as_ref() {
            root = root.shadow(shadows.clone());
        }

        if model.state.disabled {
            root = root.opacity(0.56);
        }

        if let Some(focus_ring) = look.focus_ring {
            root = root.border_1().border_color(focus_ring);
        }

        let has_elevation = !model.without_elevation && look.trigger_shadow.as_ref().is_some_and(|s| !s.is_empty());
        let shadow_extent = reserve_shadow_extent(look.trigger_shadow.as_ref(), None, scale_factor, has_elevation);
        if shadow_extent > 0.0 {
            root = div().id(format!("{}-elevation", model.id)).relative().w_full().p(px(shadow_extent)).child(root);
        }

        if model.open {
            let popup_metrics = resolve_selector_popup_metrics(
                model.trigger_bounds,
                model.placement,
                &look,
                model.items.len(),
                window.viewport_size(),
            );
            let panel_template = model.panel_template.unwrap_or(self.items_template.as_ref());
            let item_hovers = (0..model.items.len())
                .map(|model_index| {
                    let on_item_hover = on_item_hover.clone();
                    Box::new(move |hovered: &bool, window: &mut Window, cx: &mut App| {
                        on_item_hover(model_index, hovered, window, cx);
                    }) as SelectorHoverHandler
                })
                .collect::<Vec<_>>();
            let item_mouse_downs = (0..model.items.len())
                .map(|model_index| {
                    let on_item_mouse_down = on_item_mouse_down.clone();
                    Box::new(move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on_item_mouse_down(model_index, event, window, cx);
                    }) as crate::controls::selector_panel::SelectorPanelMouseDownHandler
                })
                .collect::<Vec<_>>();
            let item_clicks = (0..model.items.len())
                .map(|model_index| {
                    let on_item_click = on_item_click.clone();
                    Box::new(move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
                        on_item_click(model_index, event, window, cx);
                    }) as SelectorClickHandler
                })
                .collect::<Vec<_>>();
            let menu = panel_template.render(
                &SelectorItemsRenderModel {
                    menu_id: model.id,
                    selector_id: model.id,
                    items: model.items,
                    selected_index: model.selected_index,
                    active_path: model.active_path,
                    open: model.open,
                    enabled: model.enabled,
                    focus: model.focus,
                    item_template: model.item_template,
                    look: look.items_panel.clone(),
                    max_height: popup_metrics.max_height,
                    scrolling: popup_metrics.scrolling,
                },
                SelectorItemsTemplateHandlers { item_hovers, item_mouse_downs, item_clicks },
                cx,
            );
            let overlay = anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(popup_metrics.anchor)
                .position(popup_metrics.position)
                .offset(popup_metrics.offset)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        self.apply_modifiers(root, model)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedSelectorPlacement {
    anchor: Corner,
    position: Point<Pixels>,
    offset: Point<Pixels>,
    max_height: Pixels,
    scrolling: bool,
}

pub(crate) fn resolve_selector_popup_metrics(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: SelectorPlacement,
    look: &SelectorLook,
    item_count: usize,
    viewport_size: Size<Pixels>,
) -> ResolvedSelectorPlacement {
    let trigger_bounds = trigger_bounds.unwrap_or_else(|| {
        Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(look.trigger_height) })
    });
    let menu_size = estimated_menu_size(look, item_count, trigger_bounds.size.width);
    let offset_y = px(look.menu_offset_y);
    let viewport_margin = px(8.0);
    let viewport_top = viewport_margin;
    let viewport_bottom = viewport_size.height - viewport_margin;
    let available_below = (viewport_bottom - (trigger_bounds.bottom() + offset_y)).max(px(0.0));
    let available_above = (trigger_bounds.top() - offset_y - viewport_top).max(px(0.0));
    let minimum_height = px((look.items_panel.padding * 2.0) + look.items_panel.item_height);
    let resolved = match placement {
        SelectorPlacement::Smart => {
            if trigger_bounds.bottom() + offset_y + menu_size.height <= viewport_bottom {
                SelectorPlacement::BelowStart
            } else if trigger_bounds.top() - offset_y - menu_size.height >= viewport_top {
                SelectorPlacement::AboveStart
            } else if available_below >= available_above {
                SelectorPlacement::BelowStart
            } else {
                SelectorPlacement::AboveStart
            }
        }
        placement => placement,
    };

    let available_height = match resolved {
        SelectorPlacement::Smart | SelectorPlacement::BelowStart => available_below,
        SelectorPlacement::AboveStart => available_above,
        SelectorPlacement::CenteredOnTrigger => viewport_size.height - (viewport_margin * 2.0),
        SelectorPlacement::OverlayOnTrigger => viewport_bottom - trigger_bounds.top(),
    }
    .max(minimum_height);
    let max_height = menu_size.height.min(available_height);
    let scrolling = menu_size.height > max_height;

    match resolved {
        SelectorPlacement::Smart | SelectorPlacement::BelowStart => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
            max_height,
            scrolling,
        },
        SelectorPlacement::AboveStart => ResolvedSelectorPlacement {
            anchor: Corner::BottomLeft,
            position: point(trigger_bounds.left(), trigger_bounds.top()),
            offset: point(px(0.0), -offset_y),
            max_height,
            scrolling,
        },
        SelectorPlacement::CenteredOnTrigger => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: trigger_bounds.center(),
            offset: point(-(menu_size.width * 0.5), -(max_height * 0.5)),
            max_height,
            scrolling,
        },
        SelectorPlacement::OverlayOnTrigger => ResolvedSelectorPlacement {
            anchor: Corner::TopLeft,
            position: trigger_bounds.origin,
            offset: point(px(0.0), px(0.0)),
            max_height,
            scrolling,
        },
    }
}

fn estimated_menu_size(look: &SelectorLook, item_count: usize, trigger_width: Pixels) -> Size<Pixels> {
    let menu_min_width = px(look.items_panel.min_width);
    Size {
        width: if trigger_width > menu_min_width {
            trigger_width
        } else {
            menu_min_width
        },
        height: px(look.items_panel.padding * 2.0) + px(look.items_panel.item_height) * item_count,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::selector::SelectorTriggerStyle;
    use crate::theme::{ControlSize, InteractionState, StandardBoxScale};

    fn look() -> SelectorLook {
        let theme = default_selector_theme();
        theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Md,
            &StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0),
            false,
        )
    }

    #[test]
    fn smart_placement_uses_below_when_it_fits() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(80.0)), size(px(160.0), px(32.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::Smart,
            &look,
            3,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(popup_metrics.anchor, Corner::TopLeft);
        assert_eq!(popup_metrics.position, point(px(12.0), px(112.0)));
        assert!(!popup_metrics.scrolling);
    }

    #[test]
    fn overlay_placement_anchors_to_trigger_origin() {
        let look = look();
        let trigger = Bounds::new(point(px(30.0), px(70.0)), size(px(150.0), px(30.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::OverlayOnTrigger,
            &look,
            6,
            size(px(320.0), px(360.0)),
        );

        assert_eq!(popup_metrics.anchor, Corner::TopLeft);
        assert_eq!(popup_metrics.position, point(px(30.0), px(70.0)));
        assert_eq!(popup_metrics.offset, point(px(0.0), px(0.0)));
    }

    #[test]
    fn smart_placement_prefers_larger_visible_side_and_enables_scroll() {
        let look = look();
        let trigger = Bounds::new(point(px(12.0), px(210.0)), size(px(160.0), px(32.0)));
        let popup_metrics = resolve_selector_popup_metrics(
            Some(trigger),
            SelectorPlacement::Smart,
            &look,
            20,
            size(px(320.0), px(280.0)),
        );

        assert_eq!(popup_metrics.anchor, Corner::BottomLeft);
        assert!(popup_metrics.scrolling);
        assert!(popup_metrics.max_height < estimated_menu_size(&look, 20, trigger.size.width).height);
    }
}
