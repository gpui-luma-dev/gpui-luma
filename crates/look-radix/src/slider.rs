//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use gpui_luma::controls::slider::{SliderLook, SliderTemplate, SliderTheme, SliderThumbSize, ThemedSliderTemplate};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState};

use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;

/// Radix Themes slider visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

impl SliderVariant {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Surface, Self::Soft];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Surface => "surface",
            Self::Soft => "soft",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Surface => "Surface",
            Self::Soft => "Soft",
        }
    }
}

/// Radix `size` prop on Slider. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderSize {
    One,
    #[default]
    Two,
    Three,
}

impl SliderSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }

    fn from_control_size(size: ControlSize) -> Self {
        match size {
            ControlSize::Sm => Self::One,
            ControlSize::Md => Self::Two,
            ControlSize::Lg => Self::Three,
        }
    }

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}

struct SliderThemeAdapter {
    look: Look,
    variant: SliderVariant,
    size: Option<SliderSize>,
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

impl SliderTheme for SliderThemeAdapter {
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook {
        let look = &self.look;
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let accent = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
        let layer = state.layer();

        let fill = if state.disabled {
            gray(8)
        } else {
            match (self.variant, layer) {
                // Soft range is `--accent-6` (see radix slider.css).
                (SliderVariant::Soft, _) => accent(6),
                (_, InteractionLayer::Pressed) => accent(11),
                (_, InteractionLayer::Hovered) => accent(10),
                _ => look.resolve_role(SemanticRole::Primary).hsla(),
            }
        };

        // Soft track is gray (not accent). Surface/Classic share gray-a3.
        let track = if state.disabled {
            alpha(gray(4), 0.55)
        } else {
            match self.variant {
                SliderVariant::Soft => alpha(gray(4), 0.65),
                SliderVariant::Surface | SliderVariant::Classic => alpha(gray(3), 0.55),
            }
        };

        let thumb_bg = if state.disabled {
            look.light_gray_step(3)
        } else {
            gpui::white()
        };

        let thumb_border = if state.disabled {
            gray(6)
        } else {
            match self.variant {
                SliderVariant::Soft => alpha(accent(6), 0.35),
                SliderVariant::Surface => alpha(gpui::black(), 0.16),
                SliderVariant::Classic => alpha(gpui::black(), 0.22),
            }
        };

        let thumb_shadow = if state.disabled {
            Vec::new()
        } else {
            match self.variant {
                SliderVariant::Surface => surface_thumb_shadow(),
                SliderVariant::Classic => classic_thumb_shadow(),
                SliderVariant::Soft => soft_thumb_shadow(),
            }
        };

        let radix_size = self.size.unwrap_or_else(|| SliderSize::from_control_size(size));
        let geometry = slider_geometry(look, radix_size, thumb_size);
        SliderLook {
            track_background: track,
            fill_background: fill,
            thumb_background: thumb_bg,
            thumb_border,
            thumb_shadow,
            width: geometry.width.value_px,
            height: geometry.height.value_px,
            track_height: geometry.track_height.value_px,
            thumb_size: geometry.thumb_size.value_px,
            radius: geometry.height.value_px / 2.0,
        }
    }
}

/// Shared metric values and provenance consumed by the Radix slider adapter.
pub fn slider_geometry(
    look: &Look,
    size: SliderSize,
    thumb_size: Option<SliderThumbSize>,
) -> gpui_luma::theme::stylesheet::ResolvedSliderGeometry {
    use gpui_luma::controls::slider::DefaultSliderTheme;
    let theme =
        DefaultSliderTheme::new(gpui_luma::theme::ThemeTokens { metrics: look.metrics(), ..Default::default() });
    let fallback = theme.resolve(size.control_size(), thumb_size, Default::default());
    let thumb_key = thumb_size.map(|size| match size {
        SliderThumbSize::Sm => "1",
        SliderThumbSize::Md => "2",
        SliderThumbSize::Lg => "3",
    });
    look.common_stylesheet().slider.resolve_geometry(size.as_str(), thumb_key, &fallback)
}

fn surface_thumb_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(1.0),
        color: alpha(gpui::black(), 0.16),
        inset: false,
    }]
}

fn classic_thumb_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: alpha(gpui::black(), 0.18),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(3.0),
            spread_radius: px(0.0),
            color: alpha(gpui::black(), 0.08),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(2.0)),
            blur_radius: px(4.0),
            spread_radius: px(-1.0),
            color: alpha(gpui::black(), 0.06),
            inset: false,
        },
    ]
}

fn soft_thumb_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: alpha(gpui::black(), 0.14),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(2.0),
            spread_radius: px(0.0),
            color: alpha(gpui::black(), 0.10),
            inset: false,
        },
    ]
}

pub fn slider_theme(look: &Look) -> Arc<dyn SliderTheme> {
    slider_theme_with(look, SliderVariant::default())
}

pub fn slider_theme_with(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTheme> {
    Arc::new(SliderThemeAdapter { look: look.clone(), variant, size: None })
}

/// Bind Radix's semantic size before adapting to the SDK theme contract.
pub fn slider_theme_for(look: &Look, variant: SliderVariant, size: SliderSize) -> Arc<dyn SliderTheme> {
    Arc::new(SliderThemeAdapter { look: look.clone(), variant, size: Some(size) })
}

pub fn slider_template_for(look: &Look, variant: SliderVariant, size: SliderSize) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_for(look, variant, size)))
}

pub fn slider_template(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_with(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_slider_geometry_preserves_modes_states_sizes_and_explicit_thumbs() {
        let look = Look::built_in();
        for mode in [gpui_luma::theme::ThemeMode::Light, gpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in SliderVariant::ALL {
                let theme = slider_theme_with(&look, variant);
                for (size, height, track, thumb) in [
                    (ControlSize::Sm, 16.0, 4.0, 12.0),
                    (ControlSize::Md, 20.0, 6.0, 16.0),
                    (ControlSize::Lg, 24.0, 8.0, 20.0),
                ] {
                    for bits in 0..16 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        for (selection, expected) in [
                            (None, thumb),
                            (Some(SliderThumbSize::Sm), 12.0),
                            (Some(SliderThumbSize::Md), 16.0),
                            (Some(SliderThumbSize::Lg), 20.0),
                        ] {
                            let painted = theme.resolve(size, selection, state);
                            assert_eq!(
                                (
                                    painted.width,
                                    painted.height,
                                    painted.track_height,
                                    painted.thumb_size,
                                    painted.radius
                                ),
                                (120.0, height, track, expected, height / 2.0)
                            );
                            let inspected = slider_geometry(&look, SliderSize::from_control_size(size), selection);
                            assert_eq!(inspected.width.value_px, painted.width);
                            assert_eq!(inspected.thumb_size.value_px, painted.thumb_size);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_radix_size_retains_numeric_semantics_at_sdk_boundary() {
        let look = Look::built_in();
        let stylesheet = look.common_stylesheet();
        for sizes in [stylesheet.slider.sizes.keys().collect::<Vec<_>>(), stylesheet.switch.sizes.keys().collect()] {
            assert_eq!(sizes.len(), 3);
            assert!(sizes.iter().all(|key| ["1", "2", "3"].contains(&key.as_str())));
        }
        let theme = slider_theme_for(&look, SliderVariant::Surface, SliderSize::One);
        assert_eq!(theme.resolve(ControlSize::Lg, None, Default::default()).height, 16.0);
        let geometry = slider_geometry(&look, SliderSize::Two, Some(SliderThumbSize::Lg));
        assert!(matches!(geometry.height.source,
            gpui_luma::theme::provenance::MetricSource::Authored { key }
            if key == "common.slider.sizes.2.height"));
        assert!(matches!(geometry.thumb_size.source,
            gpui_luma::theme::provenance::MetricSource::Derived { note }
            if note.contains("common.slider.sizes.3.thumb_size")));
    }

    #[test]
    fn shared_slider_geometry_updates_live_themes_without_changing_palettes() {
        let look = Look::built_in();
        let theme = slider_theme(&look);
        let original = theme.resolve(ControlSize::Md, None, Default::default());
        let mut stylesheet = look.common_stylesheet();
        stylesheet.slider.geometry.width = Some(200.0);
        stylesheet.slider.sizes.get_mut("2").unwrap().height = Some(30.0);
        stylesheet.slider.sizes.get_mut("2").unwrap().thumb_size = Some(27.0);
        stylesheet.slider.sizes.get_mut("3").unwrap().thumb_size = Some(31.0);
        look.set_common_stylesheet(stylesheet.clone());
        let custom = theme.resolve(ControlSize::Md, Some(SliderThumbSize::Lg), Default::default());
        assert_eq!((custom.width, custom.height, custom.thumb_size, custom.radius), (200.0, 30.0, 31.0, 15.0));
        assert_eq!(custom.track_background, original.track_background);
        assert_eq!(custom.fill_background, original.fill_background);
        let fork = look.fork();
        fork.set_common_stylesheet(Default::default());
        assert_eq!(look.common_stylesheet(), stylesheet);
    }

    #[test]
    fn thumbs_keep_light_colors_in_dark_mode() {
        let look = Look::built_in();
        for variant in SliderVariant::ALL {
            let theme = slider_theme_with(&look, variant);
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
                InteractionState { focused: true, ..Default::default() },
                InteractionState { disabled: true, ..Default::default() },
            ] {
                look.set_mode(gpui_luma::theme::ThemeMode::Light);
                let light = theme.resolve(ControlSize::Md, None, state);
                look.set_mode(gpui_luma::theme::ThemeMode::Dark);
                let dark = theme.resolve(ControlSize::Md, None, state);
                assert_eq!(light.thumb_background, dark.thumb_background);
            }
        }
    }

    #[test]
    fn surface_fill_uses_primary() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Surface);
        let resolved = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(resolved.fill_background, look.resolve_role(SemanticRole::Primary).hsla());
    }

    #[test]
    fn soft_uses_gray_track_and_accent_fill() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Soft);
        let resolved = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(resolved.fill_background, look.resolve_step(ScaleFamily::Color, 6).hsla());
        assert_ne!(
            resolved.track_background,
            look.resolve_step(ScaleFamily::Color, 4).hsla(),
            "soft track is gray, not accent"
        );
    }

    #[test]
    fn classic_thumb_is_more_elevated_than_surface() {
        let look = Look::built_in();
        let classic = slider_theme_with(&look, SliderVariant::Classic).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let surface = slider_theme_with(&look, SliderVariant::Surface).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        assert!(classic.thumb_shadow.len() > surface.thumb_shadow.len());
    }
}
