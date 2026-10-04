//! Control group list chrome — muted surface + border.
//!
//! | Part            | Token                |
//! |-----------------|----------------------|
//! | Enabled bg      | `muted`              |
//! | Disabled bg     | `muted-foreground`   |
//! | Border          | `border`             |

use gpui_luma::controls::control_group::ControlGroupListLook;
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, find_control_group_list_color_rule, resolve_control_group_list_color_rule};

#[derive(Clone, Debug)]
pub struct ControlGroupListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

impl ControlGroupListColorTable {
    pub fn fallback() -> Self {
        Self { background: ResolvedColor::transparent(), border: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_control_group_list_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<ControlGroupListColorTable> {
    resolve_control_group_list_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_control_group_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ControlGroupListColorTable> {
    let rule = find_control_group_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching control group list color rule"))?;
    let colors = resolve_control_group_list_color_rule(resolver, rule)?;
    Ok(ControlGroupListColorTable { background: colors.background, border: colors.border })
}

pub fn control_group_list_look(mode: &ShadcnModeTokens, enabled: bool) -> ControlGroupListLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "control_group_list").with_stylesheet(mode.stylesheet());
    let colors = resolve_control_group_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| ControlGroupListColorTable::fallback());

    let mut look = ControlGroupListLook {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        radius: metrics.radius(ControlSize::Md),
        padding_x: 6.0,
        padding_y: 4.0,
        gap: 6.0,
    };
    let geometry = mode.stylesheet().common.control_group.resolve_geometry(
        "md",
        gpui_luma::theme::stylesheet::ControlGroupGeometry {
            padding_x: look.padding_x,
            padding_y: look.padding_y,
            gap: look.gap,
        },
    );
    look.padding_x = geometry.padding_x.value_px;
    look.padding_y = geometry.padding_y.value_px;
    look.gap = geometry.gap.value_px;

    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::control_group_list_look;

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
        let look = control_group_list_look(&mode, true);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert_eq!(look.border, catalog.color("border").expect("border"));
    }
}
