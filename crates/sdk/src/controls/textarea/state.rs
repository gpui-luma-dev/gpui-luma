#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextAreaState {
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub invalid: bool,
    pub cursor: usize,
}

impl TextAreaState {
    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.focus_visible = false;
        }
    }

    pub fn set_focus_visible(&mut self, focus_visible: bool) {
        self.focus_visible = focus_visible;
    }

    pub fn set_invalid(&mut self, invalid: bool) {
        self.invalid = invalid;
    }

    pub fn clamp_cursor(&mut self, len_chars: usize) {
        self.cursor = self.cursor.min(len_chars);
    }
}
