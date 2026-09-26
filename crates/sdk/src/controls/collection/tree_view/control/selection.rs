//! Selection transactions are keyed; ranges use effective displayed preorder.
use super::*;
use super::super::TreeViewSelectionPolicy;
use gpui::{KeyDownEvent, Modifiers};

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    pub fn selection_policy(&self) -> TreeViewSelectionPolicy {
        self.model.selection_policy
    }

    /// Change policy without focusing or expanding nodes. Single retains a
    /// selected active node, then the first displayed selection, then a stable
    /// hidden selection. None clears selection. Flags alone do not select.
    pub fn set_selection_policy(&mut self, policy: TreeViewSelectionPolicy, cx: &mut Context<Self>) {
        if self.model.selection_policy == policy {
            return;
        }
        let before = self.selected_ids.clone();
        if self.model.selection_policy.mode != policy.mode {
            self.selection_anchor = if policy.mode == TreeViewSelectionMode::Extended {
                self.active_node_id.clone()
            } else {
                None
            };
        }
        self.model.selection_policy = policy;
        match policy.mode {
            TreeViewSelectionMode::None => self.selected_ids.clear(),
            TreeViewSelectionMode::Single => {
                let retained = self
                    .active_node_id
                    .as_ref()
                    .filter(|id| self.selected_ids.contains(*id))
                    .cloned()
                    .or_else(|| self.visible_ids().into_iter().find(|id| self.selected_ids.contains(id)))
                    .or_else(|| self.selected_ids.iter().min().cloned());
                self.selected_ids = retained.into_iter().collect();
            }
            TreeViewSelectionMode::Multiple | TreeViewSelectionMode::Extended => {}
        }
        self.emit_selection_delta(&before, cx);
        cx.emit(TreeViewEvent::SelectionPolicyChanged { policy });
        cx.notify();
    }

    /// Visible, enabled stable ID anchoring the next Extended-mode range.
    pub fn range_anchor_id(&self) -> Option<&SharedString> {
        self.selection_anchor.as_ref()
    }

    /// Add visible enabled nodes in Multiple/Extended, retaining hidden selections.
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        if !matches!(
            self.model.selection_policy.mode,
            TreeViewSelectionMode::Multiple | TreeViewSelectionMode::Extended
        ) {
            return;
        }
        let before = self.selected_ids.clone();
        self.selected_ids.extend(self.flat_cache.iter().filter(|row| row.enabled).map(|row| row.id.clone()));
        self.selection_anchor = None;
        self.emit_selection_delta(&before, cx);
        cx.notify();
    }

    /// Clear every selected node, including filtered/collapsed selections.
    /// Active identity and focus are unchanged; repeated clears emit no event.
    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        let before = std::mem::take(&mut self.selected_ids);
        self.selection_anchor = None;
        self.emit_selection_delta(&before, cx);
        cx.notify();
    }

    fn emit_selection_delta(&self, before: &HashSet<SharedString>, cx: &mut Context<Self>) {
        if *before != self.selected_ids {
            cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        }
    }

    pub(super) fn prepare_selection_anchor(&mut self) {
        if self.model.selection_policy.mode == TreeViewSelectionMode::Extended && self.selection_anchor.is_none() {
            self.selection_anchor = self.active_node_id.clone();
        }
    }

    pub(super) fn select_with_modifiers(&mut self, index: usize, modifiers: Modifiers, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.flat_cache.get(index).is_some_and(|row| row.enabled) {
            return;
        }
        let id = self.flat_cache[index].id.clone();
        let before = self.selected_ids.clone();
        let policy = self.model.selection_policy;
        let toggle = modifiers.control || modifiers.platform;
        match policy.mode {
            TreeViewSelectionMode::None => {}
            TreeViewSelectionMode::Extended if modifiers.shift => {
                let anchor =
                    self.selection_anchor.clone().or_else(|| self.active_node_id.clone()).unwrap_or_else(|| id.clone());
                let start = self.flat_index_for_id(&anchor).unwrap_or(index);
                if !toggle {
                    self.selected_ids.clear();
                }
                self.selected_ids.extend(
                    self.flat_cache[start.min(index)..=start.max(index)]
                        .iter()
                        .filter(|row| row.enabled)
                        .map(|row| row.id.clone()),
                );
                self.selection_anchor = Some(anchor);
            }
            TreeViewSelectionMode::Multiple | TreeViewSelectionMode::Extended
                if policy.mode == TreeViewSelectionMode::Multiple
                    || toggle
                    || (policy.toggle_off && self.selected_ids.contains(&id)) =>
            {
                if !self.selected_ids.remove(&id) {
                    self.selected_ids.insert(id.clone());
                }
                if policy.mode == TreeViewSelectionMode::Extended {
                    self.selection_anchor = Some(id.clone());
                }
            }
            _ => {
                let deselect = policy.mode == TreeViewSelectionMode::Single
                    && policy.toggle_off
                    && self.selected_ids.contains(&id);
                self.selected_ids.clear();
                if !deselect {
                    self.selected_ids.insert(id.clone());
                }
                if policy.mode == TreeViewSelectionMode::Extended {
                    self.selection_anchor = Some(id.clone());
                }
            }
        }
        self.set_active_index(Some(index), false, cx);
        self.emit_selection_delta(&before, cx);
        cx.notify();
    }

    pub(super) fn navigate_selection(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        let Some(index) = index.filter(|index| self.flat_cache.get(*index).is_some_and(|row| row.enabled)) else {
            return;
        };
        let before = self.selected_ids.clone();
        self.set_active_index(Some(index), true, cx);
        match self.model.selection_policy.mode {
            TreeViewSelectionMode::Extended => self.selection_anchor = self.active_node_id.clone(),
            TreeViewSelectionMode::Single if self.model.selection_policy.selection_follows_active => {
                self.selected_ids = self.active_node_id.clone().into_iter().collect();
            }
            _ => {}
        }
        self.emit_selection_delta(&before, cx);
        cx.notify();
    }

    pub(super) fn activate_node(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.model.enabled
            && let Some(node) = self.flat_cache.get(index).filter(|node| node.enabled)
        {
            cx.emit(TreeViewEvent::NodeActivated { node_id: node.id.clone(), data: node.data.clone() });
        }
    }

    pub(super) fn handle_select_all(&mut self, _: &SelectAllNodes, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.focus_handle.is_focused(window) {
            return;
        }
        self.select_all(cx);
    }
    pub(super) fn handle_clear_selection(
        &mut self,
        _: &ClearSelectedNodes,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || !self.focus_handle.is_focused(window) {
            return;
        }
        self.clear_selection(cx);
    }

    pub(super) fn handle_selection_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_position();
        if self.model.selection_policy.mode != TreeViewSelectionMode::Extended
            || !self.model.enabled
            || !self.focus_handle.is_focused(window)
        {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        if modifiers.alt || modifiers.function {
            return;
        }
        let current = self.active_node_id.as_ref().and_then(|id| self.flat_index_for_id(id));
        let key = event.keystroke.key.as_str();
        if key.eq_ignore_ascii_case("a") && (modifiers.platform || modifiers.control) {
            if modifiers.shift {
                self.clear_selection(cx);
            } else {
                self.select_all(cx);
            }
        } else if matches!(key, "up" | "down" | "home" | "end") {
            // Ranges never wrap around the ends of displayed preorder.
            let target = match key {
                "home" => first_enabled_flat_index(&self.flat_cache),
                "end" => last_enabled_flat_index(&self.flat_cache),
                "up" if modifiers.platform => first_enabled_flat_index(&self.flat_cache),
                "down" if modifiers.platform => last_enabled_flat_index(&self.flat_cache),
                "up" | "down" if !modifiers.shift && self.model.wrap_navigation => next_enabled_flat_index(
                    &self.flat_cache,
                    current,
                    if key == "up" {
                        TreeDirection::Previous
                    } else {
                        TreeDirection::Next
                    },
                ),
                "up" => current.map_or_else(
                    || last_enabled_flat_index(&self.flat_cache),
                    |index| (0..index).rev().find(|&i| self.flat_cache[i].enabled).or(Some(index)),
                ),
                _ => current.map_or_else(
                    || first_enabled_flat_index(&self.flat_cache),
                    |index| ((index + 1)..self.flat_cache.len()).find(|&i| self.flat_cache[i].enabled).or(Some(index)),
                ),
            };
            if let Some(index) = target {
                if modifiers.shift {
                    self.select_with_modifiers(index, modifiers, cx);
                    self.set_active_index(Some(index), true, cx);
                } else {
                    self.navigate_selection(Some(index), cx);
                }
            }
        } else if key == "space" {
            if let Some(index) = current {
                self.select_with_modifiers(index, modifiers, cx);
            }
        } else if key == "enter" && !modifiers.modified() {
            if let Some(index) = current {
                self.activate_node(index, cx);
            }
        } else {
            return;
        }
        cx.stop_propagation();
    }
}
