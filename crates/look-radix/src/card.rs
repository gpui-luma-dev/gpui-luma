//! Look-owned, non-interactive content container with Radix Card treatments.
//! Size metrics follow the Radix Themes Card; application content stays external.
use gpui::{AnyElement, BoxShadow, Div, IntoElement, div, point, prelude::*, px};
use luma::theme::ThemeMode;
use crate::{Look, Tone};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Surface,
    Classic,
    Ghost,
}
impl CardVariant {
    pub const ALL: [Self; 3] = [Self::Surface, Self::Classic, Self::Ghost];
    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface",
            Self::Classic => "Classic",
            Self::Ghost => "Ghost",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSize {
    #[default]
    One,
    Two,
    Three,
}
impl CardSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];
    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }
    pub fn padding(self) -> f32 {
        match self {
            Self::One => 12.0,
            Self::Two => 16.0,
            Self::Three => 24.0,
        }
    }
    pub fn radius(self) -> f32 {
        match self {
            Self::One | Self::Two => 8.0,
            Self::Three => 12.0,
        }
    }
}

/// A passive content surface. Use SDK controls inside it for interactive content.
/// Ghost cancels its padding with negative margins, as in Radix Themes.
pub struct Card {
    look: Look,
    variant: CardVariant,
    size: CardSize,
    children: Vec<AnyElement>,
}
impl Card {
    pub fn new(look: &Look) -> Self {
        Self { look: look.clone(), variant: CardVariant::default(), size: CardSize::default(), children: Vec::new() }
    }
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }
}
impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl IntoElement for Card {
    type Element = Div;
    fn into_element(self) -> Div {
        let gray = |step| Tone::Gray.step(&self.look, step);
        let dark = self.look.mode() == ThemeMode::Dark;
        let mut card = div()
            .relative()
            .p(px(self.size.padding()))
            .rounded(px(self.size.radius()))
            .font_family(crate::typography::font_family(&self.look))
            .text_color(gray(12));
        if self.variant == CardVariant::Ghost {
            card = card.m(px(-self.size.padding()));
        } else {
            // Solid scale equivalents of Radix's panel and translucent neutral borders.
            let edge = gray(match self.variant {
                CardVariant::Classic if dark => 6,
                CardVariant::Classic => 3,
                _ => 5,
            });
            let mut shadows = vec![BoxShadow {
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                color: edge,
                inset: true,
            }];
            if self.variant == CardVariant::Classic {
                for (y, blur, spread, opacity) in [
                    (1.0, 1.0, -1.0, if dark { 0.3 } else { 0.05 }),
                    (2.0, 1.0, -2.0, if dark { 0.3 } else { 0.05 }),
                    (1.0, 3.0, -1.0, if dark { 0.2 } else { 0.05 }),
                ] {
                    shadows.push(BoxShadow {
                        offset: point(px(0.0), px(y)),
                        blur_radius: px(blur),
                        spread_radius: px(spread),
                        color: gpui::hsla(0.0, 0.0, 0.0, opacity),
                        inset: false,
                    });
                }
            }
            card = card.bg(gray(if dark { 2 } else { 1 })).shadow(shadows);
        }
        card.children(self.children)
    }
}
