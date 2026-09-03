use crate::controls::text::TextSelectionState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextFieldState {
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub invalid: bool,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub preferred_column: Option<usize>,
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
        <Self as TextSelectionState>::clamp_cursor(self, len_chars);
    }

    pub fn clear_selection(&mut self) {
        <Self as TextSelectionState>::clear_selection(self);
    }

    pub fn set_cursor(&mut self, cursor: usize, selecting: bool) {
        <Self as TextSelectionState>::set_cursor(self, cursor, selecting);
    }

    pub fn has_selection(&self) -> bool {
        <Self as TextSelectionState>::has_selection(self)
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        <Self as TextSelectionState>::selection_range(self)
    }
}

impl TextSelectionState for TextFieldState {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn set_cursor_raw(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn selection_anchor(&self) -> Option<usize> {
        self.selection_anchor
    }

    fn set_selection_anchor_raw(&mut self, anchor: Option<usize>) {
        self.selection_anchor = anchor;
    }

    fn preferred_column(&self) -> Option<usize> {
        self.preferred_column
    }

    fn set_preferred_column(&mut self, column: Option<usize>) {
        self.preferred_column = column;
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
