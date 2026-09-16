use std::sync::Arc;

use luma::controls::button::ButtonTemplate;
use luma::controls::checkbox::{CheckboxData, CheckboxTheme, ThemedCheckboxTemplate};
use luma::theme::{ControlSize, InteractionState};

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::checkbox::checkbox_look;
use crate::look::ShadcnLook;

struct ShadcnStyledCheckboxTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl CheckboxTheme for ShadcnStyledCheckboxTheme {
    fn resolve(
        &self,
        checked: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> luma::controls::checkbox::CheckboxPalette {
        let tokens = self.theme.mode_tokens();
        checkbox_look(tokens.as_ref(), self.style, checked, state, size)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn checkbox_theme(theme: ShadcnLook) -> Arc<dyn CheckboxTheme> {
    checkbox_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn checkbox_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn CheckboxTheme> {
    Arc::new(ShadcnStyledCheckboxTheme { theme: theme.clone(), style })
}

pub fn checkbox_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<CheckboxData>> {
    Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_with_style(theme, style)))
}
