mod template;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};

pub use crate::theme::InteractionState as SwitchState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type Switch = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).data(false).template(default_switch_template())
}
