use gpui::{App, FocusHandle, Window};

#[derive(Clone)]
pub(super) struct CompositeFocus {
    focus_handle: FocusHandle,
    active_index: Option<usize>,
}

impl CompositeFocus {
    pub(super) fn new(focus_handle: FocusHandle, active_index: Option<usize>) -> Self {
        Self { focus_handle, active_index }
    }

    pub(super) fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub(super) fn set_active_index(&mut self, active_index: Option<usize>) {
        self.active_index = active_index;
    }

    pub(super) fn focus_active_or_first<F>(&self, count: usize, mut focus_child: F)
    where
        F: FnMut(usize),
    {
        if let Some(index) = self.active_index.or_else(|| first_index(count)) {
            focus_child(index);
        }
    }

    pub(super) fn move_to_boundary<F>(&mut self, count: usize, first: bool, mut focus_child: F) -> bool
    where
        F: FnMut(usize),
    {
        let next = if first { first_index(count) } else { last_index(count) };
        let Some(index) = next else {
            return false;
        };

        self.active_index = Some(index);
        focus_child(index);
        true
    }

    pub(super) fn move_active<F>(
        &mut self,
        count: usize,
        direction: CompositeFocusDirection,
        mut focus_child: F,
    ) -> bool
    where
        F: FnMut(usize),
    {
        if count == 0 {
            return false;
        }

        let step = match direction {
            CompositeFocusDirection::Previous => count - 1,
            CompositeFocusDirection::Next => 1,
        };

        let next = match self.active_index {
            Some(index) => (index + step) % count,
            None if direction == CompositeFocusDirection::Previous => count - 1,
            None => 0,
        };

        self.active_index = Some(next);
        focus_child(next);
        true
    }

    pub(super) fn focus_next(&self, window: &mut Window, cx: &mut App) {
        self.focus_handle.focus(window, cx);
        window.focus_next(cx);
    }

    pub(super) fn focus_previous(&self, window: &mut Window, cx: &mut App) {
        self.focus_handle.focus(window, cx);
        window.focus_prev(cx);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompositeFocusDirection {
    Previous,
    Next,
}

fn first_index(count: usize) -> Option<usize> {
    if count == 0 { None } else { Some(0) }
}

fn last_index(count: usize) -> Option<usize> {
    count.checked_sub(1)
}
