#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditableTextPolicy {
    pub multiline: bool,
    pub submit_on_enter: bool,
    pub strip_newlines_on_paste: bool,
    pub allow_tab_character: bool,
    pub clear_on_escape: bool,
}

pub(crate) trait TextSelectionState {
    fn cursor(&self) -> usize;
    fn set_cursor_raw(&mut self, cursor: usize);
    fn selection_anchor(&self) -> Option<usize>;
    fn set_selection_anchor_raw(&mut self, anchor: Option<usize>);
    fn preferred_column(&self) -> Option<usize>;
    fn set_preferred_column(&mut self, column: Option<usize>);

    fn clamp_cursor(&mut self, len_chars: usize) {
        self.set_cursor_raw(self.cursor().min(len_chars));
        if let Some(anchor) = self.selection_anchor() {
            self.set_selection_anchor_raw(Some(anchor.min(len_chars)));
        }
    }

    fn clear_selection(&mut self) {
        self.set_selection_anchor_raw(None);
    }

    fn set_cursor(&mut self, cursor: usize, selecting: bool) {
        if selecting {
            if self.selection_anchor().is_none() {
                self.set_selection_anchor_raw(Some(self.cursor()));
            }
        } else {
            self.clear_selection();
        }

        self.set_cursor_raw(cursor);
    }

    fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor()?;
        if anchor == self.cursor() {
            return None;
        }

        Some((anchor.min(self.cursor()), anchor.max(self.cursor())))
    }
}

pub(crate) fn select_all<S: TextSelectionState>(state: &mut S, len: usize) {
    state.set_selection_anchor_raw(Some(0));
    state.set_cursor_raw(len);
}
