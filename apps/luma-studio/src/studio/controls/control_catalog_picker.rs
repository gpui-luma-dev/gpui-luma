use std::sync::Arc;

use gpui::{
    AnyElement, Context, FontFeatures, FontWeight, MouseButton, Pixels, SharedString, Size, TextRun, Window, div, font,
    prelude::*, px, size,
};
use gpui_luma::controls::choice_indicator_layout::shadow_extent_from_slice;
use gpui_luma::controls::floating_menu::FloatingMenuLook;
use gpui_luma::theme::LumaTextStyle;

use crate::studio::components::catalog::{
    ComponentCatalogEntry, ComponentCatalogGroup, controls_exposition_id, groups_for_column,
};

const COLUMN_COUNT: usize = 4;
const CATEGORY_ICON_SIZE: f32 = 14.0;

pub(crate) fn render_control_catalog_picker<M: 'static>(
    menu_look: &FloatingMenuLook,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    let category_style = menu_look.item_typography;
    let item_style = menu_look.item_typography;

    with_menu_elevation(
        "controls-catalog-picker",
        div()
            .id("controls-catalog-picker")
            .flex_shrink_0()
            .occlude()
            .p(px(menu_look.padding))
            .bg(menu_look.background)
            .border_1()
            .border_color(menu_look.border)
            .rounded(px(menu_look.radius))
            .shadow(menu_look.shadow.clone())
            .child(div().flex_shrink_0().flex().items_start().gap(px(menu_look.item_gap * 3.5)).children(
                (0..COLUMN_COUNT).map(|column| {
                    render_column(
                        menu_look,
                        column,
                        selected_exposition_id,
                        interactive,
                        category_style,
                        item_style,
                        cx,
                        on_select.clone(),
                    )
                }),
            )),
        &menu_look.shadow,
    )
    .into_any_element()
}

pub(crate) fn estimate_control_catalog_picker_size(
    menu_look: &FloatingMenuLook,
    font_family: SharedString,
    window: &mut Window,
) -> Size<Pixels> {
    let elevation_extent = shadow_extent_from_slice(&menu_look.shadow, 1.0, true);
    let column_gap = menu_look.item_gap * 3.5;
    let column_widths = (0..COLUMN_COUNT)
        .map(|column| estimate_column_width(menu_look, column, font_family.clone(), window))
        .collect::<Vec<_>>();
    let width = elevation_extent * 2.0
        + menu_look.padding * 2.0
        + column_widths.iter().sum::<f32>()
        + column_gap * (COLUMN_COUNT.saturating_sub(1) as f32);
    let height = elevation_extent * 2.0
        + menu_look.padding * 2.0
        + (0..COLUMN_COUNT).map(|column| estimate_column_height(menu_look, column)).fold(0.0_f32, f32::max);

    size(px(width), px(height))
}

fn with_menu_elevation(
    id: impl Into<gpui::SharedString>,
    surface: gpui::Stateful<gpui::Div>,
    shadows: &[gpui::BoxShadow],
) -> gpui::Stateful<gpui::Div> {
    let extent = shadow_extent_from_slice(shadows, 1.0, true);
    if extent <= 0.0 {
        return surface;
    }
    div().id(format!("{}-elevation", id.into())).relative().p(px(extent)).child(surface)
}

fn render_column<M: 'static>(
    menu_look: &FloatingMenuLook,
    column: usize,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    category_style: LumaTextStyle,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(menu_look.item_gap * 1.5))
        .children(groups_for_column(column).map(|group| {
            render_category_section(
                menu_look,
                group,
                selected_exposition_id,
                interactive,
                category_style,
                item_style,
                cx,
                on_select.clone(),
            )
        }))
        .into_any_element()
}

fn estimate_column_width(
    menu_look: &FloatingMenuLook,
    column: usize,
    font_family: SharedString,
    window: &mut Window,
) -> f32 {
    groups_for_column(column)
        .map(|group| estimate_category_width(menu_look, group, font_family.clone(), window))
        .fold(0.0_f32, f32::max)
}

fn estimate_column_height(menu_look: &FloatingMenuLook, column: usize) -> f32 {
    let group_count = groups_for_column(column).count();
    let group_gaps = group_count.saturating_sub(1) as f32 * menu_look.item_gap * 1.5;
    groups_for_column(column)
        .map(|group| {
            let row_count = 1 + group.entries.len();
            let row_gaps = row_count.saturating_sub(1) as f32 * menu_look.item_gap * 0.25;
            row_count as f32 * menu_look.item_height.max(menu_look.item_typography.line_height) + row_gaps
        })
        .sum::<f32>()
        + group_gaps
}

fn estimate_category_width(
    menu_look: &FloatingMenuLook,
    group: &ComponentCatalogGroup,
    font_family: SharedString,
    window: &mut Window,
) -> f32 {
    let category_width = CATEGORY_ICON_SIZE
        + menu_look.item_gap
        + shaped_text_width(group.label, menu_look.item_typography, font_family.clone(), window);
    let item_width = group
        .entries
        .iter()
        .map(|entry| {
            shaped_text_width(entry.label, menu_look.item_typography, font_family.clone(), window)
                + menu_look.item_padding_x * 2.0
        })
        .fold(0.0_f32, f32::max);

    category_width.max(item_width)
}

fn shaped_text_width(text: &'static str, style: LumaTextStyle, font_family: SharedString, window: &mut Window) -> f32 {
    let text = SharedString::from(text);
    let run = TextRun {
        len: text.len(),
        font: {
            let mut font = font(font_family);
            font.weight = style.weight;
            font
        },
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(text.clone(), px(style.size), &[run], None);
    line.x_for_index(text.len()).as_f32()
}

fn render_category_section<M: 'static>(
    menu_look: &FloatingMenuLook,
    group: &ComponentCatalogGroup,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    category_style: LumaTextStyle,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    div()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(menu_look.item_gap * 0.25))
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(menu_look.item_gap))
                .whitespace_nowrap()
                .text_color(menu_look.foreground)
                .child(render_category_icon(group.icon, menu_look.foreground))
                .child(
                    div()
                        .text_size(px(category_style.size))
                        .line_height(px(category_style.line_height))
                        .font_weight(FontWeight::SEMIBOLD)
                        .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1), ("c2sc".into(), 1)])))
                        .child(group.label),
                ),
        )
        .children(group.entries.iter().map(|entry| {
            render_catalog_item(
                menu_look,
                entry,
                selected_exposition_id,
                interactive,
                item_style,
                cx,
                on_select.clone(),
            )
        }))
        .into_any_element()
}

fn render_catalog_item<M: 'static>(
    menu_look: &FloatingMenuLook,
    entry: &ComponentCatalogEntry,
    selected_exposition_id: Option<&'static str>,
    interactive: bool,
    item_style: LumaTextStyle,
    cx: &mut Context<M>,
    on_select: impl Fn(&mut M, &'static str, &mut Window, &mut Context<M>) + Clone + 'static,
) -> AnyElement {
    let gallery_id = entry.id;
    let exposition_id = controls_exposition_id(gallery_id);
    let selectable = interactive && exposition_id.is_some();
    let active = exposition_id.is_some_and(|id| selected_exposition_id == Some(id));

    div()
        .id(format!("controls-catalog-item-{gallery_id}"))
        .flex_shrink_0()
        .whitespace_nowrap()
        .px(px(menu_look.item_padding_x))
        .min_h(px(menu_look.item_height * 0.85))
        .flex()
        .items_center()
        .rounded(px(menu_look.item_radius))
        .text_size(px(item_style.size))
        .line_height(px(item_style.line_height))
        .text_color(if active {
            menu_look.item_hover_foreground
        } else {
            menu_look.foreground
        })
        .font_weight(if active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .when(active, |item| item.bg(menu_look.item_hover_background))
        .when(selectable, |item| item.cursor_pointer())
        .when(selectable, |item| {
            item.hover(move |style| {
                style.bg(menu_look.item_hover_background).text_color(menu_look.item_hover_foreground)
            })
        })
        .when(selectable, |item| {
            let exposition_id = exposition_id.expect("selectable catalog item must map to exposition");
            item.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    on_select(this, exposition_id, window, cx);
                }),
            )
        })
        .child(entry.label)
        .into_any_element()
}

fn render_category_icon(icon: lucide_icons::Icon, color: gpui::Hsla) -> AnyElement {
    div()
        .flex_shrink_0()
        .font_family("lucide")
        .text_size(px(CATEGORY_ICON_SIZE))
        .line_height(px(CATEGORY_ICON_SIZE))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
