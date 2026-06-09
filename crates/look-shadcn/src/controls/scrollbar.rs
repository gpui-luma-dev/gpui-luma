//! Scrollbar property mappings (Radix Themes / shadcn-inspired):
//!
//! | Part  | Token              |
//! |-------|--------------------|
//! | Track | transparent        |
//! | Thumb | `border`           |
//! | Hover | `border` (no change) |
//! | Press | `border` (darkened)  |

use gpui_luma::controls::scrollbar::ScrollbarAppearance;
use gpui_luma::controls::scrollbar::ScrollbarOrientation;
use gpui_luma::theme::{InteractionLayer, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::focus::focus_ring_color;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

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

declare_look_table! {
    name: resolve_scrollbar_colors,
    inputs: {
        disabled: bool,
        layer: InteractionLayer,
    },
    output: ScrollbarColorTable { track_background, thumb_background },
    matrix: [
        [true] | [_] => "muted" | "muted-foreground",

        [false] | [InteractionLayer::Disabled] => "transparent" | "muted-foreground",
        [false] | [InteractionLayer::Default] => "transparent" | "border",
        [false] | [InteractionLayer::Hovered] => "transparent" | "border",
        [false] | [InteractionLayer::Pressed] => "transparent" | "@darken_border",
    ]
}

pub fn scrollbar_appearance(
    mode: &ShadcnModeTokens,
    state: InteractionState,
    orientation: ScrollbarOrientation,
) -> ScrollbarAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    scrollbar_appearance_from_catalog(&ctx, orientation).unwrap_or_else(|err| panic!("scrollbar properties: {err}"))
}

pub fn scrollbar_appearance_from_catalog(
    ctx: &AppearanceContext,
    orientation: ScrollbarOrientation,
) -> anyhow::Result<ScrollbarAppearance> {
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "scrollbar");
    let colors =
        resolve_scrollbar_colors(&resolver, state.disabled, layer).unwrap_or_else(|_| ScrollbarColorTable::fallback());

    let length = match orientation {
        ScrollbarOrientation::Horizontal => 260.0,
        ScrollbarOrientation::Vertical => 180.0,
    };

    Ok(ScrollbarAppearance {
        track_background: colors.track_background.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        focus_ring: state.focused.then(|| focus_ring_color(catalog)).transpose()?,
        length,
        thickness: 12.0,
        track_thickness: 4.0,
        thumb_thickness: 8.0,
        min_thumb_length: 28.0,
        radius: metrics.radius.pill,
    })
}

#[cfg(test)]
mod tests {


    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::controls::scrollbar::ScrollbarOrientation;
    use gpui_luma::theme::InteractionState;

    use crate::appearance_context::AppearanceContext;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::{resolve_scrollbar_colors_metadata, scrollbar_appearance_from_catalog};

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
        let ctx = AppearanceContext::new(&mode, gpui_luma::theme::ThemeMode::Light, InteractionState::default());
        let appearance = scrollbar_appearance_from_catalog(&ctx, ScrollbarOrientation::Vertical).expect("scrollbar");

        assert_eq!(appearance.thumb_background, catalog.color("border").expect("border"));
        assert_eq!(appearance.track_background.a, 0.0);
    }

    #[test]
    fn hovered_and_focused_scrollbar_thumb_stay_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let border = catalog.color("border").expect("border");

        let hovered = scrollbar_appearance_from_catalog(
            &AppearanceContext::new(
                &mode,
                gpui_luma::theme::ThemeMode::Light,
                InteractionState { hovered: true, ..InteractionState::default() },
            ),
            ScrollbarOrientation::Vertical,
        )
        .expect("scrollbar");
        let focused = scrollbar_appearance_from_catalog(
            &AppearanceContext::new(
                &mode,
                gpui_luma::theme::ThemeMode::Light,
                InteractionState { focused: true, ..InteractionState::default() },
            ),
            ScrollbarOrientation::Vertical,
        )
        .expect("scrollbar");

        assert_eq!(hovered.thumb_background, border);
        assert_eq!(focused.thumb_background, border);
        assert_eq!(focused.focus_ring, Some(catalog.color("ring").expect("ring")));
    }

}
