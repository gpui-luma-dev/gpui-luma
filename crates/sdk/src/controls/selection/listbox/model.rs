use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

/// Selection policy for pointer and keyboard input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionMode {
    /// Navigation and activation without selection.
    None,
    SingleRequired,
    SingleAllowNone,
    /// Clicking an item or pressing Space independently toggles its selection.
    Multiple,
    /// Plain selection replaces; modifiers toggle or extend a keyed range.
    Extended,
}

impl SelectionMode {
    pub fn allows_multiple(self) -> bool {
        matches!(self, Self::Multiple | Self::Extended)
    }

    fn is_single(self) -> bool {
        matches!(self, Self::SingleRequired | Self::SingleAllowNone)
    }
}

/// Runtime selection settings. Flags are retained across mode changes but only
/// affect the modes documented on each field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionPolicy {
    pub mode: SelectionMode,
    /// Repeated selection clears the item in SingleAllowNone only.
    pub toggle_off: bool,
    /// Navigation selects its target in either single-selection mode.
    /// Enabling this does not immediately change selection.
    pub selection_follows_active: bool,
}

impl SelectionPolicy {
    pub fn new(mode: SelectionMode) -> Self {
        Self { mode, toggle_off: false, selection_follows_active: false }
    }
}

/// Semantic modifiers; the binding maps physical platform keys to these flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListBoxSelectionModifiers {
    pub toggle: bool,
    pub extend: bool,
}

/// Invalid collection or programmatic selection. Failed operations are atomic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListBoxError {
    DuplicateKey,
    UnknownKey,
    DisabledItem,
    SelectionRequired,
    TooManySelected,
    SelectionDisabled,
}

impl fmt::Display for ListBoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SelectionDisabled => "listbox selection is disabled",
            Self::DuplicateKey => "listbox snapshot contains duplicate keys",
            Self::UnknownKey => "listbox selection key is not in the snapshot",
            Self::DisabledItem => "listbox selection targets a disabled item",
            Self::TooManySelected => "listbox single-selection mode accepts at most one key",
            Self::SelectionRequired => "listbox requires an enabled item to remain selected",
        })
    }
}

impl std::error::Error for ListBoxError {}

/// Owned items with identity and eligibility captured at construction.
pub struct ListBoxSnapshot<T, K> {
    items: Vec<T>,
    keys: Vec<K>,
    enabled: Vec<bool>,
    indices: HashMap<K, usize>,
}

impl<T, K: Clone + Eq + Hash> ListBoxSnapshot<T, K> {
    pub fn try_new(items: impl IntoIterator<Item = T>, key: impl Fn(&T) -> K) -> Result<Self, ListBoxError> {
        Self::try_with_enabled(items, key, |_| true)
    }

    pub fn try_with_enabled(
        items: impl IntoIterator<Item = T>,
        key: impl Fn(&T) -> K,
        enabled: impl Fn(&T) -> bool,
    ) -> Result<Self, ListBoxError> {
        let items: Vec<T> = items.into_iter().collect();
        let mut keys = Vec::with_capacity(items.len());
        let mut eligibility = Vec::with_capacity(items.len());
        let mut indices = HashMap::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            let key = key(item);
            if indices.insert(key.clone(), index).is_some() {
                return Err(ListBoxError::DuplicateKey);
            }
            keys.push(key);
            eligibility.push(enabled(item));
        }
        Ok(Self { items, keys, enabled: eligibility, indices })
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn item(&self, key: &K) -> Option<&T> {
        self.indices.get(key).map(|index| &self.items[*index])
    }

    fn is_enabled(&self, key: &K) -> bool {
        self.indices.get(key).is_some_and(|index| self.enabled[*index])
    }

    fn first_enabled(&self) -> Option<K> {
        self.keys.iter().zip(&self.enabled).find(|(_, enabled)| **enabled).map(|(key, _)| key.clone())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListBoxItemState {
    pub selected: bool,
    pub active: bool,
    pub enabled: bool,
}

pub struct ListBoxVisibleItem<'a, T, K> {
    pub key: K,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub state: ListBoxItemState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListBoxNavigation {
    Previous,
    Next,
    First,
    Last,
}

/// Semantic input, independent of physical layout and GPUI event types.
#[derive(Clone, Debug)]
pub enum ListBoxInput<K> {
    /// Select one item, toggle in `Multiple`, or move active only in `None`.
    /// SingleAllowNone also toggles when the policy's toggle_off flag is enabled.
    Select(K),
    SelectActive,
    /// Extended-mode modifiers; other modes retain their ordinary selection behavior.
    SelectWithModifiers {
        key: K,
        modifiers: ListBoxSelectionModifiers,
    },
    SelectActiveWithModifiers(ListBoxSelectionModifiers),
    /// Select every enabled item in `Multiple` or `Extended` mode.
    SelectAll,
    /// Clear selection unless an enabled item is required.
    ClearSelection,
    Navigate(ListBoxNavigation),
    /// Extend from the fixed range anchor in Extended mode only.
    NavigateRange {
        direction: ListBoxNavigation,
        additive: bool,
    },
    Activate(K),
    ActivateActive,
    Focus(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ListBoxEvent<K> {
    SelectionPolicyChanged { policy: SelectionPolicy },
    SelectionChanged { selected: Vec<K>, active: Option<K> },
    ActiveItemChanged { active: Option<K> },
    ItemActivated { key: K },
    ProjectionChanged { visible_count: usize },
    FocusChanged { is_focused: bool },
}

/// Committed change and effects for the host to deliver. Activation can emit an
/// event without changing state. `reveal` requests host-owned scrolling.
#[derive(Clone, Debug)]
pub struct ListBoxUpdate<K> {
    pub changed: bool,
    pub events: Vec<ListBoxEvent<K>>,
    pub reveal: Option<K>,
}

/// Non-visual collection model with key-based selection. The initial projection
/// is all snapshot items in source order; filtering is deferred.
pub struct ListBoxState<T, K> {
    snapshot: ListBoxSnapshot<T, K>,
    selected: HashSet<K>,
    active: Option<K>,
    policy: SelectionPolicy,
    anchor: Option<K>,
    focused: bool,
}

impl<T, K: Clone + Eq + Hash> ListBoxState<T, K> {
    pub fn try_new(
        items: impl IntoIterator<Item = T>,
        key: impl Fn(&T) -> K,
        mode: SelectionMode,
    ) -> Result<Self, ListBoxError> {
        Ok(Self::from_snapshot(ListBoxSnapshot::try_new(items, key)?, mode))
    }

    pub fn from_snapshot(snapshot: ListBoxSnapshot<T, K>, mode: SelectionMode) -> Self {
        let mut state = Self {
            snapshot,
            selected: HashSet::new(),
            active: None,
            policy: SelectionPolicy::new(mode),
            anchor: None,
            focused: false,
        };
        state.reconcile();
        state
    }

    pub fn snapshot(&self) -> &ListBoxSnapshot<T, K> {
        &self.snapshot
    }

    /// First selected key in source order; use `selected_keys` for multiple selection.
    pub fn selected_key(&self) -> Option<&K> {
        self.selected_keys().next()
    }

    /// Selected keys in source order. Reordering alone is not a selection change.
    pub fn selected_keys(&self) -> impl Iterator<Item = &K> {
        self.snapshot.keys.iter().filter(|key| self.selected.contains(*key))
    }

    /// Capture the selected group in source order when dragging a selected item;
    /// otherwise capture only the enabled target. No state or events are changed.
    /// Unknown/disabled targets are not draggable. The host may impose further
    /// domain-specific restrictions before creating a drag session.
    pub fn drag_keys(&self, key: &K) -> Option<Vec<K>> {
        self.snapshot.is_enabled(key).then(|| {
            if self.selected.contains(key) {
                self.selected_keys().cloned().collect()
            } else {
                vec![key.clone()]
            }
        })
    }

    pub fn selection_mode(&self) -> SelectionMode {
        self.policy.mode
    }

    pub fn selection_policy(&self) -> SelectionPolicy {
        self.policy
    }

    pub fn anchor_key(&self) -> Option<&K> {
        self.anchor.as_ref()
    }

    /// Change mode while retaining the other policy settings.
    pub fn set_selection_mode(&mut self, mode: SelectionMode) -> ListBoxUpdate<K> {
        self.set_selection_policy(SelectionPolicy { mode, ..self.policy })
    }

    /// Reconcile atomically: None clears selection; single modes retain the
    /// selected active key or first selected source key. SingleRequired fills
    /// an empty selection from the active item, then the first enabled item.
    /// A changed policy clears the range anchor and preserves focus/active state.
    pub fn set_selection_policy(&mut self, policy: SelectionPolicy) -> ListBoxUpdate<K> {
        let previous = self.previous();
        if self.policy != policy {
            self.policy = policy;
            self.anchor = None;
            if policy.mode == SelectionMode::None {
                self.selected.clear();
            } else if policy.mode.is_single() && self.selected.len() > 1 {
                let retained = self
                    .active
                    .as_ref()
                    .filter(|key| self.selected.contains(*key))
                    .cloned()
                    .or_else(|| self.selected_key().cloned());
                self.selected = retained.into_iter().collect();
            }
            if policy.mode == SelectionMode::SingleRequired && self.selected.is_empty() {
                self.selected.extend(self.active.clone());
            }
            self.reconcile();
        }
        self.finish(previous)
    }

    pub fn active_key(&self) -> Option<&K> {
        self.active.as_ref()
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Position in `visible_items()` for the current snapshot. Do not retain
    /// this index across snapshot replacement; keep the stable key instead.
    pub fn visible_index(&self, key: &K) -> Option<usize> {
        self.snapshot.indices.get(key).copied()
    }

    pub fn visible_items(&self) -> impl ExactSizeIterator<Item = ListBoxVisibleItem<'_, T, K>> {
        self.snapshot.items.iter().enumerate().map(|(index, item)| {
            let key = &self.snapshot.keys[index];
            ListBoxVisibleItem {
                key: key.clone(),
                item,
                source_index: index,
                visible_index: index,
                state: ListBoxItemState {
                    selected: self.selected.contains(key),
                    active: self.active.as_ref() == Some(key),
                    enabled: self.snapshot.enabled[index],
                },
            }
        })
    }

    /// Replace only with a successfully validated snapshot. Existing keys retain
    /// selection unless disabled. Content changes always request a render.
    pub fn replace_snapshot(&mut self, snapshot: ListBoxSnapshot<T, K>) -> ListBoxUpdate<K> {
        let previous = self.previous();
        let projection_changed = self.snapshot.keys != snapshot.keys;
        self.snapshot = snapshot;
        self.reconcile();
        self.finish_replacement(previous, projection_changed)
    }

    /// Atomically replace the snapshot, selection, and active item. Validate keys
    /// against the new snapshot before changing any state; duplicate selection
    /// keys are deduplicated. `None` chooses the first selected enabled item,
    /// then the first enabled item, or remains empty if none exists.
    ///
    /// Preserves focus and emits each changed aspect once, using final-state
    /// payloads. Like `replace_snapshot`, always requests a content repaint;
    /// scrolling remains a host decision through the returned update's `reveal`.
    pub fn replace_snapshot_with_selection(
        &mut self,
        snapshot: ListBoxSnapshot<T, K>,
        keys: impl IntoIterator<Item = K>,
        active: Option<K>,
    ) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let selected: HashSet<K> = keys.into_iter().collect();
        Self::validate_selection(&snapshot, self.policy.mode, &selected)?;
        if let Some(key) = &active {
            if !snapshot.indices.contains_key(key) {
                return Err(ListBoxError::UnknownKey);
            }
            if !snapshot.is_enabled(key) {
                return Err(ListBoxError::DisabledItem);
            }
        }
        let previous = self.previous();
        let projection_changed = self.snapshot.keys != snapshot.keys;
        self.snapshot = snapshot;
        self.selected = selected;
        self.active = active;
        self.anchor = None;
        self.reconcile();
        Ok(self.finish_replacement(previous, projection_changed))
    }

    fn finish_replacement(&self, previous: Previous<K>, projection_changed: bool) -> ListBoxUpdate<K> {
        let mut update = self.finish(previous);
        update.changed = true;
        if projection_changed {
            update
                .events
                .insert(0, ListBoxEvent::ProjectionChanged { visible_count: self.snapshot.items.len() });
        }
        update
    }

    /// Replace selection with zero or one key without moving the active item.
    pub fn set_selected(&mut self, key: Option<K>) -> Result<ListBoxUpdate<K>, ListBoxError> {
        self.set_selected_keys(key)
    }

    /// Atomically replace selection without moving the active item. Duplicate
    /// keys are deduplicated; invalid keys or a mode violation leave state intact.
    /// Successful replacement clears the range anchor, even if membership matches.
    pub fn set_selected_keys(&mut self, keys: impl IntoIterator<Item = K>) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let selected: HashSet<K> = keys.into_iter().collect();
        Self::validate_selection(&self.snapshot, self.policy.mode, &selected)?;
        let previous = self.previous();
        self.selected = selected;
        self.anchor = None;
        Ok(self.finish(previous))
    }

    fn validate_selection(
        snapshot: &ListBoxSnapshot<T, K>,
        mode: SelectionMode,
        selected: &HashSet<K>,
    ) -> Result<(), ListBoxError> {
        for key in selected {
            if !snapshot.indices.contains_key(key) {
                return Err(ListBoxError::UnknownKey);
            }
            if !snapshot.is_enabled(key) {
                return Err(ListBoxError::DisabledItem);
            }
        }
        if mode == SelectionMode::None && !selected.is_empty() {
            return Err(ListBoxError::SelectionDisabled);
        }
        if !mode.allows_multiple() && selected.len() > 1 {
            return Err(ListBoxError::TooManySelected);
        }
        if selected.is_empty() && mode == SelectionMode::SingleRequired && snapshot.first_enabled().is_some() {
            return Err(ListBoxError::SelectionRequired);
        }
        Ok(())
    }

    pub fn apply(&mut self, input: ListBoxInput<K>) -> ListBoxUpdate<K> {
        let previous = self.previous();
        let mut reveal = None;
        let mut activated = None;
        match input {
            ListBoxInput::Select(key) => self.select_item(key, ListBoxSelectionModifiers::default()),
            ListBoxInput::SelectWithModifiers { key, modifiers } => self.select_item(key, modifiers),
            ListBoxInput::SelectActiveWithModifiers(modifiers) => {
                if let Some(key) = self.active.clone() {
                    self.select_item(key, modifiers);
                }
            }
            ListBoxInput::SelectActive => {
                if let Some(key) = self.active.clone() {
                    self.select_item(key, ListBoxSelectionModifiers::default());
                }
            }
            ListBoxInput::SelectAll => {
                if self.policy.mode.allows_multiple() {
                    self.anchor = None;
                    self.selected =
                        self.snapshot.keys.iter().filter(|key| self.snapshot.is_enabled(key)).cloned().collect();
                }
            }
            ListBoxInput::ClearSelection => {
                if self.policy.mode != SelectionMode::SingleRequired {
                    self.selected.clear();
                    self.anchor = None;
                }
            }
            ListBoxInput::Navigate(direction) => {
                if let Some(key) = self.navigation_target(direction) {
                    self.active = Some(key.clone());
                    if self.policy.mode == SelectionMode::Extended {
                        self.anchor = Some(key.clone());
                    }
                    if self.policy.mode.is_single() && self.policy.selection_follows_active {
                        self.selected.clear();
                        self.selected.insert(key.clone());
                    }
                    reveal = Some(key);
                }
            }
            ListBoxInput::NavigateRange { direction, additive } => {
                if self.policy.mode == SelectionMode::Extended
                    && let Some(key) = self.navigation_target(direction)
                {
                    self.select_item(key.clone(), ListBoxSelectionModifiers { toggle: additive, extend: true });
                    reveal = Some(key);
                }
            }
            ListBoxInput::Activate(key) => {
                if self.snapshot.is_enabled(&key) {
                    activated = Some(key);
                }
            }
            ListBoxInput::ActivateActive => activated = self.active.clone(),
            ListBoxInput::Focus(focused) => self.focused = focused,
        }
        let mut update = self.finish(previous);
        update.reveal = reveal;
        if let Some(key) = activated {
            update.events.push(ListBoxEvent::ItemActivated { key });
        }
        update
    }

    fn select_item(&mut self, key: K, modifiers: ListBoxSelectionModifiers) {
        if !self.snapshot.is_enabled(&key) {
            return;
        }
        match self.policy.mode {
            SelectionMode::None => {}
            SelectionMode::Extended if modifiers.extend => {
                let anchor = self.anchor.clone().or_else(|| self.active.clone()).unwrap_or_else(|| key.clone());
                // Both keys are enabled snapshot members after reconciliation.
                if let (Some(start), Some(end)) = (self.visible_index(&anchor), self.visible_index(&key)) {
                    if !modifiers.toggle {
                        self.selected.clear();
                    }
                    for index in start.min(end)..=start.max(end) {
                        if self.snapshot.enabled[index] {
                            self.selected.insert(self.snapshot.keys[index].clone());
                        }
                    }
                    self.anchor = Some(anchor);
                }
            }
            SelectionMode::Multiple | SelectionMode::Extended
                if self.policy.mode == SelectionMode::Multiple || modifiers.toggle =>
            {
                if !self.selected.remove(&key) {
                    self.selected.insert(key.clone());
                }
                if self.policy.mode == SelectionMode::Extended {
                    self.anchor = Some(key.clone());
                }
            }
            _ => {
                let toggle_off = self.policy.mode == SelectionMode::SingleAllowNone
                    && self.policy.toggle_off
                    && self.selected.contains(&key);
                self.selected.clear();
                if !toggle_off {
                    self.selected.insert(key.clone());
                }
                if self.policy.mode == SelectionMode::Extended {
                    self.anchor = Some(key.clone());
                }
            }
        }
        self.active = Some(key);
    }

    fn reconcile(&mut self) {
        if self.anchor.as_ref().is_some_and(|key| !self.snapshot.is_enabled(key)) {
            self.anchor = None;
        }
        self.selected.retain(|key| self.snapshot.is_enabled(key));
        if self.selected.is_empty() && self.policy.mode == SelectionMode::SingleRequired {
            self.selected.extend(self.snapshot.first_enabled());
        }
        if self.active.as_ref().is_none_or(|key| !self.snapshot.is_enabled(key)) {
            self.active = self.selected_key().cloned().or_else(|| self.snapshot.first_enabled());
        }
    }

    fn navigation_target(&self, direction: ListBoxNavigation) -> Option<K> {
        let current = self.active.as_ref().and_then(|key| self.snapshot.indices.get(key)).copied();
        let mut eligible = self.snapshot.keys.iter().enumerate().filter(|(i, _)| self.snapshot.enabled[*i]);
        let target = match direction {
            ListBoxNavigation::First => eligible.next(),
            ListBoxNavigation::Last => eligible.next_back(),
            ListBoxNavigation::Next => eligible.find(|(i, _)| current.is_none_or(|current| *i > current)),
            ListBoxNavigation::Previous => eligible.rfind(|(i, _)| current.is_none_or(|current| *i < current)),
        };
        target.map(|(_, key)| key.clone())
    }

    fn previous(&self) -> Previous<K> {
        Previous {
            selected: self.selected.clone(),
            active: self.active.clone(),
            focused: self.focused,
            anchor: self.anchor.clone(),
            policy: self.policy,
        }
    }

    fn finish(&self, previous: Previous<K>) -> ListBoxUpdate<K> {
        let mut events = Vec::new();
        if previous.policy != self.policy {
            events.push(ListBoxEvent::SelectionPolicyChanged { policy: self.policy });
        }
        if previous.selected != self.selected {
            events.push(ListBoxEvent::SelectionChanged {
                selected: self.selected_keys().cloned().collect(),
                active: self.active.clone(),
            });
        }
        if previous.active != self.active {
            events.push(ListBoxEvent::ActiveItemChanged { active: self.active.clone() });
        }
        if previous.focused != self.focused {
            events.push(ListBoxEvent::FocusChanged { is_focused: self.focused });
        }
        ListBoxUpdate { changed: !events.is_empty() || previous.anchor != self.anchor, events, reveal: None }
    }
}

struct Previous<K> {
    selected: HashSet<K>,
    active: Option<K>,
    focused: bool,
    anchor: Option<K>,
    policy: SelectionPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ListBoxState<u32, u32> {
        ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::SingleAllowNone).unwrap()
    }

    fn multiple() -> ListBoxState<u32, u32> {
        let snapshot = ListBoxSnapshot::try_with_enabled([1, 2, 3, 4], |item| *item, |item| *item != 2).unwrap();
        ListBoxState::from_snapshot(snapshot, SelectionMode::Multiple)
    }

    fn selected(list: &ListBoxState<u32, u32>) -> Vec<u32> {
        list.selected_keys().copied().collect()
    }

    #[test]
    fn multiple_clicks_toggle_independently_and_emit_source_order() {
        let mut list = multiple();
        list.apply(ListBoxInput::Select(4));
        let update = list.apply(ListBoxInput::Select(1));
        assert_eq!(selected(&list), vec![1, 4]);
        assert_eq!(
            update.events,
            vec![
                ListBoxEvent::SelectionChanged { selected: vec![1, 4], active: Some(1) },
                ListBoxEvent::ActiveItemChanged { active: Some(1) },
            ]
        );
        list.apply(ListBoxInput::Select(1));
        assert_eq!(selected(&list), vec![4]);
        assert_eq!(list.active_key(), Some(&1));
        assert!(!list.apply(ListBoxInput::Select(2)).changed);
        assert!(!list.apply(ListBoxInput::Select(99)).changed);
        let flags: Vec<_> = list.visible_items().map(|item| item.state.selected).collect();
        assert_eq!(flags, vec![false, false, false, true]);
    }

    #[test]
    fn multiple_navigation_preserves_selection_and_space_toggles_active() {
        let mut list = multiple();
        list.apply(ListBoxInput::Select(1));
        let update = list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
        assert_eq!(update.reveal, Some(3));
        assert_eq!(selected(&list), vec![1]);
        list.apply(ListBoxInput::SelectActive);
        assert_eq!(selected(&list), vec![1, 3]);
        list.apply(ListBoxInput::SelectActive);
        assert_eq!(selected(&list), vec![1]);
        let update = list.apply(ListBoxInput::ActivateActive);
        assert_eq!(update.events, vec![ListBoxEvent::ItemActivated { key: 3 }]);
        assert_eq!(selected(&list), vec![1]);
    }

    #[test]
    fn select_all_skips_disabled_and_clear_preserves_active() {
        let mut list = multiple();
        list.apply(ListBoxInput::SelectAll);
        assert_eq!(selected(&list), vec![1, 3, 4]);
        assert!(!list.apply(ListBoxInput::SelectAll).changed);
        list.apply(ListBoxInput::ClearSelection);
        assert!(selected(&list).is_empty());
        assert_eq!(list.active_key(), Some(&1));
        assert!(!list.apply(ListBoxInput::ClearSelection).changed);
        list.replace_snapshot(ListBoxSnapshot::try_with_enabled([1, 2], |item| *item, |_| false).unwrap());
        assert!(!list.apply(ListBoxInput::SelectAll).changed);
        assert!(!list.apply(ListBoxInput::SelectActive).changed);
    }

    #[test]
    fn multiple_replacement_reconciles_membership_without_reorder_selection_events() {
        let mut list = multiple();
        list.set_selected_keys([4, 1, 3]).unwrap();
        let update = list.replace_snapshot(ListBoxSnapshot::try_new([4, 3, 1, 2], |item| *item).unwrap());
        assert_eq!(selected(&list), vec![4, 3, 1]);
        assert_eq!(update.events, vec![ListBoxEvent::ProjectionChanged { visible_count: 4 }]);
        let update =
            list.replace_snapshot(ListBoxSnapshot::try_with_enabled([4, 3], |item| *item, |item| *item != 3).unwrap());
        assert_eq!(selected(&list), vec![4]);
        assert_eq!(list.active_key(), Some(&4));
        assert_eq!(
            update.events,
            vec![
                ListBoxEvent::ProjectionChanged { visible_count: 2 },
                ListBoxEvent::SelectionChanged { selected: vec![4], active: Some(4) },
                ListBoxEvent::ActiveItemChanged { active: Some(4) },
            ]
        );
    }

    #[test]
    fn programmatic_multiple_selection_is_atomic_and_deduplicated() {
        let mut list = multiple();
        list.set_selected_keys([3, 1, 3]).unwrap();
        assert_eq!(selected(&list), vec![1, 3]);
        assert!(!list.set_selected_keys([1, 3]).unwrap().changed);
        assert_eq!(list.set_selected_keys([1, 99]).unwrap_err(), ListBoxError::UnknownKey);
        assert_eq!(list.set_selected_keys([4, 2]).unwrap_err(), ListBoxError::DisabledItem);
        assert_eq!(selected(&list), vec![1, 3]);
        assert_eq!(list.active_key(), Some(&1));
    }

    #[test]
    fn combined_replacement_emits_only_final_changes_in_contract_order() {
        let mut list = multiple();
        list.apply(ListBoxInput::Select(4));
        list.apply(ListBoxInput::Focus(true));
        let update = list
            .replace_snapshot_with_selection(
                ListBoxSnapshot::try_new([7, 8, 9], |item| *item).unwrap(),
                [9, 7, 9],
                Some(9),
            )
            .unwrap();
        assert_eq!(
            update.events,
            vec![
                ListBoxEvent::ProjectionChanged { visible_count: 3 },
                ListBoxEvent::SelectionChanged { selected: vec![7, 9], active: Some(9) },
                ListBoxEvent::ActiveItemChanged { active: Some(9) },
            ]
        );
        assert!(update.changed);
        assert!(list.is_focused());
        assert_eq!(update.reveal, None);
        let update = list
            .replace_snapshot_with_selection(
                ListBoxSnapshot::try_new([9, 8, 7], |item| *item).unwrap(),
                [7, 9],
                Some(9),
            )
            .unwrap();
        assert_eq!(update.events, vec![ListBoxEvent::ProjectionChanged { visible_count: 3 }]);
        let update = list
            .replace_snapshot_with_selection(
                ListBoxSnapshot::try_new([9, 8, 7], |item| *item).unwrap(),
                [9, 7],
                Some(9),
            )
            .unwrap();
        assert!(update.changed); // Snapshot content can change even with identical keys.
        assert!(update.events.is_empty());
    }

    #[test]
    fn combined_replacement_validates_selection_and_active_before_commit() {
        let mut list = multiple();
        list.apply(ListBoxInput::Select(4));
        list.apply(ListBoxInput::Focus(true));
        for (keys, active, error) in [
            (vec![7, 99], Some(7), ListBoxError::UnknownKey),
            (vec![8], Some(7), ListBoxError::DisabledItem),
            (vec![7], Some(99), ListBoxError::UnknownKey),
            (vec![7], Some(8), ListBoxError::DisabledItem),
        ] {
            let snapshot = ListBoxSnapshot::try_with_enabled([7, 8], |item| *item, |item| *item != 8).unwrap();
            assert_eq!(list.replace_snapshot_with_selection(snapshot, keys, active).unwrap_err(), error);
            assert_eq!(list.snapshot().items(), &[1, 2, 3, 4]);
            assert_eq!(selected(&list), vec![4]);
            assert_eq!(list.active_key(), Some(&4));
            assert!(list.is_focused());
        }
        for mode in [SelectionMode::SingleAllowNone, SelectionMode::SingleRequired] {
            let mut list = ListBoxState::try_new([1], |item| *item, mode).unwrap();
            let snapshot = ListBoxSnapshot::try_new([7, 8], |item| *item).unwrap();
            assert_eq!(
                list.replace_snapshot_with_selection(snapshot, [7, 8], Some(7)).unwrap_err(),
                ListBoxError::TooManySelected
            );
            assert_eq!(list.snapshot().items(), &[1]);
        }
    }

    #[test]
    fn combined_replacement_respects_required_selection_and_active_fallback() {
        let mut list = ListBoxState::try_new([1], |item| *item, SelectionMode::SingleRequired).unwrap();
        let snapshot = ListBoxSnapshot::try_new([7, 8], |item| *item).unwrap();
        assert_eq!(
            list.replace_snapshot_with_selection(snapshot, [], None).unwrap_err(),
            ListBoxError::SelectionRequired
        );
        assert_eq!(list.snapshot().items(), &[1]);
        let snapshot = ListBoxSnapshot::try_new([7, 8], |item| *item).unwrap();
        list.replace_snapshot_with_selection(snapshot, [8, 8], None).unwrap();
        assert_eq!(list.active_key(), Some(&8));
        let snapshot = ListBoxSnapshot::try_with_enabled([7, 8], |item| *item, |_| false).unwrap();
        list.replace_snapshot_with_selection(snapshot, [], None).unwrap();
        assert!(selected(&list).is_empty());
        assert_eq!(list.active_key(), None);
        list.replace_snapshot_with_selection(ListBoxSnapshot::try_new([], |item: &u32| *item).unwrap(), [], None)
            .unwrap();
        assert_eq!(list.active_key(), None);
    }

    #[test]
    fn single_modes_reject_multiple_keys_and_keep_required_selection() {
        for mode in [SelectionMode::SingleAllowNone, SelectionMode::SingleRequired] {
            let mut list = ListBoxState::try_new([1, 2, 3], |item| *item, mode).unwrap();
            list.set_selected(Some(2)).unwrap();
            assert_eq!(list.set_selected_keys([1, 3]).unwrap_err(), ListBoxError::TooManySelected);
            assert!(!list.apply(ListBoxInput::SelectAll).changed);
            assert_eq!(selected(&list), vec![2]);
            list.apply(ListBoxInput::ClearSelection);
            assert_eq!(
                selected(&list),
                if mode == SelectionMode::SingleRequired {
                    vec![2]
                } else {
                    vec![]
                }
            );
        }
    }

    #[test]
    fn duplicate_keys_fail_before_replacing_state() {
        let mut list = state();
        list.apply(ListBoxInput::Select(2));
        let replacement = ListBoxSnapshot::try_new([2, 2], |item| *item);
        assert!(matches!(replacement, Err(ListBoxError::DuplicateKey)));
        assert_eq!(list.selected_key(), Some(&2));
        assert_eq!(list.snapshot().items(), &[1, 2, 3]);
    }

    #[test]
    fn click_commits_once_and_activation_is_separate() {
        let mut list = state();
        let update = list.apply(ListBoxInput::Select(2));
        assert_eq!(
            update.events,
            vec![
                ListBoxEvent::SelectionChanged { selected: vec![2], active: Some(2) },
                ListBoxEvent::ActiveItemChanged { active: Some(2) },
            ]
        );
        assert!(!list.apply(ListBoxInput::Select(2)).changed);
        let update = list.apply(ListBoxInput::ActivateActive);
        assert!(!update.changed);
        assert_eq!(update.events, vec![ListBoxEvent::ItemActivated { key: 2 }]);
    }

    #[test]
    fn replacement_preserves_keys_and_reconciles_removal() {
        let mut list = state();
        list.apply(ListBoxInput::Select(2));
        let update = list.replace_snapshot(ListBoxSnapshot::try_new([3, 2, 1], |item| *item).unwrap());
        assert_eq!(list.selected_key(), Some(&2));
        assert_eq!(update.events, vec![ListBoxEvent::ProjectionChanged { visible_count: 3 }]);
        let update = list.replace_snapshot(ListBoxSnapshot::try_new([3, 1], |item| *item).unwrap());
        assert_eq!(list.selected_key(), None);
        assert_eq!(list.active_key(), Some(&3));
        assert_eq!(update.events.len(), 3);
    }

    #[test]
    fn required_selection_handles_disabled_and_empty_snapshots() {
        let snapshot = ListBoxSnapshot::try_with_enabled([1, 2], |item| *item, |item| *item == 2).unwrap();
        let mut list = ListBoxState::from_snapshot(snapshot, SelectionMode::SingleRequired);
        assert_eq!(list.selected_key(), Some(&2));
        assert_eq!(list.set_selected(None).unwrap_err(), ListBoxError::SelectionRequired);
        list.replace_snapshot(ListBoxSnapshot::try_with_enabled([1, 2], |item| *item, |_| false).unwrap());
        assert_eq!(list.selected_key(), None);
        assert_eq!(list.active_key(), None);
        assert!(!list.apply(ListBoxInput::Select(1)).changed);
        list.replace_snapshot(ListBoxSnapshot::try_new([], |item: &u32| *item).unwrap());
        assert_eq!(list.visible_items().len(), 0);
    }

    #[test]
    fn keyboard_skips_disabled_without_wrapping_or_selecting() {
        let snapshot = ListBoxSnapshot::try_with_enabled([1, 2, 3], |item| *item, |item| *item != 2).unwrap();
        let mut list = ListBoxState::from_snapshot(snapshot, SelectionMode::SingleAllowNone);
        let update = list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
        assert_eq!(update.reveal, Some(3));
        assert_eq!(list.active_key(), Some(&3));
        assert_eq!(list.selected_key(), None);
        assert!(!list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next)).changed);
        list.apply(ListBoxInput::SelectActive);
        assert_eq!(list.selected_key(), Some(&3));
        list.apply(ListBoxInput::Navigate(ListBoxNavigation::First));
        assert_eq!(list.active_key(), Some(&1));
    }

    #[test]
    fn invalid_programmatic_selection_is_atomic_and_blur_preserves_state() {
        let mut list = state();
        list.set_selected(Some(2)).unwrap();
        assert_eq!(list.set_selected(Some(9)).unwrap_err(), ListBoxError::UnknownKey);
        list.apply(ListBoxInput::Focus(true));
        list.apply(ListBoxInput::Focus(false));
        assert_eq!(list.selected_key(), Some(&2));
        assert_eq!(list.active_key(), Some(&1));
        assert!(!list.apply(ListBoxInput::Select(9)).changed);
    }
}
