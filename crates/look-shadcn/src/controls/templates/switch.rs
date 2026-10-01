use std::sync::Arc;

use gpui_luma::controls::button::ButtonTemplate;
use gpui_luma::controls::switch::{SwitchData, SwitchTheme, ThemedSwitchTemplate};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::switch::{switch_look, switch_scale};
use crate::look::ShadcnLook;

struct ShadcnStyledSwitchTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl SwitchTheme for ShadcnStyledSwitchTheme {
    fn resolve(
        &self,
        on: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::switch::SwitchPalette {
        let tokens = self.theme.mode_tokens();
        switch_look(tokens.as_ref(), self.theme.mode(), self.style, on, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> gpui_luma::controls::switch::SwitchScale {
        let tokens = self.theme.mode_tokens();
        switch_scale(tokens.as_ref(), self.theme.mode(), self.style, size, scale_factor)
    }
}

pub fn switch_theme(theme: ShadcnLook) -> Arc<dyn SwitchTheme> {
    switch_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn switch_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn SwitchTheme> {
    Arc::new(ShadcnStyledSwitchTheme { theme: theme.clone(), style })
}

pub fn switch_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_with_style(theme, style)))
}
