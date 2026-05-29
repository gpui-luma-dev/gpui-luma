//! List view token mapping for virtualized row lists.

use gpui::hsla;

use crate::controls::list_view::{ListViewAppearance, ListViewRowAppearance};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle};

use super::mode::RadixModeTokens;
use super::resolve::{resolve_color, resolve_color_layer};

pub(crate) fn list_view_appearance(
    mode: &RadixModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> ListViewAppearance {
    if mode.catalog.tokens.is_empty() {
        list_view_appearance_from_palette(mode, enabled, focused, size)
    } else {
        list_view_appearance_from_catalog(&mode.catalog, mode, enabled, focused, size)
            .unwrap_or_else(|err| panic!("list view appearance properties: {err}"))
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

fn list_view_appearance_from_palette(
    mode: &RadixModeTokens,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> ListViewAppearance {
    let palette = &mode.palette;
    let metrics = &mode.metrics;

    ListViewAppearance {
        background: if enabled {
            palette.app_background
        } else {
            palette.disabled_background
        },
        border: palette.input_background,
        header_background: if enabled {
            palette.muted_background
        } else {
            palette.disabled_background
        },
        header_label_color: if enabled {
            palette.app_muted_foreground
        } else {
            palette.disabled_foreground
        },
        header_typography: mode.typography.text.caption,
        radius: metrics.radius(size),
        padding_x: 0.0,
        padding_y: metrics.padding_y(size) * 0.5,
    }
}

fn list_view_appearance_from_catalog(
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> anyhow::Result<ListViewAppearance> {
    let metrics = &mode.metrics;

    Ok(ListViewAppearance {
        background: if enabled {
            resolve_color(catalog, "background")?
        } else {
            resolve_color(catalog, "muted")?
        },
        border: resolve_color(catalog, "input")?,
        header_background: resolve_color(catalog, "muted")?,
        header_label_color: resolve_color(catalog, "muted-foreground")?,
        header_typography: mode.typography.text.caption,
        radius: metrics.radius(size),
        padding_x: 0.0,
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

    let row_highlight = palette.muted_background;

    let background = if state.disabled {
        transparent
    } else if selected {
        row_highlight
    } else {
        match state.layer() {
            InteractionLayer::Disabled => transparent,
            InteractionLayer::Pressed => palette.secondary.pressed_background,
            InteractionLayer::Hovered => row_highlight,
            InteractionLayer::Default if state.focused => row_highlight,
            InteractionLayer::Default => transparent,
        }
    };

    let label_color = if state.disabled {
        palette.disabled_foreground
    } else {
        palette.app_foreground
    };

    ListViewRowAppearance {
        background,
        label_color,
        divider: palette.input_background,
        adorner: None,
        label_typography: typography.text.label,
        radius: 0.0,
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

    let row_highlight = resolve_color(catalog, "muted")?;

    let background = if state.disabled {
        transparent
    } else if selected {
        row_highlight
    } else {
        match layer {
            InteractionLayer::Pressed | InteractionLayer::Hovered => {
                resolve_color_layer(catalog, "muted", layer, true)?
            }
            InteractionLayer::Default if state.focused => row_highlight,
            InteractionLayer::Default => transparent,
            InteractionLayer::Disabled => transparent,
        }
    };

    let label_color = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else {
        resolve_color(catalog, "foreground")?
    };

    Ok(ListViewRowAppearance {
        background,
        label_color,
        divider: resolve_color(catalog, "border")?,
        adorner: None,
        label_typography: LumaTextStyle {
            size: typography.text.label.size,
            line_height: typography.text.label.line_height,
            weight: typography.text.label.weight,
        },
        radius: 0.0,
        padding_x: metrics.padding_x(size),
        padding_y: metrics.padding_y(size),
        min_height: metrics.control_height(size),
    })
}
