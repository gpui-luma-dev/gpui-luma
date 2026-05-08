use std::sync::Arc;

use gpui::{AnyElement, App, ClickEvent, Div, SharedString, Stateful, Window, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

use crate::controls::icon::lucide_icon;
use crate::controls::selector_panel::{SelectorItemLike, SelectorItemRenderModel, SelectorItemTemplate, SelectorPath};
use crate::controls::state::ControlFocusState;
use crate::theme::{ControlSize, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SelectorItemsPanelAppearance {
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: gpui::Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_disabled_foreground: gpui::Hsla,
    pub item_hover_background: gpui::Hsla,
    pub item_typography: LumaTextStyle,
    pub item_height: f32,
    pub item_padding_x: f32,
    pub item_gap: f32,
    pub item_icon_size: f32,
    pub item_radius: f32,
}

pub fn default_selector_items_panel_appearance(
    tokens: &ThemeTokens,
    size: ControlSize,
) -> SelectorItemsPanelAppearance {
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let elevation = &tokens.elevation;

    SelectorItemsPanelAppearance {
        background: palette.surface.floating.background,
        foreground: palette.surface.floating.foreground,
        border: palette.surface.floating.border,
        shadow: elevation.menu.to_box_shadows(),
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.state.disabled.foreground,
        item_hover_background: palette.state.hover.background,
        item_typography: typography.text.label,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
    }
}

pub type SelectorPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectorPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub struct SelectorItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
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
    pub appearance: SelectorItemsPanelAppearance,
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

pub struct DefaultSelectorItemsTemplate;

impl<T> SelectorItemsTemplate<T> for DefaultSelectorItemsTemplate
where
    T: SelectorItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectorItemsRenderModel<'_, T>,
        handlers: SelectorItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectorItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let appearance = model.appearance.clone();
        let mut menu = div()
            .id(format!("{}-menu", model.menu_id))
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
            let enabled_item = item.is_enabled();
            let color = if enabled_item {
                appearance.foreground
            } else {
                appearance.item_disabled_foreground
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
                    enabled: model.enabled,
                };
                item_template(&item_model, cx)
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap(px(appearance.item_gap))
                    .child(div().flex_1().child(item.label().clone()))
                    .into_any_element()
            };

            let mut row = div()
                .id(format!("{}-item-{}", model.menu_id, item.id()))
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

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = appearance.item_hover_background;
                    move |style| style.bg(hover_background)
                });

                if active {
                    row = row.bg(appearance.item_hover_background);
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = row.opacity(0.56);
            }

            menu = menu.child(row);
        }

        menu
    }
}

pub fn default_selector_items_template<T>() -> Arc<dyn SelectorItemsTemplate<T>>
where
    T: SelectorItemLike + 'static,
{
    Arc::new(DefaultSelectorItemsTemplate)
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
