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

use gpui::hsla;

use gpui_luma::controls::textfield::{TextFieldPalette, TextFieldState};
use gpui_luma::theme::{InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::focus::focus_ring_color;
use crate::resolve::{resolve_color, resolve_label_color};
use crate::mode::ShadcnModeTokens;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Standard,
    Ghost,
}

pub(crate) fn textfield_palette(
    mode: &ShadcnModeTokens,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        textfield_palette_from_palette(&ctx, style, state, enabled)
    } else {
        textfield_palette_from_catalog(&ctx, style, state, enabled)
            .unwrap_or_else(|err| panic!("textfield properties: {err}"))
    }
}

fn textfield_palette_from_palette(
    ctx: &AppearanceContext,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    let palette = ctx.palette();
    let typography = ctx.typography();
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let (background, foreground, border, placeholder, icon, selection_background, caret) = match (style, enabled) {
        (ShadcnTextFieldStyle::Standard, true) => {
            let background = palette.app_background;
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
        (ShadcnTextFieldStyle::Ghost, true) => {
            let ghost = palette.ghost;
            let background = if state.focused { ghost.background } else { transparent };
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
        (ShadcnTextFieldStyle::Standard, false) => (
            palette.disabled_background,
            palette.disabled_foreground,
            palette.input_background,
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.selected_background,
            palette.disabled_foreground,
        ),
        (ShadcnTextFieldStyle::Ghost, false) => (
            transparent,
            palette.disabled_foreground,
            palette.ghost.border,
            palette.disabled_foreground,
            palette.disabled_foreground,
            palette.selected_background,
            palette.disabled_foreground,
        ),
    };

    TextFieldPalette {
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
    }
}

pub(crate) fn textfield_palette_from_catalog(
    ctx: &AppearanceContext,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> anyhow::Result<TextFieldPalette> {
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let (background, foreground, border, placeholder, icon, selection_background, caret) = match (style, enabled) {
        (ShadcnTextFieldStyle::Standard, true) => {
            let background = resolve_color(catalog, "background")?;
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
        (ShadcnTextFieldStyle::Ghost, true) => {
            let background = transparent;
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
        (ShadcnTextFieldStyle::Standard, false) => (
            resolve_color(catalog, "muted")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "input")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "primary")?,
            resolve_color(catalog, "muted-foreground")?,
        ),
        (ShadcnTextFieldStyle::Ghost, false) => (
            transparent,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "border")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "muted-foreground")?,
            resolve_color(catalog, "primary")?,
            resolve_color(catalog, "muted-foreground")?,
        ),
    };

    Ok(TextFieldPalette {
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
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::controls::textfield::TextFieldState;

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::{ShadcnTextFieldStyle, textfield_palette_from_catalog};

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
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, Default::default());
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Standard, TextFieldState::default(), true)
                .expect("textfield");

        assert_eq!(appearance.background, catalog.color("background").expect("background"));
        assert_eq!(appearance.border, catalog.color("input").expect("input"));
        assert_eq!(appearance.foreground, catalog.color("foreground").expect("foreground"));
    }

    #[test]
    fn standard_textfield_hover_does_not_change_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, Default::default());
        let default =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Standard, TextFieldState::default(), true)
                .expect("textfield");
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let appearance =
            textfield_palette_from_catalog(&ctx, ShadcnTextFieldStyle::Standard, hovered, true).expect("textfield");

        assert_eq!(appearance.background, default.background);
        assert_eq!(appearance.background, catalog.color("background").expect("background"));
    }
}
