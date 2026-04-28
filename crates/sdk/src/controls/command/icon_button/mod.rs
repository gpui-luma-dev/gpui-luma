use gpui::SharedString;

use crate::controls::command::button::{Button, ButtonBuilder, ControlIcon};

pub struct IconButton;

impl IconButton {
    pub fn new(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> ButtonBuilder<()> {
        Button::icon(id, icon)
    }
}
