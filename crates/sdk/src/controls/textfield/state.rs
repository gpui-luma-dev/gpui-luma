#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextFieldState {
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub invalid: bool,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
}

impl TextFieldState {
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
        if let Some(anchor) = self.selection_anchor {
            self.selection_anchor = Some(anchor.min(len_chars));
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn set_cursor(&mut self, cursor: usize, selecting: bool) {
        if selecting {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else {
            self.selection_anchor = None;
        }

        self.cursor = cursor;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        if anchor == self.cursor {
            return None;
        }

        Some((anchor.min(self.cursor), anchor.max(self.cursor)))
    }
}

#[cfg(test)]
mod tests {
    use super::TextFieldState;

    #[test]
    fn clamp_cursor_to_text_len() {
        let mut state = TextFieldState { cursor: 10, ..Default::default() };

        state.clamp_cursor(3);
        assert_eq!(state.cursor, 3);
    }

    #[test]
    fn selection_range_normalizes_order() {
        let mut state = TextFieldState { cursor: 2, selection_anchor: Some(7), ..Default::default() };
        assert_eq!(state.selection_range(), Some((2, 7)));

        state.cursor = 7;
        assert_eq!(state.selection_range(), None);
    }
}
