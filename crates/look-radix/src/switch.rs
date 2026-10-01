//! Radix switch theme adapter.
//!
//! Geometry matches `@radix-ui/themes` Switch at `--scaling: 1`:
//! - sizes 1–3 → height 16 / 20 / 24; width = height × 1.75; thumb inset 1px
//! - radius = `max(--radius-{1|2} × factor, --radius-thumb)` (pill for medium+)

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use luma::controls::button::ButtonTemplate;
use luma::controls::switch::{SwitchPalette, SwitchScale, SwitchTheme, ThemedSwitchTemplate};
use luma::theme::{ControlSize, InteractionState, MetricTokens};

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::tone::Tone;
use crate::typography::{font_family, label_typography};

/// The Radix Themes switch variant scheme (Classic / Surface / Soft).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Switch. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchSize {
    One,
    #[default]
    Two,
    Three,
}

impl SwitchSize {
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

    /// Track height (`--switch-height`).
    pub fn height(self) -> f32 {
        match self {
            Self::One => 16.0,
            Self::Two => 20.0,
            Self::Three => 24.0,
        }
    }

    /// `--radius-1` for size 1, `--radius-2` for sizes 2–3 (before factor).
    fn radius_base(self) -> f32 {
        match self {
            Self::One => 3.0,
            Self::Two | Self::Three => 4.0,
        }
    }
}

/// Radix `--radius-thumb` for the given theme radius setting.
fn radius_thumb(radius: Radius) -> f32 {
    match radius {
        Radius::None | Radius::Small => 0.5,
        Radius::Medium | Radius::Large | Radius::Full => 9999.0,
    }
}

pub fn resolve_switch_radius(size: SwitchSize, radius: Radius) -> f32 {
    let themed = size.radius_base() * radius.factor();
    themed.max(radius_thumb(radius)).min(size.height() / 2.0)
}

pub fn switch_scale_for(size: SwitchSize, radius: Radius) -> SwitchScale {
    let height = size.height();
    let width = height * 1.75;
    let inset = 1.0;
    let thumb = height - inset * 2.0;
    SwitchScale {
        track_width: width,
        track_height: height,
        track_padding: inset,
        thumb_size: thumb,
        track_radius: resolve_switch_radius(size, radius),
        gap: 8.0,
        label_baseline_shift: 0.0,
    }
}

struct SwitchThemeAdapter {
    look: Look,
    variant: SwitchVariant,
    paint: Paint,
    geometry: Option<(SwitchSize, Radius)>,
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

fn thumb_shadow(on: bool) -> Vec<BoxShadow> {
    let soft = alpha(gpui::black(), if on { 0.12 } else { 0.08 });
    let hairline = alpha(gpui::black(), 0.06);
    vec![
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(3.0),
            spread_radius: px(0.0),
            color: soft,
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: hairline,
            inset: false,
        },
    ]
}

impl SwitchTheme for SwitchThemeAdapter {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette {
        let look = &self.look;
        let high_contrast = self.paint.high_contrast;
        let family = match self.paint.tone {
            Tone::Accent => ScaleFamily::Color,
            Tone::Gray => ScaleFamily::Gray,
        };
        let step = |n| look.resolve_step(family, n).hsla();
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let white = gpui::white();

        let accent_on = if high_contrast {
            step(12)
        } else {
            look.resolve_role(SemanticRole::Primary).hsla()
        };
        let soft_on = if high_contrast { step(6) } else { step(4) };

        let (track_background, mut track_border, thumb_background, thumb_border, thumb_shadow) = if state.disabled {
            let thumb = look.light_gray_step(2);
            (alpha(gray(3), 0.55), alpha(gray(3), 0.55), thumb, thumb, thumb_shadow(false))
        } else {
            match (self.variant, on) {
                (SwitchVariant::Surface, false) => {
                    (alpha(gray(3), 0.55), alpha(gray(5), 0.7), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Surface, true) => (accent_on, accent_on, white, white, thumb_shadow(true)),
                (SwitchVariant::Classic, false) => {
                    (alpha(gray(4), 0.6), alpha(gray(5), 0.55), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Classic, true) => (accent_on, alpha(gray(3), 0.5), white, white, thumb_shadow(true)),
                (SwitchVariant::Soft, false) => {
                    (alpha(gray(3), 0.55), alpha(gray(3), 0.55), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Soft, true) => (soft_on, soft_on, white, white, thumb_shadow(true)),
            }
        };

        if state.focused && !state.disabled {
            track_border = look.resolve_role(SemanticRole::Focus).hsla();
        }

        SwitchPalette {
            track_background,
            track_border,
            thumb_background,
            thumb_border,
            thumb_shadow,
            label_color: if state.disabled {
                look.resolve_role(SemanticRole::MutedForeground).hsla()
            } else {
                look.resolve_role(SemanticRole::Foreground).hsla()
            },
            label_typography: label_typography(size),
            label_font_family: font_family(look),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        let _ = scale_factor;
        if let Some((radix_size, radius)) = self.geometry {
            return switch_scale_for(radix_size, radius);
        }
        let radix_size = match size {
            ControlSize::Sm => SwitchSize::One,
            ControlSize::Md => SwitchSize::Two,
            ControlSize::Lg => SwitchSize::Three,
        };
        switch_scale_for(radix_size, Radius::Medium)
    }
}

pub fn switch_theme(look: &Look) -> Arc<dyn SwitchTheme> {
    switch_theme_with(look, SwitchVariant::default(), Paint::accent())
}

pub fn switch_theme_with(look: &Look, variant: SwitchVariant, paint: Paint) -> Arc<dyn SwitchTheme> {
    Arc::new(SwitchThemeAdapter { look: look.clone(), variant, paint, geometry: None })
}

pub fn switch_theme_for(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
) -> Arc<dyn SwitchTheme> {
    Arc::new(SwitchThemeAdapter { look: look.clone(), variant, paint, geometry: Some((size, radius)) })
}

pub fn switch_template(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<luma::controls::switch::SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_with(look, variant, paint)))
}

pub fn switch_template_for(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
) -> Arc<dyn ButtonTemplate<luma::controls::switch::SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_for(look, variant, paint, size, radius)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(variant: SwitchVariant, on: bool) -> SwitchPalette {
        let look = Look::built_in();
        switch_theme_with(&look, variant, Paint::accent()).resolve(on, InteractionState::default(), ControlSize::Md)
    }

    #[test]
    fn thumbs_keep_light_colors_in_dark_mode() {
        let look = Look::built_in();
        for variant in [SwitchVariant::Classic, SwitchVariant::Surface, SwitchVariant::Soft] {
            let theme = switch_theme_with(&look, variant, Paint::accent());
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
                InteractionState { focused: true, ..Default::default() },
                InteractionState { disabled: true, ..Default::default() },
            ] {
                for on in [false, true] {
                    look.set_mode(luma::theme::ThemeMode::Light);
                    let light = theme.resolve(on, state, ControlSize::Md);
                    look.set_mode(luma::theme::ThemeMode::Dark);
                    let dark = theme.resolve(on, state, ControlSize::Md);
                    assert_eq!(light.thumb_background, dark.thumb_background);
                    assert_eq!(light.thumb_border, dark.thumb_border);
                }
            }
        }
    }

    #[test]
    fn surface_on_uses_primary_fill() {
        let look = Look::built_in();
        assert_eq!(
            palette(SwitchVariant::Surface, true).track_background,
            look.resolve_role(SemanticRole::Primary).hsla()
        );
    }

    #[test]
    fn soft_on_uses_step_four() {
        let look = Look::built_in();
        assert_eq!(
            palette(SwitchVariant::Soft, true).track_background,
            look.resolve_step(ScaleFamily::Color, 4).hsla()
        );
    }

    #[test]
    fn surface_high_contrast_uses_step_twelve() {
        let look = Look::built_in();
        let paint = Paint::accent().high_contrast();
        let on = switch_theme_with(&look, SwitchVariant::Surface, paint).resolve(
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        assert_eq!(on.track_background, look.resolve_step(ScaleFamily::Color, 12).hsla());
    }

    #[test]
    fn medium_radius_pills_the_track() {
        let size = SwitchSize::Two;
        assert_eq!(resolve_switch_radius(size, Radius::Medium), size.height() / 2.0);
        assert!(resolve_switch_radius(size, Radius::None) < 1.0);
    }

    #[test]
    fn size_two_matches_radix_height() {
        let scale = switch_scale_for(SwitchSize::Two, Radius::Medium);
        assert_eq!(scale.track_height, 20.0);
        assert_eq!(scale.track_width, 35.0);
        assert_eq!(scale.thumb_size, 18.0);
    }
}
