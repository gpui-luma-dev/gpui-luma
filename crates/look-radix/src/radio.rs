//! Radix radio theme adapter.
//!
//! Geometry matches `@radix-ui/themes` BaseRadio at `--scaling: 1`:
//! - sizes 1–3 → 14 / 16 / 20 (`space-4` × 0.875 / 1 / 1.25)
//! - indicator is always circular; theme radius does not apply

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use gpui_luma::controls::button::ButtonTemplate;
use gpui_luma::controls::radio_button::{RadioButtonPalette, RadioButtonTheme, RadioScale, ThemedRadioButtonTemplate};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};

use crate::button::Paint;
use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::tone::Tone;
use crate::typography::{font_family, label_typography};

/// The Radix Themes radio variant scheme (Classic / Surface / Soft).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadioVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Radio. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadioSize {
    One,
    #[default]
    Two,
    Three,
}

impl RadioSize {
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

    /// Outer indicator diameter (`--radio-size`).
    pub fn box_size(self) -> f32 {
        match self {
            Self::One => 14.0,
            Self::Two => 16.0,
            Self::Three => 20.0,
        }
    }

    /// Inner dot diameter — Radix scales the indicator with `transform: scale(0.4)`.
    pub fn dot_size(self) -> f32 {
        self.box_size() * 0.4
    }
}

pub fn radio_scale_for(size: RadioSize) -> RadioScale {
    let box_size = size.box_size();
    RadioScale {
        control_radius: box_size / 2.0,
        control_padding_x: 0.0,
        control_padding_y: 0.0,
        indicator_size: box_size,
        height: box_size,
        gap: 8.0,
        label_baseline_shift: 0.0,
        dot_size: size.dot_size(),
    }
}

struct RadioThemeAdapter {
    look: Look,
    variant: RadioVariant,
    paint: Paint,
    size_override: Option<RadioSize>,
}

fn resting_shadow(edge: Hsla, fade: Hsla) -> Vec<BoxShadow> {
    vec![inset(0.0, 0.0, 1.0, edge), inset(1.5, 2.0, 0.0, fade)]
}

fn checked_shadow(highlight: Hsla, shade: Hsla) -> Vec<BoxShadow> {
    vec![inset(0.5, 0.5, 0.0, highlight), inset(-0.5, 0.5, 0.0, shade)]
}

fn inset(offset_y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow {
        offset: point(px(0.0), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        color,
        inset: true,
    }
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

impl RadioButtonTheme for RadioThemeAdapter {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> RadioButtonPalette {
        let look = &self.look;
        let layer = state.layer();
        let high_contrast = self.paint.high_contrast;
        let family = match self.paint.tone {
            Tone::Accent => ScaleFamily::Color,
            Tone::Gray => ScaleFamily::Gray,
        };
        let step = |n| look.resolve_step(family, n).hsla();
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let surface = look.resolve_role(SemanticRole::Surface).hsla();

        let filled = if high_contrast {
            step(12)
        } else {
            match layer {
                InteractionLayer::Pressed => step(11),
                InteractionLayer::Hovered => step(10),
                _ => look.resolve_role(SemanticRole::Primary).hsla(),
            }
        };

        // Surface/Classic selected dots use contrast on the fill; Soft uses accent text steps.
        let surface_dot = if high_contrast {
            step(1)
        } else {
            look.resolve_role(SemanticRole::PrimaryForeground).hsla()
        };
        let soft_dot = if high_contrast { step(12) } else { step(11) };

        let (indicator_background, mut indicator_border, dot_color, indicator_shadow) =
            match (layer, self.variant, checked) {
                (InteractionLayer::Disabled, RadioVariant::Classic, _) => {
                    (alpha(gray(3), 0.45), gray(6), gray(8), Some(resting_shadow(gray(5), alpha(gray(3), 0.4))))
                }
                (InteractionLayer::Disabled, RadioVariant::Soft, _) => {
                    (alpha(gray(3), 0.45), alpha(surface, 0.0), gray(8), None)
                }
                (InteractionLayer::Disabled, RadioVariant::Surface, _) => {
                    (alpha(gray(3), 0.45), gray(6), gray(8), None)
                }
                // Soft: accent-a4 face always; dot only matters when checked (template fades it).
                (_, RadioVariant::Soft, _) => (step(4), step(4), soft_dot, None),
                (_, RadioVariant::Surface, false) => (surface, gray(7), gray(11), None),
                (_, RadioVariant::Surface, true) => (filled, filled, surface_dot, None),
                (_, RadioVariant::Classic, false) => {
                    (surface, gray(7), gray(11), Some(resting_shadow(gray(5), alpha(gray(2), 0.5))))
                }
                (_, RadioVariant::Classic, true) => (
                    filled,
                    filled,
                    surface_dot,
                    Some(checked_shadow(alpha(surface_dot, 0.35), alpha(gpui::black(), 0.28))),
                ),
            };

        if state.focused && !state.disabled {
            indicator_border = look.resolve_role(SemanticRole::Focus).hsla();
        }

        let label_color = if state.disabled {
            look.resolve_role(SemanticRole::MutedForeground).hsla()
        } else {
            look.resolve_role(SemanticRole::Foreground).hsla()
        };

        RadioButtonPalette {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border,
            dot_color,
            label_color,
            label_typography: label_typography(size),
            label_font_family: font_family(look),
            indicator_shadow,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> RadioScale {
        let _ = scale_factor;
        let radix_size = self.size_override.unwrap_or(match size {
            ControlSize::Sm => RadioSize::One,
            ControlSize::Md => RadioSize::Two,
            ControlSize::Lg => RadioSize::Three,
        });
        radio_scale_for(radix_size)
    }
}

pub fn radio_theme(look: &Look, variant: RadioVariant) -> Arc<dyn RadioButtonTheme> {
    radio_theme_with(look, variant, Paint::accent())
}

pub fn radio_theme_with(look: &Look, variant: RadioVariant, paint: Paint) -> Arc<dyn RadioButtonTheme> {
    Arc::new(RadioThemeAdapter { look: look.clone(), variant, paint, size_override: None })
}

pub fn radio_theme_for(look: &Look, variant: RadioVariant, paint: Paint, size: RadioSize) -> Arc<dyn RadioButtonTheme> {
    Arc::new(RadioThemeAdapter { look: look.clone(), variant, paint, size_override: Some(size) })
}

pub fn radio_template(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<gpui_luma::controls::radio_button::RadioButtonData>> {
    Arc::new(ThemedRadioButtonTemplate::new(radio_theme_with(look, variant, paint)))
}

pub fn radio_template_for(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
) -> Arc<dyn ButtonTemplate<gpui_luma::controls::radio_button::RadioButtonData>> {
    Arc::new(ThemedRadioButtonTemplate::new(radio_theme_for(look, variant, paint, size)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(variant: RadioVariant, checked: bool) -> RadioButtonPalette {
        let look = Look::built_in();
        radio_theme(&look, variant).resolve(checked, InteractionState::default(), ControlSize::Md)
    }

    #[test]
    fn surface_fills_with_the_accent_behind_a_gray_edge() {
        let look = Look::built_in();
        let unchecked = palette(RadioVariant::Surface, false);
        let checked = palette(RadioVariant::Surface, true);

        assert_eq!(unchecked.indicator_border, look.resolve_step(ScaleFamily::Gray, 7).hsla());
        assert_eq!(checked.indicator_background, look.resolve_role(SemanticRole::Primary).hsla());
        assert!(unchecked.indicator_shadow.is_none());
    }

    #[test]
    fn soft_keeps_accent_tint_and_uses_step_eleven_dot() {
        let look = Look::built_in();
        let tint = look.resolve_step(ScaleFamily::Color, 4).hsla();
        for checked in [false, true] {
            assert_eq!(palette(RadioVariant::Soft, checked).indicator_background, tint);
        }
        assert_eq!(palette(RadioVariant::Soft, true).dot_color, look.resolve_step(ScaleFamily::Color, 11).hsla());
    }

    #[test]
    fn classic_is_the_only_variant_that_carries_a_shadow() {
        for checked in [false, true] {
            assert!(palette(RadioVariant::Classic, checked).indicator_shadow.is_some());
            assert!(palette(RadioVariant::Surface, checked).indicator_shadow.is_none());
            assert!(palette(RadioVariant::Soft, checked).indicator_shadow.is_none());
        }
    }

    #[test]
    fn surface_high_contrast_uses_step_twelve_fill() {
        let look = Look::built_in();
        let paint = Paint::accent().high_contrast();
        let checked = radio_theme_with(&look, RadioVariant::Surface, paint).resolve(
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        assert_eq!(checked.indicator_background, look.resolve_step(ScaleFamily::Color, 12).hsla());
        assert_eq!(checked.dot_color, look.resolve_step(ScaleFamily::Color, 1).hsla());
    }

    #[test]
    fn size_two_matches_space_four() {
        let scale = radio_scale_for(RadioSize::Two);
        assert_eq!(scale.indicator_size, 16.0);
        assert_eq!(scale.dot_size, 6.4);
    }
}
