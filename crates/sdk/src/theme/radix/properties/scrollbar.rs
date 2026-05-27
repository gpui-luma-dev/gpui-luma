//! Scrollbar property mappings (Radix Themes / shadcn-inspired):
//!
//! | Part  | Token              |
//! |-------|--------------------|
//! | Track | transparent        |
//! | Thumb | `border`           |
//! | Hover | `border` (no change) |
//! | Press | `border` (darkened)  |

use gpui::hsla;

use crate::controls::scrollbar::ScrollbarAppearance;
use crate::controls::scrollbar::ScrollbarOrientation;
use crate::theme::{InteractionLayer, InteractionState};

use super::focus::focus_ring_color;
use super::resolve::resolve_color;
use super::super::catalog::CssTokenMap;
use super::super::color::darken;
use super::super::mode::RadixModeTokens;

pub(crate) fn scrollbar_appearance(
    mode: &RadixModeTokens,
    state: InteractionState,
    orientation: ScrollbarOrientation,
) -> ScrollbarAppearance {
    scrollbar_appearance_from_catalog(&mode.catalog, &mode.metrics, state, orientation)
        .unwrap_or_else(|err| panic!("scrollbar properties: {err}"))
}

pub(crate) fn scrollbar_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    state: InteractionState,
    orientation: ScrollbarOrientation,
) -> anyhow::Result<ScrollbarAppearance> {
    let layer = state.layer();
    let thumb = resolve_color(catalog, "border")?;

    let thumb_background = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted-foreground")?,
        InteractionLayer::Pressed => darken(thumb, 0.08),
        InteractionLayer::Hovered | InteractionLayer::Default => thumb,
    };

    let track_background = if state.disabled {
        resolve_color(catalog, "muted")?
    } else {
        hsla(0.0, 0.0, 0.0, 0.0)
    };

    let length = match orientation {
        ScrollbarOrientation::Horizontal => 260.0,
        ScrollbarOrientation::Vertical => 180.0,
    };

    Ok(ScrollbarAppearance {
        track_background,
        thumb_background,
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

    use crate::controls::scrollbar::ScrollbarOrientation;
    use crate::theme::InteractionState;

    use super::super::super::catalog::CssTokenMap;
    use super::super::super::mode::RadixModeTokens;
    use super::scrollbar_appearance_from_catalog;

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
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let appearance = scrollbar_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
        )
        .expect("scrollbar");

        assert_eq!(appearance.thumb_background, catalog.color("border").expect("border"));
        assert_eq!(appearance.track_background.a, 0.0);
    }

    #[test]
    fn hovered_and_focused_scrollbar_thumb_stay_border() {
        let catalog = sample_catalog();
        let mode = RadixModeTokens::from_catalog(catalog.clone()).expect("catalog");
        let border = catalog.color("border").expect("border");

        let hovered = scrollbar_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            InteractionState { hovered: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
        )
        .expect("scrollbar");
        let focused = scrollbar_appearance_from_catalog(
            &catalog,
            &mode.metrics,
            InteractionState { focused: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
        )
        .expect("scrollbar");

        assert_eq!(hovered.thumb_background, border);
        assert_eq!(focused.thumb_background, border);
        assert_eq!(focused.focus_ring, Some(catalog.color("ring").expect("ring")));
    }
}
