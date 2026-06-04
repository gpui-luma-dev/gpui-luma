//! Accordion property mappings:
//!
//! | Part            | Token              |
//! |-----------------|--------------------|
//! | Trigger label   | `foreground`       |
//! | Trigger hover fg | `accent-foreground` (no bg change) |
//! | Disabled label  | `muted-foreground` |
//! | Chevron         | `muted-foreground` |
//! | Item border     | `border`           |
//! | Content label   | `foreground`       |

use gpui::hsla;

use crate::controls::accordion::{AccordionContentPalette, AccordionPalette};
use crate::theme::{InteractionLayer, InteractionState, ThemeMode};

use super::context::AppearanceContext;
use super::resolve::{resolve_accent_foreground, resolve_color, resolve_label_color};
use super::mode::RadixModeTokens;

pub(crate) fn accordion_trigger_palette(
    mode: &RadixModeTokens,
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
    mode: &RadixModeTokens,
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
    let layer = state.layer();
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let background = match layer {
        InteractionLayer::Disabled
        | InteractionLayer::Default
        | InteractionLayer::Hovered
        | InteractionLayer::Pressed => None,
    };

    let foreground = if state.disabled {
        palette.disabled_foreground
    } else if matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
        palette.primary.foreground
    } else {
        palette.app_foreground
    };

    AccordionPalette {
        background: background.filter(|color| *color != transparent),
        foreground,
        border_color: palette.border_default,
        icon_color: foreground,
        chevron_color: palette.app_muted_foreground,
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
    let layer = state.layer();
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let background = match layer {
        InteractionLayer::Disabled
        | InteractionLayer::Default
        | InteractionLayer::Hovered
        | InteractionLayer::Pressed => None,
    };

    let foreground = if state.disabled {
        resolve_color(catalog, "muted-foreground")?
    } else if matches!(layer, InteractionLayer::Hovered | InteractionLayer::Pressed) {
        resolve_accent_foreground(catalog)?
    } else {
        resolve_label_color(catalog, false)?
    };

    Ok(AccordionPalette {
        background: background.filter(|color| *color != transparent),
        foreground,
        border_color: resolve_color(catalog, "border")?,
        icon_color: foreground,
        chevron_color: resolve_color(catalog, "muted-foreground")?,
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    })
}

fn accordion_content_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<AccordionContentPalette> {
    Ok(AccordionContentPalette { background: None, foreground: resolve_color(ctx.catalog(), "foreground")? })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::theme::{InteractionState, ThemeMode};

    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
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
    fn trigger_uses_accent_foreground_on_hover_without_background() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let default = accordion_trigger_palette(&mode, ThemeMode::Light, InteractionState::default());
        let hovered = accordion_trigger_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        );

        assert_eq!(default.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(default.chevron_color, catalog.color("muted-foreground").expect("muted"));
        assert!(default.background.is_none());
        assert!(hovered.background.is_none());
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }
}
