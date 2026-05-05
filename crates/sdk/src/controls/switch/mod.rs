mod template;
mod theme;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};
pub use theme::{DefaultSwitchTheme, SWITCH_THEME_USAGE, SwitchAppearance, SwitchTheme, default_switch_theme};

pub use crate::controls::motion::{MotionEasing as SwitchMotionEasing, MotionSpec as SwitchThumbMotion};

pub use crate::theme::InteractionState as SwitchState;

use gpui::{Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder};

pub type Switch = Entity<Button<bool>>;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).data(false).template(default_switch_template())
}
