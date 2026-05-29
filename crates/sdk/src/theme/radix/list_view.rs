//! List view token mapping for virtualized row lists.

use gpui::hsla;

use crate::controls::list_view::{ListViewListAppearance, ListViewRowAppearance};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle};

use super::focus::focus_adorner;
use super::mode::RadixModeTokens;
use super::resolve::{resolve_color, resolve_color_layer};

pub(crate) fn list_view_list_appearance(
    mode: &RadixModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> ListViewListAppearance {
    if mode.catalog.tokens.is_empty() {
        list_view_list_from_palette(mode, enabled, focused, size)
    } else {
        list_view_list_from_catalog(&mode.catalog, mode, enabled, focused, size)
            .unwrap_or_else(|err| panic!("list view list properties: {err}"))
    }
}

pub(crate) fn list_view_row_appearance(
    mode: &RadixModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> ListViewRowAppearance {
    if mode.catalog.tokens.is_empty() {
        list_view_row_from_palette(mode, selected, state, size)
    } else {
        list_view_row_from_catalog(&mode.catalog, mode, selected, state, size)
            .unwrap_or_else(|err| panic!("list view row properties: {err}"))
    }
}

fn list_view_list_from_palette(
    mode: &RadixModeTokens,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> ListViewListAppearance {
    let palette = &mode.palette;
    let metrics = &mode.metrics;

    ListViewListAppearance {
        background: if enabled {
            palette.app_background
        } else {
            palette.disabled_background
        },
        border: palette.input_background,
        header_label_color: if enabled {
            palette.selected_foreground
        } else {
            palette.disabled_foreground
        },
        header_typography: mode.typography.text.label,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size) * 0.5,
        padding_y: metrics.padding_y(size) * 0.5,
    }
}

fn list_view_list_from_catalog(
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> anyhow::Result<ListViewListAppearance> {
    let metrics = &mode.metrics;

    Ok(ListViewListAppearance {
        background: if enabled {
            resolve_color(catalog, "background")?
        } else {
            resolve_color(catalog, "muted")?
        },
        border: resolve_color(catalog, "input")?,
        header_label_color: resolve_color(catalog, "primary-foreground")?,
        header_typography: mode.typography.text.label,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size) * 0.5,
        padding_y: metrics.padding_y(size) * 0.5,
    })
}

fn list_view_row_from_palette(
    mode: &RadixModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> ListViewRowAppearance {
    let palette = &mode.palette;
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let background = if state.disabled {
        transparent
    } else if selected {
        palette.selected_background
    } else {
        match state.layer() {
            InteractionLayer::Disabled => transparent,
            InteractionLayer::Pressed => palette.secondary.pressed_background,
            InteractionLayer::Hovered => palette.secondary.background,
            InteractionLayer::Default if state.focused => palette.secondary.background,
            InteractionLayer::Default => transparent,
        }
    };

    let label_color = if state.disabled {
        palette.disabled_foreground
    } else if selected {
        palette.selected_foreground
    } else {
        palette.app_foreground
    };

    let adorner = if state.focused {
        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: palette.focus_ring,
            placement: AdornerPlacement::Inset,
            distance: metrics.border_width.default,
            width: metrics.focus.width,
        }))
    } else {
        None
    };

    ListViewRowAppearance {
        background,
        label_color,
        adorner,
        label_typography: typography.text.label,
        radius: metrics.radius(size) * 0.8,
        padding_x: metrics.padding_x(size),
        padding_y: metrics.padding_y(size),
        min_height: metrics.control_height(size),
    }
}

fn list_view_row_from_catalog(
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> anyhow::Result<ListViewRowAppearance> {
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let layer = state.layer();

    let background = if state.disabled {
        transparent
    } else if selected {
        resolve_color(catalog, "primary")?
    } else {
        match layer {
            InteractionLayer::Pressed | InteractionLayer::Hovered => {
                resolve_color_layer(catalog, "accent", layer, true)?
            }
            InteractionLayer::Default if state.focused => resolve_color(catalog, "accent")?,
            InteractionLayer::Default => transparent,
            InteractionLayer::Disabled => transparent,
        }
    };

    let label_color = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else if selected {
        resolve_color(catalog, "primary-foreground")?
    } else {
        resolve_color(catalog, "foreground")?
    };

    Ok(ListViewRowAppearance {
        background,
        label_color,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: LumaTextStyle {
            size: typography.text.label.size,
            line_height: typography.text.label.line_height,
            weight: typography.text.label.weight,
        },
        radius: metrics.radius(size) * 0.8,
        padding_x: metrics.padding_x(size),
        padding_y: metrics.padding_y(size),
        min_height: metrics.control_height(size),
    })
}
