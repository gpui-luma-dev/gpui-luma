//! Radix `data-accent-color` override: paint from the accent scale or lock to gray.
//!
//! Radix Themes resolves every chromatic step against the accent scale unless a subtree
//! sets `data-accent-color="gray"`, which forces the neutral scale no matter what the root
//! accent is. `RadixTone` is that switch, so a control can be previewed or placed as either.

use gpui::Hsla;
use luma::theme::ThemeMode;

use crate::look::RadixLook;
use crate::scale::{ScaleFamily, ScaleStep};
use crate::semantic::SemanticRole;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadixTone {
    /// Follows the theme's accent scale.
    #[default]
    Accent,
    /// Locked to the neutral scale, whatever the accent is.
    Gray,
}

impl RadixTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accent => "accent",
            Self::Gray => "gray",
        }
    }

    /// The 12-step scale this tone paints from.
    pub fn family(self) -> ScaleFamily {
        match self {
            Self::Accent => ScaleFamily::Color,
            Self::Gray => ScaleFamily::Gray,
        }
    }

    pub fn step(self, look: &RadixLook, step: ScaleStep) -> Hsla {
        look.resolve_step(self.family(), step).hsla()
    }

    /// Foreground that sits on the tone's solid step 9, Radix's `*-contrast`.
    pub fn contrast(self, look: &RadixLook) -> Hsla {
        match self {
            Self::Accent => look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
            Self::Gray => match look.mode() {
                ThemeMode::Light => look.resolve_step(ScaleFamily::Gray, 1).hsla(),
                ThemeMode::Dark => look.resolve_step(ScaleFamily::Gray, 12).hsla(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{RadixAccent, RadixGray};

    #[test]
    fn gray_tone_ignores_the_accent_scale() {
        let look = RadixLook::built_in();

        assert_eq!(RadixTone::Gray.step(&look, 9), look.resolve_step(ScaleFamily::Gray, 9).hsla());
        assert_ne!(RadixTone::Accent.step(&look, 9), RadixTone::Gray.step(&look, 9));
    }

    #[test]
    fn contrast_stays_legible_on_a_solid_face() {
        let look = RadixLook::built_in();

        for tone in [RadixTone::Accent, RadixTone::Gray] {
            assert!(tone.contrast(&look).l > tone.step(&look, 9).l);
        }
    }

    #[test]
    fn bright_accents_use_dark_contrast_on_solid_faces() {
        for accent in [RadixAccent::Lime, RadixAccent::Mint, RadixAccent::Sky, RadixAccent::Yellow, RadixAccent::Amber]
        {
            let look = RadixLook::from_palettes(accent, RadixGray::Auto, Default::default(), ThemeMode::Light);
            let contrast = RadixTone::Accent.contrast(&look);
            assert!(
                contrast.l < RadixTone::Accent.step(&look, 9).l,
                "{accent:?} light solid needs a darker contrast label"
            );

            look.set_mode(ThemeMode::Dark);
            let contrast = RadixTone::Accent.contrast(&look);
            assert!(
                contrast.l < RadixTone::Accent.step(&look, 9).l,
                "{accent:?} dark solid needs a darker contrast label"
            );
        }
    }
}
