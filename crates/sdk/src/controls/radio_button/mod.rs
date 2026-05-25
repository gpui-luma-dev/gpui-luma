mod template;
mod theme;

pub use template::{ThemedRadioButtonTemplate, default_template as default_radio_button_template};
pub use theme::{
    DefaultRadioButtonTheme, RADIO_BUTTON_THEME_USAGE, RadioButtonAppearance, RadioButtonTheme,
    default_radio_button_theme,
};

pub use crate::theme::InteractionState as RadioButtonState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type RadioButton = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).typed(false).template(default_radio_button_template())
}
