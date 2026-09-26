use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{Context, SharedString};

use super::{TableControl, TableEvent, TableSelectionMode};

pub(super) type RowKeyFn<T> = Arc<dyn Fn(&T) -> SharedString + Send + Sync>;

/// Invalid keyed selection or row replacement. Failed updates are atomic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableSelectionError {
    KeysNotConfigured,
    DuplicateKey,
    UnknownKey,
    DisabledRow,
    SelectionDisabled,
    TooManySelected,
}

impl std::fmt::Display for TableSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::KeysNotConfigured => "table row keys are not configured",
            Self::DuplicateKey => "table row keys must be unique",
            Self::UnknownKey => "selected key is not in the table",
            Self::DisabledRow => "selected row is disabled",
            Self::SelectionDisabled => "table selection is disabled",
            Self::TooManySelected => "single selection accepts at most one row",
        })
    }
}

impl std::error::Error for TableSelectionError {}

pub(super) fn collect_keys<T>(
    items: &[T],
    key: &dyn Fn(&T) -> SharedString,
) -> Result<Vec<SharedString>, TableSelectionError> {
    let keys: Vec<_> = items.iter().map(key).collect();
    if keys.iter().collect::<HashSet<_>>().len() != keys.len() {
        return Err(TableSelectionError::DuplicateKey);
    }
    Ok(keys)
}

pub(super) struct SelectionSnapshot {
    indices: Vec<usize>,
    keys: Vec<SharedString>,
    active: Option<usize>,
}

impl<T: 'static> TableControl<T> {
    /// Enable stable row identities after spawning the table. Keys must be unique
    /// and stable for each record. Existing index selection is retained. Duplicate
    /// keys leave the previous configuration intact. Future `set_items` calls
    /// preserve selected rows, active row and range anchor by these keys.
    pub fn set_row_key<F, S>(&mut self, key: F, cx: &mut Context<Self>) -> Result<(), TableSelectionError>
    where
        F: Fn(&T) -> S + Send + Sync + 'static,
        S: Into<SharedString>,
    {
        let key: RowKeyFn<T> = Arc::new(move |row| key(row).into());
        let keys = collect_keys(&self.model.items, key.as_ref())?;
        let previous = self.selection_snapshot();
        self.row_key = Some(key);
        self.row_keys = keys;
        self.emit_selection_changes(previous, cx);
        cx.notify();
        Ok(())
    }

    /// Selected stable identities in row order. Empty until keys are configured.
    pub fn selected_keys(&self) -> Vec<SharedString> {
        self.model.selected_indices.iter().filter_map(|&index| self.row_keys.get(index).cloned()).collect()
    }

    /// Owner-driven selection, with no focus, active-row or scroll movement.
    /// Duplicate selections are deduplicated. Unknown/disabled keys and mode
    /// violations reject the entire update. Identical membership emits no event.
    pub fn set_selected_keys<S: Into<SharedString>>(
        &mut self,
        keys: impl IntoIterator<Item = S>,
        cx: &mut Context<Self>,
    ) -> Result<(), TableSelectionError> {
        if self.row_key.is_none() {
            return Err(TableSelectionError::KeysNotConfigured);
        }
        let keys: HashSet<SharedString> = keys.into_iter().map(Into::into).collect();
        let indices: HashMap<_, _> = self.row_keys.iter().enumerate().map(|(index, key)| (key, index)).collect();
        let mut selected = Vec::new();
        for key in keys {
            let &index = indices.get(&key).ok_or(TableSelectionError::UnknownKey)?;
            if !self.model.row_is_enabled(&self.model.items[index]) {
                return Err(TableSelectionError::DisabledRow);
            }
            selected.push(index);
        }
        if self.model.selection_mode == TableSelectionMode::None && !selected.is_empty() {
            return Err(TableSelectionError::SelectionDisabled);
        }
        if self.model.selection_mode == TableSelectionMode::Single && selected.len() > 1 {
            return Err(TableSelectionError::TooManySelected);
        }
        self.set_selected_indices(selected, cx);
        Ok(())
    }

    pub(super) fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot {
            indices: self.model.selected_indices.clone(),
            keys: self.selected_keys(),
            active: self.model.active_index,
        }
    }

    pub(super) fn emit_selection_changes(&self, previous: SelectionSnapshot, cx: &mut Context<Self>) {
        if previous.indices != self.model.selected_indices {
            cx.emit(TableEvent::SelectionChanged { selected_indices: self.model.selected_indices.clone() });
        }
        let keys = self.selected_keys();
        if previous.keys.iter().collect::<HashSet<_>>() != keys.iter().collect::<HashSet<_>>() {
            cx.emit(TableEvent::SelectedKeysChanged { selected_keys: keys });
        }
        if previous.active != self.model.active_index {
            cx.emit(TableEvent::ActiveIndexChanged { active_index: self.model.active_index });
        }
    }

    pub(super) fn remap_selection(&mut self, keys: &[SharedString]) {
        let indices: HashMap<_, _> = keys.iter().enumerate().map(|(index, key)| (key, index)).collect();
        let remap = |index: usize| self.row_keys.get(index).and_then(|key| indices.get(key).copied());
        self.model.selected_indices = self.model.selected_indices.iter().filter_map(|&index| remap(index)).collect();
        self.model.active_index = self.model.active_index.and_then(remap);
        self.selection_anchor = self.selection_anchor.and_then(remap);
    }
}
