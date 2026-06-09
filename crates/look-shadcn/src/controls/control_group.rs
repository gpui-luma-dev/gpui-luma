//! Control group list chrome — muted surface + border.
//!
//! | Part            | Token                |
//! |-----------------|----------------------|
//! | Enabled bg      | `muted`              |
//! | Disabled bg     | `muted-foreground`   |
//! | Border          | `border`             |

use gpui_luma::controls::control_group::ControlGroupListAppearance;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

#[derive(Clone, Debug)]
pub struct ControlGroupListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

impl ControlGroupListColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
        }
    }
}

declare_look_table! {
    name: resolve_control_group_list_colors,
    inputs: {
        enabled: bool,
    },
    output: ControlGroupListColorTable { background, border },
    matrix: [
        [true] => "muted" | "border",
        [false] => "muted-foreground" | "border",
    ]
}

pub fn control_group_list_appearance(mode: &ShadcnModeTokens, enabled: bool) -> ControlGroupListAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        control_group_list_from_palette(&ctx, enabled)
    } else {
        control_group_list_from_catalog(&ctx, enabled).unwrap_or_else(|err| panic!("control group properties: {err}"))
    }
}

pub fn control_group_list_from_palette(ctx: &AppearanceContext, enabled: bool) -> ControlGroupListAppearance {
    let palette = ctx.palette();
    let metrics = ctx.metrics();

    ControlGroupListAppearance {
        background: if enabled {
            palette.muted_background
        } else {
            palette.disabled_background
        },
        border: palette.border_default,
        radius: metrics.radius(ControlSize::Md),
        padding_x: 6.0,
        padding_y: 4.0,
        gap: 6.0,
    }
}

fn control_group_list_from_catalog(
    ctx: &AppearanceContext,
    enabled: bool,
) -> anyhow::Result<ControlGroupListAppearance> {
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "control_group_list");
    let colors =
        resolve_control_group_list_colors(&resolver, enabled).unwrap_or_else(|_| ControlGroupListColorTable::fallback());

    Ok(ControlGroupListAppearance {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        radius: metrics.radius(ControlSize::Md),
        padding_x: 6.0,
        padding_y: 4.0,
        gap: 6.0,
    })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::{
        control_group_list_appearance,
        resolve_control_group_list_colors_metadata,
    };

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn control_group_uses_muted_surface_and_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let appearance = control_group_list_appearance(&mode, true);

        assert_eq!(appearance.background, catalog.color("muted").expect("muted"));
        assert_eq!(appearance.border, catalog.color("border").expect("border"));
    }

}
