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
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_accordion_content_color_rule, find_accordion_trigger_color_rule,
    resolve_accordion_content_color_rule, resolve_accordion_trigger_color_rule,
};

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

pub fn resolve_accordion_trigger_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<AccordionTriggerColorTable> {
    resolve_accordion_trigger_colors_with_stylesheet(resolver, embedded_stylesheet(), disabled, layer)
}

pub fn resolve_accordion_trigger_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<AccordionTriggerColorTable> {
    let rule = find_accordion_trigger_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching accordion trigger color rule"))?;
    let colors = resolve_accordion_trigger_color_rule(resolver, rule, layer)?;
    Ok(AccordionTriggerColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
        border_color: colors.border_color,
        background: colors.background,
    })
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

pub fn resolve_accordion_content_colors(
    resolver: &LookResolver<'_>,
    expanded: bool,
) -> anyhow::Result<AccordionContentColorTable> {
    resolve_accordion_content_colors_with_stylesheet(resolver, embedded_stylesheet(), expanded)
}

pub fn resolve_accordion_content_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    expanded: bool,
) -> anyhow::Result<AccordionContentColorTable> {
    let rule = find_accordion_content_color_rule(stylesheet, expanded)
        .ok_or_else(|| anyhow::anyhow!("no matching accordion content color rule"))?;
    let colors = resolve_accordion_content_color_rule(resolver, rule)?;
    Ok(AccordionContentColorTable { foreground: colors.foreground, background: colors.background })
}

pub fn accordion_trigger_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> AccordionPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_trigger");
    let colors = resolve_accordion_trigger_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| AccordionTriggerColorTable::fallback());

    let mut trigger_typography = typography.text.label;
    super::apply_button_metrics_typography(&mut trigger_typography, mode, size);

    AccordionPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
        border_color: colors.border_color.hsla(),
        icon_color: colors.icon_color.hsla(),
        chevron_color: colors.chevron_color.hsla(),
        typography: trigger_typography,
        font_family: typography.font.sans.family.clone().into(),
    }
}

pub fn accordion_content_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    expanded: bool,
) -> AccordionContentPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_content");
    let colors = resolve_accordion_content_colors(&resolver, expanded)
        .unwrap_or_else(|_| AccordionContentColorTable::fallback());

    AccordionContentPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn trigger_uses_paired_accent_hover_like_sidebar() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = accordion_trigger_palette(&mode, ThemeMode::Light, InteractionState::default(), ControlSize::Md);
        let hovered = accordion_trigger_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
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
