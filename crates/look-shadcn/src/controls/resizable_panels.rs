//! Resizable panels property mappings:
//!
//! | Part          | Token              |
//! |---------------|--------------------|
//! | Divider line  | `border`           |
//! | Panel border  | `border`           |
//! | Handle grip   | `border`           |
//! | Grip emphasis | `accent` (layer)   |
//! | Disabled      | `muted-foreground` |

use gpui_luma::controls::resizable_panels::ResizablePanelsAppearance;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

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

declare_look_table! {
    name: resolve_resizable_panels_colors,
    inputs: {
        disabled: bool,
        layer: InteractionLayer,
    },
    output: ResizablePanelsColorTable { border, divider, grip, grip_emphasis },
    matrix: [
        [true] | [_] => "border" | "muted-foreground" | "muted-foreground" | "muted-foreground",

        [false] | [InteractionLayer::Default] => "border" | "border" | "border" | "accent",
        [false] | [InteractionLayer::Hovered] => "border" | "border" | "border" | "first_layer(accent)",
        [false] | [InteractionLayer::Pressed] => "border" | "border" | "border" | "first_layer(accent)",
        [false] | [InteractionLayer::Disabled] => "border" | "border" | "border" | "accent",
    ]
}

pub fn resizable_panels_appearance(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ResizablePanelsAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if mode.catalog.tokens.is_empty() {
        resizable_panels_from_palette(&ctx)
    } else {
        resizable_panels_from_catalog(&ctx).unwrap_or_else(|err| panic!("resizable panels properties: {err}"))
    }
}

pub fn resizable_panels_from_palette(ctx: &AppearanceContext) -> ResizablePanelsAppearance {
    let palette = ctx.palette();
    let disabled = ctx.state.layer() == InteractionLayer::Disabled;

    ResizablePanelsAppearance {
        border: palette.border_default,
        divider: if disabled {
            palette.disabled_foreground
        } else {
            palette.border_default
        },
        grip: if disabled {
            palette.disabled_foreground
        } else {
            palette.border_default
        },
        grip_emphasis: if disabled {
            palette.disabled_foreground
        } else {
            palette.accent_background
        },
        disabled_opacity: if disabled { 0.45 } else { 1.0 },
    }
}

fn resizable_panels_from_catalog(ctx: &AppearanceContext) -> anyhow::Result<ResizablePanelsAppearance> {
    let state = ctx.state;
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "resizable_panels");
    let colors = resolve_resizable_panels_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| ResizablePanelsColorTable::fallback());

    Ok(ResizablePanelsAppearance {
        border: colors.border.hsla(),
        divider: colors.divider.hsla(),
        grip: colors.grip.hsla(),
        grip_emphasis: colors.grip_emphasis.hsla(),
        disabled_opacity: if state.disabled { 0.45 } else { 1.0 },
    })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;

    use gpui_luma::controls::resizable_panels::ResizeHandleSize;
    use gpui_luma::theme::{InteractionLayer, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::LookResolver;
    use super::{
        resizable_panels_appearance,
        resolve_resizable_panels_colors, resolve_resizable_panels_colors_metadata,
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
        ]))
    }

    #[test]
    fn enabled_resizable_panels_use_border_and_accent_grip() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance = resizable_panels_appearance(&mode, ThemeMode::Light, Default::default());
        assert_eq!(appearance.border, catalog.color("border").expect("border"));
        assert_eq!(appearance.grip_emphasis, catalog.color("accent").expect("accent"));
    }

}
