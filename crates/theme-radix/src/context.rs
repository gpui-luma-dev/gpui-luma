use gpui_luma::theme::{InteractionState, LumaTypography, MetricTokens, ThemeMode};

use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;
use super::palette::RadixPalette;

/// Bundles styling dependencies and interactive state for control appearance resolution.
pub struct AppearanceContext<'a> {
    pub tokens: &'a RadixModeTokens,
    pub theme_mode: ThemeMode,
    pub state: InteractionState,
}

impl<'a> AppearanceContext<'a> {
    pub fn new(tokens: &'a RadixModeTokens, theme_mode: ThemeMode, state: InteractionState) -> Self {
        Self { tokens, theme_mode, state }
    }

    pub fn catalog(&self) -> &CssTokenMap {
        &self.tokens.catalog
    }

    pub fn palette(&self) -> &RadixPalette {
        &self.tokens.palette
    }

    pub fn metrics(&self) -> &MetricTokens {
        &self.tokens.metrics
    }

    pub fn typography(&self) -> &LumaTypography {
        &self.tokens.typography
    }
}
