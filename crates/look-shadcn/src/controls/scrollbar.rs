//! Scrollbar property mappings for shadcn-inspired controls:
//!
//! | Part  | Token              |
//! |-------|--------------------|
//! | Track | transparent (`Ghost`) / muted (`Soft`) |
//! | Thumb | `border`           |
//! | Hover | `border` (no change) |
//! | Press | `border` (darkened)  |

use gpui_luma::controls::scrollbar::ScrollbarLook;
use gpui_luma::controls::scrollbar::ScrollbarOrientation;
use gpui_luma::controls::scrollbar::ScrollbarStyle;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_scrollbar_color_rule, resolve_scrollbar_color_rule,
    resolve_scrollbar_metrics,
};

const DEFAULT_SCROLLBAR_THICKNESS: f32 = 12.0;
const DEFAULT_SCROLLBAR_TRACK_THICKNESS: f32 = 4.0;
const DEFAULT_SCROLLBAR_THUMB_THICKNESS: f32 = 8.0;
const DEFAULT_SCROLLBAR_MIN_THUMB_LENGTH: f32 = 28.0;
const DEFAULT_SCROLLBAR_LENGTH_H: f32 = 260.0;
const DEFAULT_SCROLLBAR_LENGTH_V: f32 = 180.0;

#[derive(Clone, Debug)]
pub struct ScrollbarColorTable {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

impl ScrollbarColorTable {
    pub fn fallback() -> Self {
        Self {
            track_background: ResolvedColor::transparent(),
            thumb_background: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_scrollbar_colors(
    resolver: &LookResolver<'_>,
    style: ScrollbarStyle,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ScrollbarColorTable> {
    resolve_scrollbar_colors_with_stylesheet(resolver, embedded_stylesheet(), style, disabled, layer)
}

pub fn resolve_scrollbar_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ScrollbarStyle,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ScrollbarColorTable> {
    let rule = find_scrollbar_color_rule(stylesheet, scrollbar_style_key(style), disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching scrollbar color rule"))?;
    let colors = resolve_scrollbar_color_rule(resolver, rule, layer)?;
    Ok(ScrollbarColorTable { track_background: colors.track_background, thumb_background: colors.thumb_background })
}

pub fn scrollbar_look(
    mode: &ShadcnModeTokens,
    state: InteractionState,
    orientation: ScrollbarOrientation,
    size: ControlSize,
    style: ScrollbarStyle,
) -> ScrollbarLook {
    let ctx = LookContext::new(mode, ThemeMode::Light, state);
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "scrollbar");
    let colors = resolve_scrollbar_colors(&resolver, style, state.disabled, layer)
        .unwrap_or_else(|_| ScrollbarColorTable::fallback());

    let length = match orientation {
        ScrollbarOrientation::Horizontal => DEFAULT_SCROLLBAR_LENGTH_H,
        ScrollbarOrientation::Vertical => DEFAULT_SCROLLBAR_LENGTH_V,
    };
    let stylesheet = embedded_stylesheet();
    let (thickness, track_thickness, thumb_thickness, min_thumb_length) = stylesheet
        .scrollbar
        .metrics_for_size(size)
        .map(resolve_scrollbar_metrics)
        .map(|metrics| (metrics.thickness, metrics.track_thickness, metrics.thumb_thickness, metrics.min_thumb_length))
        .unwrap_or((
            DEFAULT_SCROLLBAR_THICKNESS,
            DEFAULT_SCROLLBAR_TRACK_THICKNESS,
            DEFAULT_SCROLLBAR_THUMB_THICKNESS,
            DEFAULT_SCROLLBAR_MIN_THUMB_LENGTH,
        ));
    ScrollbarLook {
        track_background: colors.track_background.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        length,
        thickness,
        track_thickness,
        thumb_thickness,
        min_thumb_length,
        radius: metrics.radius.pill,
    }
}

fn scrollbar_style_key(style: ScrollbarStyle) -> &'static str {
    match style {
        ScrollbarStyle::Ghost => "ghost",
        ScrollbarStyle::Soft => "soft",
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::controls::scrollbar::ScrollbarOrientation;
    use gpui_luma::controls::scrollbar::ScrollbarStyle;
    use gpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::scrollbar_look;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn default_scrollbar_uses_border_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );

        assert_eq!(look.thumb_background, catalog.color("border").expect("border"));
        assert_eq!(look.track_background.a, 0.0);
    }

    #[test]
    fn scrollbar_size_changes_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Sm,
            ScrollbarStyle::Ghost,
        );
        let medium = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );
        let large = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Lg,
            ScrollbarStyle::Ghost,
        );

        assert!(small.thickness < medium.thickness);
        assert!(medium.thickness < large.thickness);
        assert!(small.thumb_thickness < medium.thumb_thickness);
        assert!(medium.thumb_thickness < large.thumb_thickness);
    }

    #[test]
    fn hovered_and_focused_scrollbar_thumb_stay_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let border = catalog.color("border").expect("border");

        let hovered = scrollbar_look(
            &mode,
            InteractionState { hovered: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );
        let focused = scrollbar_look(
            &mode,
            InteractionState { focused: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );

        assert_eq!(hovered.thumb_background, border);
        assert_eq!(focused.thumb_background, border);
    }

    #[test]
    fn soft_scrollbar_uses_visible_track() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            gpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Soft,
        );

        assert_eq!(look.track_background, catalog.color("muted").expect("muted"));
        assert_eq!(look.thumb_background, catalog.color("border").expect("border"));
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "scroll_container_tests.rs"]
mod scroll_container_tests;
