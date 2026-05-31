//! List box — same input surface tokens as text field; accent row hover.

use gpui::hsla;

use crate::controls::listbox::{ListBoxListAppearance, ListBoxRowPalette};
use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState};

use super::focus::focus_adorner;
use super::resolve::{resolve_color, resolve_color_layer};
use super::mode::RadixModeTokens;

pub(crate) fn listbox_list_appearance(
    mode: &RadixModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> ListBoxListAppearance {
    if mode.catalog.tokens.is_empty() {
        listbox_list_from_palette(mode, enabled, focused, size)
    } else {
        listbox_list_from_catalog(&mode.catalog, mode, enabled, focused, size)
            .unwrap_or_else(|err| panic!("listbox list properties: {err}"))
    }
}

pub(crate) fn listbox_row_palette(
    mode: &RadixModeTokens,
    _selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> ListBoxRowPalette {
    if mode.catalog.tokens.is_empty() {
        listbox_row_from_palette(mode, state, size)
    } else {
        listbox_row_from_catalog(&mode.catalog, mode, state)
            .unwrap_or_else(|err| panic!("listbox row properties: {err}"))
    }
}

fn listbox_list_from_palette(
    mode: &RadixModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> ListBoxListAppearance {
    let palette = &mode.palette;
    let metrics = &mode.metrics;

    let adorner = if focused {
        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: palette.focus_ring,
            placement: AdornerPlacement::Inset,
            distance: metrics.border_width.default,
            width: metrics.focus.width,
        }))
    } else {
        None
    };

    ListBoxListAppearance {
        background: if enabled {
            palette.app_background
        } else {
            palette.disabled_background
        },
        border: palette.input_background,
        adorner,
        divider: palette.border_default,
        radius: metrics.radius(size),
        padding_x: 6.0,
        padding_y: metrics.padding_y(size) * 0.5,
        row_gap: metrics.padding_y(size) * 0.25,
    }
}

fn listbox_list_from_catalog(
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> anyhow::Result<ListBoxListAppearance> {
    let metrics = &mode.metrics;

    Ok(ListBoxListAppearance {
        background: if enabled {
            resolve_color(catalog, "background")?
        } else {
            resolve_color(catalog, "muted")?
        },
        border: resolve_color(catalog, "input")?,
        adorner: focus_adorner(catalog, metrics, focused)?,
        divider: resolve_color(catalog, "border")?,
        radius: metrics.radius(size),
        padding_x: 6.0,
        padding_y: metrics.padding_y(size) * 0.5,
        row_gap: metrics.padding_y(size) * 0.25,
    })
}

fn listbox_row_from_palette(mode: &RadixModeTokens, state: InteractionState, _size: ControlSize) -> ListBoxRowPalette {
    let palette = &mode.palette;
    let typography = &mode.typography;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let background = match state.layer() {
        InteractionLayer::Disabled => transparent,
        InteractionLayer::Pressed => palette.secondary.pressed_background,
        InteractionLayer::Hovered => palette.secondary.background,
        InteractionLayer::Default if state.focused => palette.secondary.background,
        InteractionLayer::Default => transparent,
    };

    let label_color = if state.disabled {
        palette.disabled_foreground
    } else {
        palette.app_foreground
    };

    ListBoxRowPalette { background, label_color, adorner: None, label_typography: typography.text.label }
}

fn listbox_row_from_catalog(
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    state: InteractionState,
) -> anyhow::Result<ListBoxRowPalette> {
    let typography = &mode.typography;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let layer = state.layer();

    let background = if state.disabled {
        transparent
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
    } else {
        resolve_color(catalog, "foreground")?
    };

    Ok(ListBoxRowPalette { background, label_color, adorner: None, label_typography: typography.text.label })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::theme::{ControlSize, InteractionLayer};

    use super::super::resolve::resolve_color_layer;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::{listbox_list_appearance, listbox_row_palette};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn listbox_uses_input_border_and_accent_hover() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let list = listbox_list_appearance(&mode, true, false, ControlSize::Md);
        let row = listbox_row_palette(
            &mode,
            false,
            crate::theme::InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );

        assert_eq!(list.border, catalog.color("input").expect("input"));
        let expected_hover =
            resolve_color_layer(&catalog, "accent", InteractionLayer::Hovered, true).expect("accent hover");
        assert_eq!(row.background, expected_hover);
    }
}
