//! Inspect metadata for `switch`.

use gpui_luma::theme::{InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{
    AppearanceContext, ColorSource, LookResolver, ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens,
    format_inspect_css_key,
};

pub struct SwitchInspectPalette {
    pub track_background: ResolvedColor,
    pub track_border: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
    pub focus_ring: Option<ResolvedColor>,
}

pub fn inspect_switch_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchInspectPalette {
    let ctx = AppearanceContext::new(mode, theme_mode, state);
    if ctx.catalog().tokens.is_empty() {
        let appearance = gpui_luma_look_shadcn::paint::switch_appearance_from_palette(&ctx, style, on);
        return SwitchInspectPalette {
            track_background: resolved_from_hsla(
                appearance.track_background,
                if on {
                    ColorSource::CssVar { token: "primary".into() }
                } else {
                    ColorSource::CssVar { token: "input".into() }
                },
            ),
            track_border: resolved_from_hsla(appearance.track_border, ColorSource::CssVar { token: "border".into() }),
            thumb_background: resolved_from_hsla(
                appearance.thumb_background,
                ColorSource::CssVar { token: "background".into() },
            ),
            thumb_border: resolved_from_hsla(appearance.thumb_border, ColorSource::CssVar { token: "border".into() }),
            label_color: resolved_from_hsla(appearance.label_color, ColorSource::CssVar { token: "foreground".into() }),
            focus_ring: state
                .focused
                .then(|| resolved_from_hsla(ctx.palette().focus_ring, ColorSource::CssVar { token: "ring".into() })),
        };
    }

    let catalog = ctx.catalog();
    let resolver = LookResolver::new(catalog, theme_mode, "switch_inspect");
    let colors = gpui_luma_look_shadcn::tables::resolve_switch_colors(&resolver, style, on, state.disabled)
        .unwrap_or_else(|_| gpui_luma_look_shadcn::tables::SwitchColorTable::fallback());
    let thumb_border_color = colors.thumb_border;
    let track_border = if on && !state.disabled {
        let note = format!("= {}", format_inspect_css_key(&colors.track_background.source));
        ResolvedColor { value: colors.track_background.value, source: ColorSource::Derived { note } }
    } else {
        resolver.resolve_decl("border").unwrap_or(thumb_border_color.clone())
    };
    let thumb_border = if on && !state.disabled {
        let note = format!("= {}", format_inspect_css_key(&colors.thumb_background.source));
        ResolvedColor { value: colors.thumb_background.value, source: ColorSource::Derived { note } }
    } else {
        thumb_border_color
    };
    let focus_ring = state.focused.then(|| resolver.resolve_decl("ring")).transpose().ok().flatten();

    SwitchInspectPalette {
        track_background: colors.track_background,
        track_border,
        thumb_background: colors.thumb_background,
        thumb_border,
        label_color: colors.label_color,
        focus_ring,
    }
}

fn resolved_from_hsla(value: gpui::Hsla, source: ColorSource) -> ResolvedColor {
    ResolvedColor { value, source }
}

#[derive(Clone, Debug)]
pub struct SwitchInspectMetrics {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub track_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_switch_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: gpui_luma::theme::ControlSize,
) -> SwitchInspectMetrics {
    use gpui_luma::controls::switch::SwitchScale;

    use gpui_luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        pill_radius_metric, spacing_control_metric,
    };

    let ctx = AppearanceContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = SwitchScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    SwitchInspectMetrics {
        track_width: derived_metric(format!("{size_key} track width = control_height × 42/36"), scale.track_width),
        track_height: derived_metric(format!("{size_key} track height = control_height × 22/36"), scale.track_height),
        track_padding: derived_metric(format!("{size_key} track padding = control_height × 2/36"), scale.track_padding),
        thumb_size: derived_metric(format!("{size_key} thumb = control_height × 0.5"), scale.thumb_size),
        gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap),
        track_radius: pill_radius_metric(catalog, scale.track_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
