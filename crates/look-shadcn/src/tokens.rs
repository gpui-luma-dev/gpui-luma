/// Semantic shadcn color tokens mapped from CSS custom properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnToken {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
}

/// A color token with an optional opacity modifier (Tailwind `/opacity` suffix).
#[derive(Clone, Copy, Debug)]
pub struct ShadcnStyle {
    pub token: ShadcnToken,
    pub opacity: Option<f32>,
}

impl ShadcnToken {
    /// Chainable opacity modifier matching the `/opacity` Tailwind suffix.
    pub fn opacity(self, alpha: f32) -> ShadcnStyle {
        ShadcnStyle { token: self, opacity: Some(alpha) }
    }
}

impl From<ShadcnToken> for ShadcnStyle {
    fn from(token: ShadcnToken) -> Self {
        ShadcnStyle { token, opacity: None }
    }
}

/// Font families defined in the CSS layout (e.g. `--font-sans`, `--font-mono`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnFont {
    Sans,
    Serif,
    Mono,
}

/// Border radius slots computed relative to the base theme `--radius`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnRadius {
    None,
    Sm,
    Md,
    Lg,
    Xl,
}

/// Box shadow elevations defined in the CSS (e.g. `--shadow-sm`, `--shadow-md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnShadow {
    None,
    TwoXs,
    Xs,
    Sm,
    Default,
    Md,
    Lg,
    Xl,
    TwoXl,
}

/// Standardized typographic scale matching Tailwind text sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnTextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    TwoXl,
}

impl ShadcnTextSize {
    pub fn px(self) -> f32 {
        match self {
            Self::Xs => 11.0,
            Self::Sm => 12.5,
            Self::Base => 14.0,
            Self::Lg => 16.0,
            Self::Xl => 18.0,
            Self::TwoXl => 20.0,
        }
    }
}
