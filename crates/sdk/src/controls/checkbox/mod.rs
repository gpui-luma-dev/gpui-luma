mod template;
mod theme;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use theme::{DefaultCheckboxTheme, CheckboxAppearance, CheckboxTheme, default_checkbox_theme};

pub use crate::theme::InteractionState as CheckboxState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type Checkbox = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).typed(false).template(default_checkbox_template())
}
