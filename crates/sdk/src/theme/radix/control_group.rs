//! Control group list chrome — muted surface + border.

use crate::controls::control_group::ControlGroupListAppearance;
use crate::theme::ControlSize;

use super::resolve::resolve_color;
use super::mode::RadixModeTokens;

pub(crate) fn control_group_list_appearance(mode: &RadixModeTokens, enabled: bool) -> ControlGroupListAppearance {
    if mode.catalog.tokens.is_empty() {
        control_group_list_from_palette(mode, enabled)
    } else {
        control_group_list_from_catalog(&mode.catalog, mode, enabled)
            .unwrap_or_else(|err| panic!("control group properties: {err}"))
    }
}

fn control_group_list_from_palette(mode: &RadixModeTokens, enabled: bool) -> ControlGroupListAppearance {
    let palette = &mode.palette;
    let metrics = &mode.metrics;

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
    catalog: &super::catalog::CssTokenMap,
    mode: &RadixModeTokens,
    enabled: bool,
) -> anyhow::Result<ControlGroupListAppearance> {
    let metrics = &mode.metrics;

    Ok(ControlGroupListAppearance {
        background: if enabled {
            resolve_color(catalog, "muted")?
        } else {
            resolve_color(catalog, "muted-foreground")?
        },
        border: resolve_color(catalog, "border")?,
        radius: metrics.radius(ControlSize::Md),
        padding_x: 6.0,
        padding_y: 4.0,
        gap: 6.0,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::control_group_list_appearance;

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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = control_group_list_appearance(&mode, true);

        assert_eq!(appearance.background, catalog.color("muted").expect("muted"));
        assert_eq!(appearance.border, catalog.color("border").expect("border"));
    }
}
