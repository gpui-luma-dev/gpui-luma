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

use gpui_luma::controls::accordion::{AccordionContentPalette, AccordionPalette};
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

#[derive(Clone, Debug)]
pub struct AccordionTriggerColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub border_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionTriggerColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            chevron_color: ResolvedColor::fallback_foreground(),
            border_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

declare_look_table! {
    name: resolve_accordion_trigger_colors,
    inputs: {
        disabled: bool,
        layer: InteractionLayer,
    },
    output: AccordionTriggerColorTable { foreground, icon_color, chevron_color, border_color, background },
    matrix: [
        [true] | [_] => "muted-foreground" | "muted-foreground" | "muted-foreground" | "border" | None,

        [false] | [InteractionLayer::Default] => "foreground" | "foreground" | "muted-foreground" | "border" | None,
        [false] | [InteractionLayer::Disabled] => "foreground" | "foreground" | "muted-foreground" | "border" | None,
        [false] | [InteractionLayer::Hovered] => "accent-foreground" | "accent-foreground" | "accent-foreground" | "border" | "accent",
        [false] | [InteractionLayer::Pressed] => "accent-foreground" | "accent-foreground" | "accent-foreground" | "border" | "accent",
    ]
}

#[derive(Clone, Debug)]
pub struct AccordionContentColorTable {
    pub foreground: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionContentColorTable {
    pub fn fallback() -> Self {
        Self { foreground: ResolvedColor::fallback_foreground(), background: None }
    }
}

declare_look_table! {
    name: resolve_accordion_content_colors,
    inputs: {
        expanded: bool,
    },
    output: AccordionContentColorTable { foreground, background },
    matrix: [
        [true] => "foreground" | None,
        [false] => "foreground" | None,
    ]
}

pub fn accordion_trigger_palette(
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

pub fn accordion_content_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    expanded: bool,
) -> AccordionContentPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        accordion_content_from_palette(&ctx)
    } else {
        accordion_content_from_catalog(&ctx, expanded).unwrap_or_else(|err| panic!("accordion content properties: {err}"))
    }
}

pub fn accordion_trigger_from_palette(ctx: &AppearanceContext) -> AccordionPalette {
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

pub fn accordion_content_from_palette(ctx: &AppearanceContext) -> AccordionContentPalette {
    let palette = ctx.palette();

    AccordionContentPalette { background: None, foreground: palette.app_foreground }
}

fn accordion_trigger_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<AccordionPalette> {
    let state = ctx.state;
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_trigger");
    let colors = resolve_accordion_trigger_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| AccordionTriggerColorTable::fallback());

    Ok(AccordionPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
        border_color: colors.border_color.hsla(),
        icon_color: colors.icon_color.hsla(),
        chevron_color: colors.chevron_color.hsla(),
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    })
}

fn accordion_trigger_colors(
    disabled: bool,
    layer: InteractionLayer,
    default_foreground: gpui::Hsla,
    default_chevron: gpui::Hsla,
    disabled_foreground: gpui::Hsla,
    accent_background: gpui::Hsla,
    accent_foreground: gpui::Hsla,
) -> (Option<gpui::Hsla>, gpui::Hsla, gpui::Hsla, gpui::Hsla) {
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

fn accordion_content_from_catalog(ctx: &AppearanceContext, expanded: bool) -> anyhow::Result<AccordionContentPalette> {
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_content");
    let colors =
        resolve_accordion_content_colors(&resolver, expanded).unwrap_or_else(|_| AccordionContentColorTable::fallback());

    Ok(AccordionContentPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
    })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;

    use gpui_luma::theme::{InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::ColorSource;
    use super::{
        accordion_trigger_palette, resolve_accordion_content_colors_metadata,
        resolve_accordion_trigger_colors_metadata,
    };

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
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
