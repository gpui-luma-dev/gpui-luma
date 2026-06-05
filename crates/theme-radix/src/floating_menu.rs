//! Floating menu property mappings (shadcn / tweakcn):
//!
//! | Part          | Token                              |
//! |---------------|------------------------------------|
//! | Surface       | `popover` (fallback `card`)        |
//! | Foreground    | `popover-foreground`               |
//! | Border        | `border`                           |
//! | Item hover bg | `accent`                           |
//! | Item hover fg | `accent-foreground`                |
//! | Item disabled | `muted-foreground`               |

use gpui_luma::controls::floating_menu::FloatingMenuAppearance;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use super::context::AppearanceContext;
use super::elevation::menu_shadow;
use super::resolve::{resolve_accent_hover_pair, resolve_color, resolve_popover_background, resolve_popover_foreground};
use super::mode::RadixModeTokens;

pub(crate) fn floating_menu_appearance(
    mode: &RadixModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuAppearance {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        floating_menu_appearance_from_palette(&ctx, size)
    } else {
        floating_menu_appearance_from_catalog(&ctx, size)
            .unwrap_or_else(|err| panic!("floating menu properties: {err}"))
    }
}

fn floating_menu_appearance_from_palette(ctx: &AppearanceContext, size: ControlSize) -> FloatingMenuAppearance {
    let palette = ctx.palette();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let shadow = menu_shadow(ctx.theme_mode);

    FloatingMenuAppearance {
        background: palette.panel_background,
        foreground: palette.app_foreground,
        border: palette.border_default,
        shadow,
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: palette.disabled_foreground,
        item_hover_background: palette.ghost.hover_background,
        item_hover_foreground: palette.primary.foreground,
        item_typography: typography.text.label,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
        submenu_offset_x: metrics.gap(size) * 0.5,
    }
}

pub(crate) fn floating_menu_appearance_from_catalog(
    ctx: &AppearanceContext,
    size: ControlSize,
) -> anyhow::Result<FloatingMenuAppearance> {
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let shadow = menu_shadow(ctx.theme_mode);

    let (item_hover_background, item_hover_foreground) = resolve_accent_hover_pair(catalog)?;

    Ok(FloatingMenuAppearance {
        background: resolve_popover_background(catalog)?,
        foreground: resolve_popover_foreground(catalog)?,
        border: resolve_color(catalog, "border")?,
        shadow,
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: resolve_color(catalog, "muted-foreground")?,
        item_hover_background,
        item_hover_foreground,
        item_typography: typography.text.label,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: metrics.control_height(size) * 0.44,
        item_radius: metrics.radius.sm,
        submenu_offset_x: metrics.gap(size) * 0.5,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::ControlSize;

    use super::super::context::AppearanceContext;
    use super::super::catalog::CssTokenMap;
    use super::super::mode::RadixModeTokens;
    use super::floating_menu_appearance_from_catalog;

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ]))
    }

    #[test]
    fn floating_menu_uses_popover_surface_and_accent_item_hover() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, Default::default());
        let appearance = floating_menu_appearance_from_catalog(&ctx, ControlSize::Md).expect("floating menu");

        assert_eq!(appearance.background, catalog.color("popover").expect("popover"));
        assert_eq!(appearance.foreground, catalog.color("popover-foreground").expect("popover-foreground"));
        assert_eq!(appearance.item_hover_background, catalog.color("accent").expect("accent"));
        assert_eq!(appearance.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }
}
