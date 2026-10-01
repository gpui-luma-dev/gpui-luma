//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use luma::controls::slider::{SliderLook, SliderTemplate, SliderTheme, SliderThumbSize, ThemedSliderTemplate};
use luma::theme::{ControlSize, InteractionLayer, InteractionState};

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

pub fn slider_theme(look: &Look) -> Arc<dyn SliderTheme> {
    slider_theme_with(look, SliderVariant::default())
}

pub fn slider_theme_with(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTheme> {
    Arc::new(SliderThemeAdapter { look: look.clone(), variant })
}

pub fn slider_template(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_with(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
                look.set_mode(luma::theme::ThemeMode::Light);
                let light = theme.resolve(ControlSize::Md, None, state);
                look.set_mode(luma::theme::ThemeMode::Dark);
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
