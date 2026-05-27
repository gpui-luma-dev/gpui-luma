use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::AdornerSpec;

use crate::theme::radix::{RadixModeTokens, button_appearance};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Standard,
    Subtle,
    Ghost,
    Prominent,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonFamilyRole {
    #[default]
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait ButtonFamilyTheme: Send + Sync {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultButtonFamilyTheme {
    tokens: ThemeTokens,
}

pub fn default_button_family_theme() -> Arc<dyn ButtonFamilyTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.button_family_theme();
    }
    static THEME: OnceLock<Arc<dyn ButtonFamilyTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultButtonFamilyTheme::default())).clone()
}

impl DefaultButtonFamilyTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

fn radix_style_from_variant(variant: ButtonVariant) -> crate::theme::radix::RadixButtonStyle {
    use crate::theme::radix::RadixButtonStyle;

    match variant {
        ButtonVariant::Prominent => RadixButtonStyle::Primary,
        ButtonVariant::Standard => RadixButtonStyle::Secondary,
        ButtonVariant::Subtle => RadixButtonStyle::Outline,
        ButtonVariant::Ghost => RadixButtonStyle::Ghost,
    }
}

impl ButtonFamilyTheme for DefaultButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        let mode = RadixModeTokens::from_luma_tokens(&self.tokens);
        button_appearance(&mode, radix_style_from_variant(variant), role, size, state)
    }
}
