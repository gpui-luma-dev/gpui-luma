mod template;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use crate::theme::InteractionState as CheckboxState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type Checkbox = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).data(false).template(default_checkbox_template())
}
