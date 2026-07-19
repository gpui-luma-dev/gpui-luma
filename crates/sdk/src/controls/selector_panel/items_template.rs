use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, MouseButton, MouseDownEvent, Pixels, SharedString, Stateful, Window, div,
    prelude::*, px,
};
use lucide_icons::Icon as LucideIcon;

use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::{SelectorItemLike, SelectorItemRenderModel, SelectorItemTemplate, SelectorPath};
use crate::controls::state::ControlFocusState;
use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectorItemsPanelLook {
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: gpui::Hsla,
    pub item_hover_background: gpui::Hsla,
    pub item_hover_foreground: gpui::Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
}

pub fn default_selector_items_panel_look(tokens: &ThemeTokens, size: ControlSize) -> SelectorItemsPanelLook {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;
    let item_typography = match size {
        ControlSize::Sm => typography.text.scale.sm,
        ControlSize::Md => typography.text.body,
        ControlSize::Lg => typography.text.scale.lg,
    };

    SelectorItemsPanelLook {
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_hover_foreground: palette.state.hover.foreground,
        item_typography,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
    }
}

pub type SelectorPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SelectorPanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Default)]
pub struct SelectorItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_mouse_downs: Vec<SelectorPanelMouseDownHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub struct SelectorItemsRenderModel<'a, T>
where
    T: SelectorItemLike + 'static,
{
    pub menu_id: &'a SharedString,
    pub selector_id: &'a SharedString,
    pub items: &'a [T],
    pub selected_index: Option<usize>,
    pub active_path: Option<SelectorPath>,
    pub open: bool,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: Option<&'a SelectorItemTemplate<T>>,
    pub look: SelectorItemsPanelLook,
    pub max_height: Pixels,
    pub scrolling: bool,
}

pub trait SelectorItemsTemplate<T>: Send + Sync
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub type SelectorItemsTemplateModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    modifiers: Vec<SelectorItemsTemplateModifier<T>>,
}

struct ModifiedSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    base: Arc<dyn SelectorItemsTemplate<T>>,
    modifiers: Vec<SelectorItemsTemplateModifier<T>>,
}

impl<T> DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorItemsRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl<T> Default for DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ModifiedSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn new(base: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectorItemsRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl<T> SelectorItemsTemplate<T> for DefaultSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectorItemsTemplateHandlers { item_hovers, item_mouse_downs, item_clicks } = handlers;
        let look = model.look.clone();
        let content_max_height = (model.max_height - px(look.padding * 2.0)).max(px(look.item_height));
        let mut rows = div().id(format!("{}-rows", model.menu_id)).relative().flex().flex_col().w_full();

        if model.scrolling {
            rows = rows.max_h(content_max_height).overflow_y_scroll();
        }

        let mut mouse_downs = item_mouse_downs.into_iter();
        let mut clicks = item_clicks.into_iter();

        for ((index, item), hover) in model.items.iter().enumerate().zip(item_hovers) {
            let enabled_item = item.is_enabled();
            let color = if enabled_item {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let selected = model.selected_index == Some(index);
            let active = model.active_path.is_some_and(|path| path.is_item(index));
            let content = if let Some(item_template) = model.item_template {
                let item_model = SelectorItemRenderModel {
                    selector_id: model.selector_id,
                    item,
                    index,
                    selected,
                    active,
                    open: model.open,
                    enabled: model.enabled && enabled_item,
                };
                item_template(&item_model, cx)
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap(px(look.item_gap))
                    .child(div().flex_1().child(item.label().clone()))
                    .into_any_element()
            };

            let mut row = div()
                .id(format!("{}-item-{}", model.menu_id, item.id()))
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
                .child(div().flex_1().child(content))
                .child(render_selection_checkmark(selected, color, look.item_icon_size));

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if let Some(mouse_down) = mouse_downs.next() {
                    row = row.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.stop_propagation();
                        mouse_down(event, window, cx);
                    });
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = row.opacity(0.56);
            }

            rows = rows.child(row);
        }

        let root = div()
            .id(format!("{}-menu", model.menu_id))
            .relative()
            .min_w(px(look.min_width))
            .max_h(model.max_height)
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .overflow_hidden()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .child(rows);

        self.apply_modifiers(root, model)
    }
}

impl<T> SelectorItemsTemplate<T> for ModifiedSelectorItemsTemplate<T>
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_selector_items_template<T>() -> Arc<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(DefaultSelectorItemsTemplate::<T>::new())
}

pub(crate) fn items_template_with_modifier<T, F>(
    template: Arc<dyn SelectorItemsTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectorItemsRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedSelectorItemsTemplate::new(template).with_modifier(modifier))
}

fn render_selection_checkmark(selected: bool, color: gpui::Hsla, size: f32) -> AnyElement {
    if selected {
        render_lucide_icon(LucideIcon::Check, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    lucide_icon(icon, color, size)
}
