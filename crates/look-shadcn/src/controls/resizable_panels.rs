//! Resizable panels property mappings:
//!
//! | Part          | Token              |
//! |---------------|--------------------|
//! | Divider line  | `border`           |
//! | Panel border  | `border`           |
//! | Handle grip   | `border`           |
//! | Grip emphasis | `accent` (layer)   |
//! | Disabled      | `muted-foreground` |

use gpui_luma::controls::resizable_panels::ResizablePanelsLook;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, find_resizable_panels_color_rule, resolve_resizable_panels_color_rule};

#[derive(Clone, Debug)]
pub struct ResizablePanelsColorTable {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

impl ResizablePanelsColorTable {
    pub fn fallback() -> Self {
        Self {
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
            grip: ResolvedColor::fallback_foreground(),
            grip_emphasis: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_resizable_panels_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ResizablePanelsColorTable> {
    resolve_resizable_panels_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, layer)
}

pub fn resolve_resizable_panels_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ResizablePanelsColorTable> {
    let rule = find_resizable_panels_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching resizable panels color rule"))?;
    let colors = resolve_resizable_panels_color_rule(resolver, rule, layer)?;
    Ok(ResizablePanelsColorTable {
        border: colors.border,
        divider: colors.divider,
        grip: colors.grip,
        grip_emphasis: colors.grip_emphasis,
    })
}

pub fn resizable_panels_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ResizablePanelsLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "resizable_panels").with_stylesheet(mode.stylesheet());
    let colors = resolve_resizable_panels_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| ResizablePanelsColorTable::fallback());

    ResizablePanelsLook {
        border: colors.border.hsla(),
        divider: colors.divider.hsla(),
        grip: colors.grip.hsla(),
        grip_emphasis: colors.grip_emphasis.hsla(),
        disabled_opacity: if state.disabled { 0.45 } else { 1.0 },
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::resizable_panels_look;

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
    fn enabled_resizable_panels_use_border_and_accent_grip() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = resizable_panels_look(&mode, ThemeMode::Light, Default::default());
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert_eq!(look.grip_emphasis, catalog.color("accent").expect("accent"));
    }
}
