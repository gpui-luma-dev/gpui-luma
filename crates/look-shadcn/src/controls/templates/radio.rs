use std::sync::Arc;

use gpui_luma::controls::button::ButtonTemplate;
use gpui_luma::controls::control_group::{ControlGroupItemLike, ControlGroupTemplate};
use gpui_luma::controls::radio_button::{RadioButtonData, RadioButtonTheme, ThemedRadioButtonTemplate};
use gpui_luma::controls::radio_group::{RadioGroupLayout, radio_group_buttons_template};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::button::ShadcnButtonStyle;
use crate::controls::radio::radio_button_look;
use crate::look::ShadcnLook;

struct ShadcnStyledRadioButtonTheme {
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
}

impl RadioButtonTheme for ShadcnStyledRadioButtonTheme {
    fn resolve(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::radio_button::RadioButtonPalette {
        let tokens = self.theme.mode_tokens();
        radio_button_look(tokens.as_ref(), self.style, selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn radio_button_theme(theme: ShadcnLook) -> Arc<dyn RadioButtonTheme> {
    radio_button_theme_with_style(theme, ShadcnButtonStyle::Primary)
}

pub fn radio_button_theme_with_style(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn RadioButtonTheme> {
    Arc::new(ShadcnStyledRadioButtonTheme { theme: theme.clone(), style })
}

pub fn radio_button_template(theme: ShadcnLook, style: ShadcnButtonStyle) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
    Arc::new(ThemedRadioButtonTemplate::new(radio_button_theme_with_style(theme, style)))
}

pub fn radio_group_template<T>(
    theme: ShadcnLook,
    style: ShadcnButtonStyle,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_buttons_template(theme.radio_button_template(style), layout)
}
