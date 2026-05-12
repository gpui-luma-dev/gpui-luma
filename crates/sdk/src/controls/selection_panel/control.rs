use gpui::SharedString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionPanelEvent {
    HoverChanged { visible_index: Option<usize> },
    ActivateRow { source_index: usize, visible_index: usize, item_id: SharedString },
    ActiveIndexChanged { visible_index: Option<usize> },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionPanelState {
    active_visible_index: Option<usize>,
    hovered_visible_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelStepDirection {
    Previous,
    Next,
}

impl SelectionPanelState {
    pub fn active_visible_index(&self) -> Option<usize> {
        self.active_visible_index
    }

    pub fn hovered_visible_index(&self) -> Option<usize> {
        self.hovered_visible_index
    }

    pub fn set_hovered_visible_index(&mut self, hovered_visible_index: Option<usize>) -> bool {
        if self.hovered_visible_index == hovered_visible_index {
            return false;
        }
        self.hovered_visible_index = hovered_visible_index;
        true
    }

    pub fn set_active_visible_index(&mut self, active_visible_index: Option<usize>) -> bool {
        if self.active_visible_index == active_visible_index {
            return false;
        }
        self.active_visible_index = active_visible_index;
        true
    }

    pub fn step(&mut self, visible_len: usize, direction: SelectionPanelStepDirection) -> bool {
        if visible_len == 0 {
            return false;
        }

        let current = self.active_visible_index.unwrap_or(0);
        let next = match direction {
            SelectionPanelStepDirection::Previous => {
                if current == 0 {
                    visible_len - 1
                } else {
                    current - 1
                }
            }
            SelectionPanelStepDirection::Next => {
                if current + 1 >= visible_len {
                    0
                } else {
                    current + 1
                }
            }
        };

        self.set_active_visible_index(Some(next))
    }
}
