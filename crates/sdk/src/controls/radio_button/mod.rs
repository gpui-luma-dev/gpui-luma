mod template;

pub use template::{ThemedRadioButtonTemplate, default_template as default_radio_button_template};

pub use crate::theme::InteractionState as RadioButtonState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type RadioButton = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).data(false).template(default_radio_button_template())
}
