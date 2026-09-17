use gpui::Context;

use super::super::model::{TableModel, TableSelectionMode};
use super::{TableControl, TableEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TableDirection {
    Previous,
    Next,
}

impl<T> TableControl<T>
where
    T: 'static,
{
    pub(super) fn set_active_index_internal(
        &mut self,
        active_index: Option<usize>,
        scroll_direction: Option<TableDirection>,
        cx: &mut Context<Self>,
    ) -> bool {
        let next = match active_index {
            Some(index) if index < self.model.items.len() => Some(index),
            _ => normalize_active_index(
                active_index,
                self.model.items.as_slice(),
                self.model.row_enabled.as_ref(),
                &self.model.selected_indices,
            ),
        };
        if self.model.active_index == next {
            return false;
        }

        self.model.active_index = next;
        if let Some(index) = next {
            self.ensure_page_for_index(index, cx);
            if self.is_paged() {
                // All rows on the current page are visible without scrolling.
            } else {
                self.scroll_active_into_view(index, scroll_direction, cx);
            }
        }
        cx.emit(TableEvent::ActiveIndexChanged { active_index: next });
        cx.notify();
        true
    }

    pub(super) fn commit_select_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let next = compute_next_selected_indices(self.model.selection_mode, &self.model.selected_indices, index);
        if next == self.model.selected_indices {
            return false;
        }

        self.model.selected_indices = next.clone();
        self.model.active_index =
            normalize_active_index(Some(index), self.model.items.as_slice(), self.model.row_enabled.as_ref(), &next);
        cx.emit(TableEvent::SelectionChanged { selected_indices: next });
        cx.notify();
        true
    }

    pub(super) fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(|item| self.model.row_is_enabled(item))
    }
}

pub(crate) fn normalize_model<T>(model: &mut TableModel<T>)
where
    T: 'static,
{
    model.selected_indices = normalize_selected_indices(
        model.selection_mode,
        model.items.as_slice(),
        model.row_enabled.as_ref(),
        &model.selected_indices,
    );
    model.active_index = normalize_active_index(
        model.active_index,
        model.items.as_slice(),
        model.row_enabled.as_ref(),
        &model.selected_indices,
    );
}

pub(crate) fn normalize_selected_indices<T>(
    selection_mode: TableSelectionMode,
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    selected_indices: &[usize],
) -> Vec<usize> {
    let mut normalized = Vec::new();

    if selection_mode == TableSelectionMode::None {
        return normalized;
    }

    for index in selected_indices.iter().copied() {
        if index >= items.len() || !row_enabled(&items[index]) || normalized.contains(&index) {
            continue;
        }

        normalized.push(index);
        if selection_mode == TableSelectionMode::Single {
            break;
        }
    }

    normalized
}

pub(crate) fn normalize_active_index<T>(
    active_index: Option<usize>,
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    selected_indices: &[usize],
) -> Option<usize> {
    if let Some(active_index) = active_index
        && active_index < items.len()
        && row_enabled(&items[active_index])
    {
        return Some(active_index);
    }

    if let Some(selected_index) =
        selected_indices.iter().copied().find(|index| *index < items.len() && row_enabled(&items[*index]))
    {
        return Some(selected_index);
    }

    items.iter().position(row_enabled)
}

pub(super) fn first_enabled_index<T>(items: &[T], row_enabled: &dyn Fn(&T) -> bool) -> Option<usize> {
    items.iter().position(row_enabled)
}

pub(super) fn last_enabled_index<T>(items: &[T], row_enabled: &dyn Fn(&T) -> bool) -> Option<usize> {
    items.iter().rposition(row_enabled)
}

pub(super) fn next_enabled_index<T>(
    items: &[T],
    row_enabled: &dyn Fn(&T) -> bool,
    current_index: Option<usize>,
    direction: TableDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    match current_index {
        None => match direction {
            TableDirection::Next => first_enabled_index(items, row_enabled),
            TableDirection::Previous => last_enabled_index(items, row_enabled),
        },
        Some(start) => match direction {
            TableDirection::Next => (start + 1..len)
                .find(|&index| row_enabled(&items[index]))
                .or_else(|| (start + 1 < len).then(|| start + 1)),
            TableDirection::Previous => {
                (0..start).rfind(|&index| row_enabled(&items[index])).or_else(|| (start > 0).then(|| start - 1))
            }
        },
    }
}

pub(super) fn compute_next_selected_indices(
    selection_mode: TableSelectionMode,
    current: &[usize],
    toggled_index: usize,
) -> Vec<usize> {
    match selection_mode {
        TableSelectionMode::None => Vec::new(),
        TableSelectionMode::Single => vec![toggled_index],
        TableSelectionMode::Multiple => {
            let mut next = current.to_vec();
            if let Some(position) = next.iter().position(|index| *index == toggled_index) {
                next.remove(position);
            } else {
                next.push(toggled_index);
                next.sort_unstable();
            }
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{
        TableDirection, compute_next_selected_indices, next_enabled_index, normalize_active_index,
        normalize_selected_indices,
    };
    use crate::controls::table::TableLabel;
    use crate::controls::table::TableSelectionMode;
    use crate::controls::table::layout::{clamp_page, page_count, visible_item_count};
    fn row_enabled(row: &TableLabel) -> bool {
        row.enabled
    }

    #[test]
    fn none_selection_mode_clears_selection() {
        let items = demo_items();
        assert_eq!(
            normalize_selected_indices(TableSelectionMode::None, &items, &row_enabled, &[0, 1, 2]),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn single_selection_mode_keeps_first_enabled_item() {
        let items = vec![TableLabel::new("one"), TableLabel::new("two").enabled(false), TableLabel::new("three")];

        assert_eq!(normalize_selected_indices(TableSelectionMode::Single, &items, &row_enabled, &[1, 2, 0]), vec![2]);
    }

    #[test]
    fn active_index_falls_back_to_first_selected_then_first_enabled() {
        let items = vec![TableLabel::new("one").enabled(false), TableLabel::new("two"), TableLabel::new("three")];

        assert_eq!(normalize_active_index(Some(0), &items, &row_enabled, &[2]), Some(2));
        assert_eq!(normalize_active_index(None, &items, &row_enabled, &[]), Some(1));
    }

    #[test]
    fn multiple_selection_toggles_index() {
        assert_eq!(compute_next_selected_indices(TableSelectionMode::Multiple, &[1, 3], 2), vec![1, 2, 3]);
        assert_eq!(compute_next_selected_indices(TableSelectionMode::Multiple, &[1, 2, 3], 2), vec![1, 3]);
    }

    #[test]
    fn paging_helpers() {
        assert_eq!(page_count(25, 10), 3);
        assert_eq!(visible_item_count(25, 2, 10), 5);
        assert_eq!(clamp_page(9, 25, 10), 2);
    }

    #[test]
    fn next_enabled_index_does_not_wrap() {
        let items = demo_items();

        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), TableDirection::Previous), None);
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(2), TableDirection::Next), None);
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), TableDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(2), TableDirection::Previous), Some(1));
    }

    #[test]
    fn next_enabled_index_steps_onto_adjacent_disabled_row_at_boundary() {
        let items = vec![TableLabel::new("zero").enabled(false), TableLabel::new("one"), TableLabel::new("two")];

        assert_eq!(next_enabled_index(&items, &row_enabled, Some(1), TableDirection::Previous), Some(0));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), TableDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, &row_enabled, Some(0), TableDirection::Previous), None);
    }

    fn demo_items() -> Vec<TableLabel> {
        vec![
            TableLabel::new(SharedString::from("one")),
            TableLabel::new(SharedString::from("two")),
            TableLabel::new(SharedString::from("three")),
        ]
    }

    #[test]
    fn row_label_round_trips() {
        let row = TableLabel::new("alpha");
        assert_eq!(row.label, SharedString::from("alpha"));
    }
}
