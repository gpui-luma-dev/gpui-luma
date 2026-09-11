//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use luma::controls::slider::{SliderLook, SliderTheme, SliderThumbSize};
use luma::theme::{ControlSize, InteractionLayer, InteractionState};

use crate::look::RadixLook;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;

/// Radix Themes slider visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadixSliderVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

impl RadixSliderVariant {
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

struct RadixSliderTheme {
    look: RadixLook,
    variant: RadixSliderVariant,
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

impl SliderTheme for RadixSliderTheme {
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
                (RadixSliderVariant::Soft, _) => accent(6),
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
                RadixSliderVariant::Soft => alpha(gray(4), 0.65),
                RadixSliderVariant::Surface | RadixSliderVariant::Classic => alpha(gray(3), 0.55),
            }
        };

        let thumb_bg = if state.disabled {
            gray(3)
        } else {
            look.resolve_role(SemanticRole::Background).hsla()
        };

        let thumb_border = if state.disabled {
            gray(6)
        } else {
            match self.variant {
                RadixSliderVariant::Soft => alpha(accent(6), 0.35),
                RadixSliderVariant::Surface => alpha(gpui::black(), 0.16),
                RadixSliderVariant::Classic => alpha(gpui::black(), 0.22),
            }
        };

        let thumb_shadow = if state.disabled {
            Vec::new()
        } else {
            match self.variant {
                RadixSliderVariant::Surface => surface_thumb_shadow(),
                RadixSliderVariant::Classic => classic_thumb_shadow(),
                RadixSliderVariant::Soft => soft_thumb_shadow(),
            }
        };

        let (height, track_height, default_thumb) = match size {
            ControlSize::Sm => (16.0, 4.0, 12.0),
            ControlSize::Md => (20.0, 6.0, 16.0),
            ControlSize::Lg => (24.0, 8.0, 20.0),
        };
        let thumb = match thumb_size {
            Some(SliderThumbSize::Sm) => 12.0,
            Some(SliderThumbSize::Md) => 16.0,
            Some(SliderThumbSize::Lg) => 20.0,
            None => default_thumb,
        };

        SliderLook {
            track_background: track,
            fill_background: fill,
            thumb_background: thumb_bg,
            thumb_border,
            thumb_shadow,
            width: 120.0,
            height,
            track_height,
            thumb_size: thumb,
            radius: height / 2.0,
        }
    }
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

pub fn slider_theme(look: Arc<RadixLook>) -> Arc<dyn SliderTheme> {
    slider_theme_with(look, RadixSliderVariant::default())
}

pub fn slider_theme_with(look: Arc<RadixLook>, variant: RadixSliderVariant) -> Arc<dyn SliderTheme> {
    Arc::new(RadixSliderTheme { look: look.as_ref().clone(), variant })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_fill_uses_primary() {
        let look = Arc::new(RadixLook::built_in());
        let theme = slider_theme_with(Arc::clone(&look), RadixSliderVariant::Surface);
        let resolved = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(resolved.fill_background, look.resolve_role(SemanticRole::Primary).hsla());
    }

    #[test]
    fn soft_uses_gray_track_and_accent_fill() {
        let look = Arc::new(RadixLook::built_in());
        let theme = slider_theme_with(Arc::clone(&look), RadixSliderVariant::Soft);
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
        let look = Arc::new(RadixLook::built_in());
        let classic = slider_theme_with(Arc::clone(&look), RadixSliderVariant::Classic).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let surface = slider_theme_with(Arc::clone(&look), RadixSliderVariant::Surface).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        assert!(classic.thumb_shadow.len() > surface.thumb_shadow.len());
    }
}
