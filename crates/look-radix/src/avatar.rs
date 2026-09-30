//! Non-interactive, look-owned Radix Avatar: image with a short-text or icon fallback.
//! Geometry follows https://www.radix-ui.com/themes/docs/components/avatar.

use gpui::{AnyElement, Div, FontWeight, Hsla, ImageSource, IntoElement, SharedString, div, img, prelude::*, px};
use luma::controls::button::ControlIcon;
use crate::{Look, Radius, Tone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarVariant {
    Solid,
    #[default]
    Soft,
}
impl AvatarVariant {
    pub const ALL: [Self; 2] = [Self::Solid, Self::Soft];
    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Soft => "Soft",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    One,
    Two,
    #[default]
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}
impl AvatarSize {
    pub const ALL: [Self; 9] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
    ];
    pub fn label(self) -> &'static str {
        ["Size 1", "Size 2", "Size 3", "Size 4", "Size 5", "Size 6", "Size 7", "Size 8", "Size 9"][self as usize]
    }
    pub fn diameter(self) -> f32 {
        [24.0, 32.0, 40.0, 48.0, 64.0, 80.0, 96.0, 128.0, 160.0][self as usize]
    }
    fn font_size(self, two_letters: bool) -> f32 {
        if two_letters {
            [12.0, 14.0, 16.0, 18.0, 24.0, 28.0, 28.0, 35.0, 60.0][self as usize]
        } else {
            [14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 28.0, 35.0, 60.0][self as usize]
        }
    }
    pub fn corner_radius(self, radius: Radius) -> f32 {
        if radius == Radius::Full {
            self.diameter() / 2.0
        } else {
            [4.0, 4.0, 6.0, 6.0, 8.0, 12.0, 12.0, 16.0, 16.0][self as usize] * radius.factor()
        }
    }
}

#[derive(Clone)]
enum Fallback {
    Text(SharedString),
    Icon(ControlIcon),
}

/// A display element, not a button. Rebuild it with the owning view when the look changes.
#[derive(Clone)]
pub struct Avatar {
    look: Look,
    image: Option<ImageSource>,
    fallback: Fallback,
    variant: AvatarVariant,
    tone: Tone,
    high_contrast: bool,
    size: AvatarSize,
    radius: Radius,
}
impl Avatar {
    /// Short text is uppercased and limited to two Unicode characters.
    pub fn new(look: &Look, text: impl Into<SharedString>) -> Self {
        Self {
            look: look.clone(),
            image: None,
            fallback: Fallback::Text(short_text(text.into())),
            variant: AvatarVariant::Soft,
            tone: Tone::Accent,
            high_contrast: false,
            size: AvatarSize::Three,
            radius: Radius::Medium,
        }
    }
    /// Image is cropped to cover the avatar. Text/icon remains its loading/error fallback.
    pub fn image(mut self, image: impl Into<ImageSource>) -> Self {
        self.image = Some(image.into());
        self
    }
    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.fallback = Fallback::Icon(icon.into());
        self
    }
    pub fn variant(mut self, variant: AvatarVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.high_contrast = high_contrast;
        self
    }
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    fn colors(&self) -> (Hsla, Hsla) {
        let step = |n| self.tone.step(&self.look, n);
        match (self.variant, self.high_contrast) {
            (AvatarVariant::Solid, true) => (step(12), step(1)),
            (AvatarVariant::Solid, false) => (step(9), self.tone.contrast(&self.look)),
            (AvatarVariant::Soft, high) => (step(3), step(if high { 12 } else { 11 })),
        }
    }
    fn fallback_element(&self) -> AnyElement {
        let (background, foreground) = self.colors();
        let font_size = self.size.font_size(matches!(&self.fallback, Fallback::Text(text) if text.chars().count() > 1));
        let content = match &self.fallback {
            Fallback::Text(text) => div().child(text.clone()).into_any_element(),
            Fallback::Icon(ControlIcon::SvgPath(path)) => gpui::svg()
                .path(path.clone())
                .size(px(self.size.diameter() * 0.6))
                .text_color(foreground)
                .into_any_element(),
            Fallback::Icon(ControlIcon::Lucide(icon)) => {
                luma::infra::icon::lucide_icon(*icon, foreground, self.size.diameter() * 0.6)
            }
        };
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(background)
            .rounded(px(self.size.corner_radius(self.radius)))
            .text_color(foreground)
            .font_family(crate::typography::font_family(&self.look))
            .font_weight(FontWeight::MEDIUM)
            .text_size(px(font_size))
            .line_height(px(font_size))
            .child(content)
            .into_any_element()
    }
}
fn short_text(text: SharedString) -> SharedString {
    text.trim().chars().flat_map(char::to_uppercase).take(2).collect::<String>().into()
}
impl IntoElement for Avatar {
    type Element = Div;
    fn into_element(mut self) -> Div {
        let content = if let Some(source) = self.image.take() {
            let loading = self.clone();
            let failure = self.clone();
            img(source)
                .size_full()
                .rounded(px(self.size.corner_radius(self.radius)))
                .object_fit(gpui::ObjectFit::Cover)
                .with_loading(move || loading.fallback_element())
                .with_fallback(move || failure.fallback_element())
                .into_any_element()
        } else {
            self.fallback_element()
        };
        div()
            .size(px(self.size.diameter()))
            .flex_none()
            .rounded(px(self.size.corner_radius(self.radius)))
            .overflow_hidden()
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sizes_and_radius_match_radix() {
        for (size, diameter) in
            AvatarSize::ALL.into_iter().zip([24.0, 32.0, 40.0, 48.0, 64.0, 80.0, 96.0, 128.0, 160.0])
        {
            assert_eq!(size.diameter(), diameter);
            assert_eq!(size.corner_radius(Radius::None), 0.0);
            assert_eq!(size.corner_radius(Radius::Full), diameter / 2.0);
            assert!(size.corner_radius(Radius::Small) < size.corner_radius(Radius::Medium));
            assert!(size.corner_radius(Radius::Medium) < size.corner_radius(Radius::Large));
        }
        assert_eq!(short_text("bg-long".into()).as_ref(), "BG");
        assert_eq!(short_text("éø".into()).as_ref(), "ÉØ");
    }
    #[test]
    fn colors_follow_mode_tone_and_contrast() {
        let look = Look::built_in();
        for mode in [luma::theme::ThemeMode::Light, luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for tone in [Tone::Accent, Tone::Gray] {
                for variant in AvatarVariant::ALL {
                    let normal = Avatar::new(&look, "V").tone(tone).variant(variant).colors();
                    let high = Avatar::new(&look, "BG").tone(tone).variant(variant).high_contrast(true).colors();
                    assert_eq!(normal.0, tone.step(&look, if variant == AvatarVariant::Solid { 9 } else { 3 }));
                    assert_eq!(high.1, tone.step(&look, if variant == AvatarVariant::Solid { 1 } else { 12 }));
                }
            }
        }
    }
}
