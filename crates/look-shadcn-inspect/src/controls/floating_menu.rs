//! Inspect metadata for `floating_menu`.

use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, ColorSource, LookResolver, ResolvedColor, ShadcnModeTokens,
};


pub struct FloatingMenuInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct FloatingMenuInspectMetrics {
    pub radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub padding: gpui_luma_look_shadcn::ResolvedMetric,
    pub min_width: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_height: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_padding_x: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_gap: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_icon_size: gpui_luma_look_shadcn::ResolvedMetric,
    pub item_radius: gpui_luma_look_shadcn::ResolvedMetric,
    pub submenu_offset_x: gpui_luma_look_shadcn::ResolvedMetric,
}

pub fn inspect_floating_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::floating_menu_appearance_from_palette(&ctx, size);
        return FloatingMenuInspectPalette {
            background: resolved_from_hsla(appearance.background, ColorSource::CssVar { token: "popover".into() }),
            foreground: resolved_from_hsla(
                appearance.foreground,
                ColorSource::CssVar { token: "popover-foreground".into() },
            ),
            border: resolved_from_hsla(appearance.border, ColorSource::CssVar { token: "border".into() }),
            item_hover_background: resolved_from_hsla(
                appearance.item_hover_background,
                ColorSource::CssVar { token: "accent".into() },
            ),
            item_hover_foreground: resolved_from_hsla(
                appearance.item_hover_foreground,
                ColorSource::CssVar { token: "accent-foreground".into() },
            ),
            item_disabled_foreground: resolved_from_hsla(
                appearance.item_disabled_foreground,
                ColorSource::CssVar { token: "muted-foreground".into() },
            ),
        };
    }

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "floating_menu_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_floating_menu_colors(&resolver, true).unwrap_or_else(|_| gpui_luma_look_shadcn::tables::FloatingMenuColorTable::fallback());
    FloatingMenuInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        item_hover_background: colors.item_hover_background,
        item_hover_foreground: colors.item_hover_foreground,
        item_disabled_foreground: colors.item_disabled_foreground,
    }
}

pub fn inspect_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectMetrics {
    use crate::metrics::{derived_metric, scaffold_control_metric, spacing_control_metric};
    use gpui_luma_look_shadcn::catalog::SpacingField;

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let appearance = if ctx.catalog().tokens.is_empty() {
        gpui_luma_look_shadcn::paint::floating_menu_appearance_from_palette(&ctx, size)
    } else {
        gpui_luma_look_shadcn::paint::floating_menu_appearance_from_catalog(&ctx, size)
            .unwrap_or_else(|_| gpui_luma_look_shadcn::paint::floating_menu_appearance_from_palette(&ctx, size))
    };
    let catalog = ctx.catalog();

    FloatingMenuInspectMetrics {
        radius: derived_metric("lg = --radius", appearance.radius),
        padding: derived_metric("padding_y × 0.5", appearance.padding),
        min_width: derived_metric("floating menu min width", appearance.min_width),
        item_height: derived_metric("control_height × 0.9", appearance.item_height),
        item_padding_x: derived_metric("padding_x × 0.75", appearance.item_padding_x),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, appearance.item_gap),
        item_icon_size: derived_metric("control_height × 0.44", appearance.item_icon_size),
        item_radius: scaffold_control_metric("sm", "radius", appearance.item_radius),
        submenu_offset_x: derived_metric("gap × 0.5", appearance.submenu_offset_x),
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

