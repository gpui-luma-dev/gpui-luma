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

use gpui_luma::controls::floating_menu::FloatingMenuLook;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_floating_menu_surface_color_rule,
    find_floating_menu_surface_elevation_rule, find_floating_menu_trigger_color_rule, resolve_button_metrics_rule,
    resolve_floating_menu_surface_color_rule, resolve_floating_menu_trigger_color_rule,
    resolve_stylesheet_shadow_token,
};

#[derive(Clone, Debug)]
pub struct FloatingMenuColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

impl FloatingMenuColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
            item_hover_background: ResolvedColor::transparent(),
            item_hover_foreground: ResolvedColor::fallback_foreground(),
            item_disabled_foreground: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_floating_menu_colors(
    resolver: &LookResolver<'_>,
    present: bool,
) -> anyhow::Result<FloatingMenuColorTable> {
    resolve_floating_menu_colors_with_stylesheet(resolver, embedded_stylesheet(), present)
}

pub fn resolve_floating_menu_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    _present: bool,
) -> anyhow::Result<FloatingMenuColorTable> {
    let rule = find_floating_menu_surface_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching floating menu surface color rule"))?;
    let colors = resolve_floating_menu_surface_color_rule(resolver, rule)?;
    Ok(FloatingMenuColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        item_hover_background: colors.item_hover_background,
        item_hover_foreground: colors.item_hover_foreground,
        item_disabled_foreground: colors.item_disabled_foreground,
    })
}

#[derive(Clone, Debug)]
pub struct GhostTriggerColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
}

impl GhostTriggerColorTable {
    pub fn fallback() -> Self {
        Self { background: ResolvedColor::transparent(), foreground: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_ghost_trigger_colors(
    resolver: &LookResolver<'_>,
    layer: InteractionLayer,
    disabled: bool,
) -> anyhow::Result<GhostTriggerColorTable> {
    resolve_ghost_trigger_colors_with_stylesheet(resolver, embedded_stylesheet(), layer, disabled)
}

pub fn resolve_ghost_trigger_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
    disabled: bool,
) -> anyhow::Result<GhostTriggerColorTable> {
    let rule = find_floating_menu_trigger_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching ghost trigger color rule"))?;
    let colors = resolve_floating_menu_trigger_color_rule(resolver, rule, layer)?;
    Ok(GhostTriggerColorTable { background: colors.background, foreground: colors.foreground })
}

pub fn floating_menu_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> FloatingMenuLook {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let shadow = floating_menu_elevation_shadow(catalog, embedded_stylesheet());
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "floating_menu");
    let colors = resolve_floating_menu_colors(&resolver, true).unwrap_or_else(|_| FloatingMenuColorTable::fallback());
    let button_metrics = embedded_stylesheet()
        .button
        .metrics_for_size(size)
        .map(|rule| resolve_button_metrics_rule(rule, metrics, size));
    let mut item_typography = typography.text.label;
    if let Some(button_metrics) = button_metrics.as_ref() {
        let base_size = item_typography.size;
        item_typography.size = button_metrics.font_size;
        if base_size > 0.0 {
            item_typography.line_height = button_metrics.font_size * (item_typography.line_height / base_size);
        }
    }

    FloatingMenuLook {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        shadow,
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * 0.5,
        min_width: 180.0,
        item_disabled_foreground: colors.item_disabled_foreground.hsla(),
        item_hover_background: colors.item_hover_background.hsla(),
        item_hover_foreground: colors.item_hover_foreground.hsla(),
        item_typography,
        item_height: metrics.control_height(size) * 0.9,
        item_padding_x: metrics.padding_x(size) * 0.75,
        item_gap: metrics.gap(size),
        item_icon_size: button_metrics
            .as_ref()
            .map(|m| m.icon_size)
            .unwrap_or_else(|| metrics.control_height(size) * 0.44),
        item_radius: metrics.radius.sm,
        disabled_opacity: 0.56,
        submenu_offset_x: metrics.gap(size) * 0.5,
    }
}

fn floating_menu_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
) -> Vec<gpui::BoxShadow> {
    let Some(rule) = find_floating_menu_surface_elevation_rule(stylesheet) else {
        return Vec::new();
    };
    let Some(token) = resolve_stylesheet_shadow_token(&rule.shadow) else {
        return Vec::new();
    };
    parse_shadow_token(catalog, &token).unwrap_or_default()
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::theme::ControlSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::floating_menu_look;

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
            ("shadow-md".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 2px 4px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn floating_menu_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, gpui_luma::theme::ThemeMode::Light, ControlSize::Md);

        assert!(!look.shadow.is_empty());
    }

    #[test]
    fn floating_menu_uses_popover_surface_and_accent_item_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, gpui_luma::theme::ThemeMode::Light, ControlSize::Md);

        assert_eq!(look.background, catalog.color("popover").expect("popover"));
        assert_eq!(look.foreground, catalog.color("popover-foreground").expect("popover-foreground"));
        assert_eq!(look.item_hover_background, catalog.color("accent").expect("accent"));
        assert_eq!(look.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn floating_menu_item_typography_scales_with_size() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let sm = floating_menu_look(&mode, gpui_luma::theme::ThemeMode::Light, ControlSize::Sm);
        let md = floating_menu_look(&mode, gpui_luma::theme::ThemeMode::Light, ControlSize::Md);
        let lg = floating_menu_look(&mode, gpui_luma::theme::ThemeMode::Light, ControlSize::Lg);

        assert!((sm.item_typography.size - 12.0).abs() < f32::EPSILON);
        assert!((md.item_typography.size - 14.0).abs() < f32::EPSILON);
        assert!((lg.item_typography.size - 16.0).abs() < f32::EPSILON);
        assert!(sm.item_typography.size < md.item_typography.size);
        assert!(md.item_typography.size < lg.item_typography.size);
    }
}
