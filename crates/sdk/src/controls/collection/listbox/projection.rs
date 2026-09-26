//! Ordered views over an unchanged source snapshot. Query/sort logic stays with the host.
use super::*;
use std::sync::Arc;

pub(super) struct Projection<K> {
    pub source_indices: Vec<usize>,
    pub indices: HashMap<K, usize>,
    pub identity: Arc<()>,
}

impl<K: Clone + Eq + Hash> Projection<K> {
    pub fn all<T>(snapshot: &ListBoxSnapshot<T, K>) -> Self {
        Self {
            source_indices: (0..snapshot.items.len()).collect(),
            indices: snapshot.indices.clone(),
            identity: Arc::new(()),
        }
    }

    fn try_new<T>(snapshot: &ListBoxSnapshot<T, K>, keys: impl IntoIterator<Item = K>) -> Result<Self, ListBoxError> {
        let mut source_indices = Vec::new();
        let mut indices = HashMap::new();
        for key in keys {
            let source = snapshot.indices.get(&key).copied().ok_or(ListBoxError::UnknownKey)?;
            if indices.insert(key, source_indices.len()).is_some() {
                return Err(ListBoxError::DuplicateKey);
            }
            source_indices.push(source);
        }
        Ok(Self { source_indices, indices, identity: Arc::new(()) })
    }

    fn for_snapshot<T>(snapshot: &ListBoxSnapshot<T, K>, keys: Option<Vec<K>>) -> Result<Self, ListBoxError> {
        keys.map_or_else(|| Ok(Self::all(snapshot)), |keys| Self::try_new(snapshot, keys))
    }
}

impl<T, K: Clone + Eq + Hash> ListBoxState<T, K> {
    /// Replace the displayed order/subset without replacing source data. Unknown
    /// or duplicate keys fail atomically. Hidden selections survive; active item
    /// and range anchor must remain enabled and visible. Identical views are no-ops.
    pub fn set_projection(&mut self, keys: impl IntoIterator<Item = K>) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let projection = Projection::try_new(&self.snapshot, keys)?;
        Ok(self.commit_projection(projection))
    }

    /// Display every source item in source order, retaining keyed selection.
    pub fn reset_projection(&mut self) -> ListBoxUpdate<K> {
        self.commit_projection(Projection::all(&self.snapshot))
    }

    fn commit_projection(&mut self, projection: Projection<K>) -> ListBoxUpdate<K> {
        let previous = self.previous();
        if self.projection.source_indices == projection.source_indices {
            return self.finish(previous);
        }
        self.projection = projection;
        self.reconcile();
        self.finish_replacement(previous, true)
    }

    /// Atomically replace source data and its ordered view. `None` displays all
    /// source items; `Some(vec![])` displays none. Selection is reconciled against
    /// source eligibility, active/anchor against the new view. Does not scroll.
    pub fn replace_snapshot_with_projection(
        &mut self,
        snapshot: ListBoxSnapshot<T, K>,
        keys: Option<Vec<K>>,
    ) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let projection = Projection::for_snapshot(&snapshot, keys)?;
        let previous = self.previous();
        let changed = !self.projected_keys().eq(projection.source_indices.iter().map(|&i| &snapshot.keys[i]));
        self.snapshot = snapshot;
        self.projection = projection;
        self.reconcile();
        Ok(self.finish_replacement(previous, changed))
    }

    /// Combined transaction for hosts committing collection changes (for example,
    /// a drop) while retaining a filter/sort. Explicit active keys must be enabled
    /// and visible; `None` reconciles to a visible candidate. Selected keys may
    /// be hidden. All validation completes before any state changes.
    pub fn replace_snapshot_with_projection_and_selection(
        &mut self,
        snapshot: ListBoxSnapshot<T, K>,
        projection: Option<Vec<K>>,
        keys: impl IntoIterator<Item = K>,
        active: Option<K>,
    ) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let projection = Projection::for_snapshot(&snapshot, projection)?;
        let selected: HashSet<K> = keys.into_iter().collect();
        Self::validate_selection(&snapshot, self.policy.mode, &selected)?;
        if let Some(key) = &active {
            if !snapshot.indices.contains_key(key) {
                return Err(ListBoxError::UnknownKey);
            }
            if !snapshot.is_enabled(key) {
                return Err(ListBoxError::DisabledItem);
            }
            if !projection.indices.contains_key(key) {
                return Err(ListBoxError::HiddenItem);
            }
        }
        let previous = self.previous();
        let changed = !self.projected_keys().eq(projection.source_indices.iter().map(|&i| &snapshot.keys[i]));
        self.snapshot = snapshot;
        self.projection = projection;
        self.selected = selected;
        self.active = active;
        self.anchor = None;
        self.reconcile();
        Ok(self.finish_replacement(previous, changed))
    }

    pub(in super::super) fn projection_identity(&self) -> &Arc<()> {
        &self.projection.identity
    }

    pub(super) fn projected_keys(&self) -> impl DoubleEndedIterator<Item = &K> + ExactSizeIterator {
        self.projection.source_indices.iter().map(|&i| &self.snapshot.keys[i])
    }

    pub(super) fn is_visible_enabled(&self, key: &K) -> bool {
        self.projection.indices.contains_key(key) && self.snapshot.is_enabled(key)
    }
}
