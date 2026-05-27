//! Text field property mappings (shadcn / tweakcn):
//!
//! | Part        | Token              |
//! |-------------|--------------------|
//! | Background  | `background`       |
//! | Border      | `input`            |
//! | Foreground  | `foreground`       |
//! | Placeholder | `muted-foreground` |
//! | Selection   | `primary`          |
//! | Focus ring  | `ring`             |
//! | Ghost hover | `accent`           |

use gpui::hsla;

use crate::controls::textfield::{TextFieldAppearance, TextFieldState, TextFieldVariant};
use crate::theme::ControlSize;

use super::focus::focus_ring_color;
use super::resolve::{resolve_color, resolve_ghost_background, resolve_label_color};
use super::super::catalog::CssTokenMap;
use super::super::mode::RadixModeTokens;
use super::super::palette::RadixPalette;

pub(crate) fn textfield_appearance(
    mode: &RadixModeTokens,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldAppearance {
    if mode.catalog.tokens.is_empty() {
        textfield_appearance_from_palette(&mode.palette, mode, variant, state, enabled)
    } else {
        textfield_appearance_from_catalog(&mode.catalog, mode, variant, state, enabled)
            .unwrap_or_else(|err| panic!("textfield properties: {err}"))
    }
}

fn textfield_appearance_from_palette(
    palette: &RadixPalette,
    mode: &RadixModeTokens,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldAppearance {
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let size = ControlSize::Md;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let (background, foreground, border, placeholder, icon, selection_background, caret) = match (variant, enabled) {
        (TextFieldVariant::Standard, true) => {
            let background = if state.focused {
                palette.app_background
            } else if state.hovered {
                palette.muted_background
            } else {
                palette.app_background
            };
            let border = if state.invalid {
                palette.focus_ring
            } else {
                palette.input_background
            };

            (
                background,
                palette.app_foreground,
                border,
                palette.app_muted_foreground,
                palette.app_muted_foreground,
                palette.selected_background,
                palette.app_foreground,
            )
        }
        (TextFieldVariant::Ghost, true) => {
            let ghost = palette.ghost;
            let background = if state.focused {
                ghost.background
            } else if state.hovered {
                ghost.hover_background
            } else {
                transparent
            };
            let placeholder_color = ghost.foreground.opacity(0.65);

            (
                background,
                ghost.foreground,
                ghost.border,
                placeholder_color,
                placeholder_color,
                palette.selected_background,
                ghost.foreground,
            )
        }
        (TextFieldVariant::Standard, false) => (
            palette.disabled_background,
            palette.disabled_foreground,
            palette.input_background,
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.selected_background,
            palette.disabled_foreground,
        ),
        (TextFieldVariant::Ghost, false) => (
            transparent,
            palette.disabled_foreground,
            palette.ghost.border,
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.selected_background,
            palette.disabled_foreground,
        ),
    };

    TextFieldAppearance {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        caret,
        focus_ring: (enabled && state.focus_visible).then_some(palette.focus_ring),
        typography: typography.text.body,
        font_family: typography.font.sans.family.clone().into(),
        min_height: metrics.control_height(size),
        padding_x: metrics.padding_x(size),
        padding_y: metrics.padding_y(size),
        gap: metrics.gap(size),
        radius: metrics.radius(size),
        border_width: metrics.border_width.default,
        icon_size: typography.text.body.size + 2.0,
    }
}

pub(crate) fn textfield_appearance_from_catalog(
    catalog: &CssTokenMap,
    mode: &RadixModeTokens,
    variant: TextFieldVariant,
    state: TextFieldState,
    enabled: bool,
) -> anyhow::Result<TextFieldAppearance> {
    let metrics = &mode.metrics;
    let typography = &mode.typography;
    let size = ControlSize::Md;
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let (background, foreground, border, placeholder, icon, selection_background, caret) = match (variant, enabled) {
        (TextFieldVariant::Standard, true) => {
            let background = if state.focused {
                resolve_color(catalog, "background")?
            } else if state.hovered {
                resolve_color(catalog, "accent")?
            } else {
                resolve_color(catalog, "background")?
            };
            let border = if state.invalid {
                focus_ring_color(catalog)?
            } else {
                resolve_color(catalog, "input")?
            };

            (
                background,
                resolve_color(catalog, "foreground")?,
                border,
                resolve_color(catalog, "muted-foreground")?,
                resolve_color(catalog, "muted-foreground")?,
                resolve_color(catalog, "primary")?,
                resolve_color(catalog, "foreground")?,
            )
        }
        (TextFieldVariant::Ghost, true) => {
            let layer = if state.focused {
                crate::theme::InteractionLayer::Default
            } else if state.hovered {
                crate::theme::InteractionLayer::Hovered
            } else {
                crate::theme::InteractionLayer::Default
            };
            let background = resolve_ghost_background(catalog, layer)?;
            let foreground = resolve_label_color(catalog, false)?;
            let placeholder_color = foreground.opacity(0.65);

            (
                background,
                foreground,
                resolve_color(catalog, "border")?,
                placeholder_color,
                placeholder_color,
                resolve_color(catalog, "primary")?,
                foreground,
            )
        }
        (TextFieldVariant::Standard, false) => (
            resolve_color(catalog, "muted")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "input")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "primary")?,
            resolve_color(catalog, "muted-foreground")?,
        ),
        (TextFieldVariant::Ghost, false) => (
            transparent,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "border")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "primary")?,
            resolve_color(catalog, "muted-foreground")?,
        ),
    };

    Ok(TextFieldAppearance {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        caret,
        focus_ring: (enabled && state.focus_visible).then(|| focus_ring_color(catalog)).transpose()?,
        typography: typography.text.body,
        font_family: typography.font.sans.family.clone().into(),
        min_height: metrics.control_height(size),
        padding_x: metrics.padding_x(size),
        padding_y: metrics.padding_y(size),
        gap: metrics.gap(size),
        radius: metrics.radius(size),
        border_width: metrics.border_width.default,
        icon_size: typography.text.body.size + 2.0,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::controls::textfield::{TextFieldState, TextFieldVariant};

    use super::super::super::catalog::CssTokenMap;
    use super::super::super::mode::RadixModeTokens;
    use super::textfield_appearance_from_catalog;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn standard_textfield_uses_input_border_and_background() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = textfield_appearance_from_catalog(
            &catalog,
            &mode,
            TextFieldVariant::Standard,
            TextFieldState::default(),
            true,
        )
        .expect("textfield");

        assert_eq!(appearance.background, catalog.color("background").expect("background"));
        assert_eq!(appearance.border, catalog.color("input").expect("input"));
        assert_eq!(appearance.foreground, catalog.color("foreground").expect("foreground"));
    }
}
