use gpui::{Entity, SharedString};

use crate::controls::button::{Button, ButtonBuilder, ControlIcon};

pub type IconButton = Entity<Button<()>>;

pub fn new(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
    Button::icon(id, icon)
}
