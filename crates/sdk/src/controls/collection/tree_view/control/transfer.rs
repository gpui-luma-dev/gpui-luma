use super::*;

/// Selection, expansion, active identity and range anchor from one loaded subtree.
/// Hosts transfer this only after preparing and applying a cross-tree move.
#[derive(Clone, Debug)]
pub struct TreeViewSubtreeState {
    nodes: HashSet<SharedString>,
    selected: HashSet<SharedString>,
    expanded: HashSet<SharedString>,
    active: Option<SharedString>,
    anchor: Option<SharedString>,
}

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    /// Capture loaded descendants by stable ID; a missing root yields empty state.
    pub fn capture_subtree_state(&self, root: &SharedString) -> TreeViewSubtreeState {
        self.capture_subtrees_state(std::slice::from_ref(root))
    }

    /// Capture a group of loaded subtrees as one state transaction. Overlapping
    /// roots are deduplicated; active identity and range anchor are captured once.
    pub fn capture_subtrees_state(&self, roots: &[SharedString]) -> TreeViewSubtreeState {
        let roots = roots.iter().collect();
        let nodes: HashSet<_> = self
            .model
            .index
            .nodes
            .keys()
            .filter(|id| self.model.index.is_in_subtrees(id, &roots))
            .cloned()
            .collect();
        TreeViewSubtreeState {
            selected: self.selected_ids.intersection(&nodes).cloned().collect(),
            expanded: self.expanded_ids.intersection(&nodes).cloned().collect(),
            active: self.active_node_id.clone().filter(|id| nodes.contains(id)),
            anchor: self.selection_anchor.clone().filter(|id| nodes.contains(id)),
            nodes,
        }
    }

    /// Restore eligible transferred state without focus or expansion events.
    /// Multiple/Extended retain unrelated selections; Single gives a transferred
    /// selection priority. Hidden selections remain eligible. Unrelated expansion
    /// and the destination viewport are preserved.
    pub fn restore_subtree_state(&mut self, state: &TreeViewSubtreeState, cx: &mut Context<Self>) {
        self.cancel_position();
        let selected_before = self.selected_ids.clone();
        let active_before = self.active_node_id.clone();
        for id in &state.nodes {
            if self.model.index.nodes.get(id).is_some_and(|node| node.branch) {
                if state.expanded.contains(id) {
                    self.expanded_ids.insert(id.clone());
                } else {
                    self.expanded_ids.remove(id);
                }
            }
        }
        let mut transferred: Vec<_> = state
            .selected
            .iter()
            .filter(|id| self.model.index.nodes.get(*id).is_some_and(|n| n.enabled))
            .cloned()
            .collect();
        transferred.sort();
        match self.model.selection_policy.mode {
            TreeViewSelectionMode::None => {}
            TreeViewSelectionMode::Single => {
                if let Some(id) = transferred.first() {
                    self.selected_ids = HashSet::from([id.clone()]);
                }
            }
            TreeViewSelectionMode::Multiple | TreeViewSelectionMode::Extended => self.selected_ids.extend(transferred),
        }
        self.expand_transitions.clear();
        self.sync_expand_transitions_for_tree();
        let visible = self.visible_ids();
        let active = state
            .active
            .as_ref()
            .filter(|id| self.model.index.nodes.contains_key(*id))
            .or(self.active_node_id.as_ref());
        self.active_node_id = self.model.index.reconcile_active(&self.model.index, active, &visible, &visible);
        if self.model.selection_policy.mode == TreeViewSelectionMode::Extended {
            let anchor = state.anchor.as_ref().or(self.selection_anchor.as_ref());
            self.selection_anchor = self.model.index.reconcile_active(&self.model.index, anchor, &visible, &visible);
        }
        self.rebuild_flat_cache(cx);
        if selected_before != self.selected_ids {
            cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        }
        if active_before != self.active_node_id {
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: self.active_node_id.clone() });
        }
        cx.notify();
    }
}
