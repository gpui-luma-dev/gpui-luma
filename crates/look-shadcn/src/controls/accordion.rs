//! Accordion property mappings:
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Trigger label     | `foreground`                       |
//! | Trigger hover bg  | `accent`                           |
//! | Trigger hover fg  | `accent-foreground` (paired)       |
//! | Disabled label    | `muted-foreground`                 |
//! | Chevron           | `muted-foreground` / hover fg      |
//! | Item border       | `border`                           |
//! | Content label     | `foreground`                       |

use gpui::Hsla;

use gpui_luma::controls::accordion::{AccordionContentPalette, AccordionPalette};
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::resolve::{resolve_accent_hover_pair, resolve_color, resolve_label_color};

pub(crate) fn accordion_trigger_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> AccordionPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        accordion_trigger_from_palette(&ctx)
    } else {
        accordion_trigger_from_catalog(&ctx).unwrap_or_else(|err| panic!("accordion trigger properties: {err}"))
    }
}

pub(crate) fn accordion_content_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    _expanded: bool,
) -> AccordionContentPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        accordion_content_from_palette(&ctx)
    } else {
        accordion_content_from_catalog(&ctx).unwrap_or_else(|err| panic!("accordion content properties: {err}"))
    }
}

fn accordion_trigger_from_palette(ctx: &AppearanceContext) -> AccordionPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let typography = ctx.typography();

    let (background, foreground, icon_color, chevron_color) = accordion_trigger_colors(
        state.disabled,
        state.layer(),
        palette.app_foreground,
        palette.app_muted_foreground,
        palette.disabled_foreground,
        palette.accent_background,
        palette.accent_foreground,
    );

    AccordionPalette {
        background,
        foreground,
        border_color: palette.border_default,
        icon_color,
        chevron_color,
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

fn accordion_content_from_palette(ctx: &AppearanceContext) -> AccordionContentPalette {
    let palette = ctx.palette();

    AccordionContentPalette { background: None, foreground: palette.app_foreground }
}

fn accordion_trigger_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<AccordionPalette> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography = ctx.typography();
    let (accent_background, accent_foreground) = resolve_accent_hover_pair(catalog)?;

    let (background, foreground, icon_color, chevron_color) = accordion_trigger_colors(
        state.disabled,
        state.layer(),
        resolve_label_color(catalog, false)?,
        resolve_color(catalog, "muted-foreground")?,
        resolve_color(catalog, "muted-foreground")?,
        accent_background,
        accent_foreground,
    );

    Ok(AccordionPalette {
        background,
        foreground,
        border_color: resolve_color(catalog, "border")?,
        icon_color,
        chevron_color,
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    })
}

fn accordion_trigger_colors(
    disabled: bool,
    layer: InteractionLayer,
    default_foreground: Hsla,
    default_chevron: Hsla,
    disabled_foreground: Hsla,
    accent_background: Hsla,
    accent_foreground: Hsla,
) -> (Option<Hsla>, Hsla, Hsla, Hsla) {
    if disabled {
        return (None, disabled_foreground, disabled_foreground, disabled_foreground);
    }

    match layer {
        InteractionLayer::Disabled | InteractionLayer::Default => {
            (None, default_foreground, default_foreground, default_chevron)
        }
        InteractionLayer::Hovered | InteractionLayer::Pressed => {
            (Some(accent_background), accent_foreground, accent_foreground, accent_foreground)
        }
    }
}

fn accordion_content_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<AccordionContentPalette> {
    Ok(AccordionContentPalette { background: None, foreground: resolve_color(ctx.catalog(), "foreground")? })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::accordion_trigger_palette;

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
    fn trigger_uses_paired_accent_hover_like_navigation_sidebar() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = accordion_trigger_palette(&mode, ThemeMode::Light, InteractionState::default());
        let hovered = accordion_trigger_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(default.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(default.chevron_color, catalog.color("muted-foreground").expect("muted"));
        assert!(default.background.is_none());
        assert_eq!(hovered.background, Some(catalog.color("accent").expect("accent")));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(hovered.icon_color, hovered.foreground);
        assert_eq!(hovered.chevron_color, hovered.foreground);
    }
}
