use std::collections::{HashMap, HashSet};

use gpui::{
    App, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, FocusHandle, IntoElement, ListAlignment,
    ListState, Render, ScrollWheelEvent, SharedString, Subscription, Window, div, list, prelude::*, px,
};

use super::{
    FlatTreeNode, TreeNode, TreeViewBuilder, TreeViewModel, TreeViewRenderModel, TreeViewSelectionMode,
    TreeViewTemplateHandlers,
};
use crate::animation::DisclosureMotion;
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::key_handling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use crate::theme::observe_theme_revision;

const EXPAND_VISIBLE_EPSILON: f32 = 0.001;

gpui::actions!(tree_view, [ExpandNode, CollapseNode]);

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TreeViewEvent<T>
where
    T: Clone,
{
    NodeExpanded { node_id: SharedString, data: T },
    NodeCollapsed { node_id: SharedString, data: T },
    SelectionChanged { selected_ids: HashSet<SharedString> },
    ActiveNodeChanged { node_id: Option<SharedString> },
    ScrollChanged { top_index: usize },
    FocusChanged { focused: bool },
    RowHoverChanged { node_id: SharedString, index: usize, hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeDirection {
    Previous,
    Next,
}

pub(crate) struct FlatNodeWrapper<T> {
    id: SharedString,
    label: SharedString,
    icon: Option<lucide_svg_static::Icon>,
    depth: usize,
    has_children: bool,
    enabled: bool,
    expand_progress: f32,
    row_height_factor: f32,
    data: T,
}

pub struct TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    model: TreeViewModel<T>,
    focus_handle: FocusHandle,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    list_state: ListState,
    expanded_ids: HashSet<SharedString>,
    expand_transitions: HashMap<SharedString, DisclosureMotion>,
    selected_ids: HashSet<SharedString>,
    active_node_id: Option<SharedString>,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    flat_cache: Vec<FlatNodeWrapper<T>>,
    emitted_scroll_top_index: usize,
    emitted_focused: bool,
}

impl<T> EventEmitter<TreeViewEvent<T>> for TreeViewControl<T> where T: Clone + Send + Sync + 'static {}

impl<T> TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) fn from_builder(builder: TreeViewBuilder<T>, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();

        let mut control = Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            focus_in_subscription: None,
            focus_out_subscription: None,
            list_state: ListState::new(0, ListAlignment::Top, px(480.0)),
            expanded_ids: HashSet::new(),
            expand_transitions: HashMap::new(),
            selected_ids: HashSet::new(),
            active_node_id: None,
            hovered_index: None,
            pressed_index: None,
            flat_cache: Vec::new(),
            emitted_scroll_top_index: 0,
            emitted_focused: false,
        };

        control.populate_initial_expands();
        control.sync_expand_transitions_for_tree();
        control.rebuild_flat_cache(None, cx);

        control
    }

    pub fn items(&self) -> &[TreeNode<T>] {
        &self.model.items
    }

    pub fn selected_ids(&self) -> &HashSet<SharedString> {
        &self.selected_ids
    }

    pub fn active_node_id(&self) -> Option<&SharedString> {
        self.active_node_id.as_ref()
    }

    pub fn is_expanded(&self, node_id: &SharedString) -> bool {
        self.expanded_ids.contains(node_id)
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = TreeNode<T>>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.selected_ids.clear();
        self.active_node_id = None;
        self.hovered_index = None;
        self.pressed_index = None;
        self.expanded_ids.clear();
        self.expand_transitions.clear();
        self.populate_initial_expands();
        self.sync_expand_transitions_for_tree();
        self.rebuild_flat_cache(None, cx);
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.model.animated == animated {
            return;
        }
        self.model.animated = animated;
        for transition in self.expand_transitions.values_mut() {
            transition.set_animated(animated);
        }
        cx.notify();
    }

    pub fn set_size(&mut self, size: crate::theme::ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::TreeViewTemplate<T>>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;

        if !enabled {
            self.clear_hovered_index(cx);
            self.pressed_index = None;
            self.emit_focus_changed(false, cx);
        }

        cx.emit(TreeViewEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn expand(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        let splice_anchor = self.flat_index_for_id(&node_id);
        let already_expanded = self.expanded_ids.contains(&node_id);
        if !already_expanded {
            self.expanded_ids.insert(node_id.clone());
            if let Some(data) = self.data_for_id(&node_id) {
                cx.emit(TreeViewEvent::NodeExpanded { node_id: node_id.clone(), data });
            }
        }

        let was_visible = self.branch_children_visible(&node_id);
        self.set_expand_target(&node_id, 1.0);

        if !was_visible {
            self.rebuild_flat_cache(splice_anchor, cx);
        } else {
            cx.notify();
        }
    }

    pub fn collapse(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        let was_expanded = self.expanded_ids.remove(&node_id);
        if !was_expanded && !self.branch_children_visible(&node_id) {
            return;
        }

        if was_expanded && let Some(data) = self.data_for_id(&node_id) {
            cx.emit(TreeViewEvent::NodeCollapsed { node_id: node_id.clone(), data });
        }

        self.set_expand_target(&node_id, 0.0);

        if !self.model.animated {
            let splice_anchor = self.flat_index_for_id(&node_id);
            self.rebuild_flat_cache(splice_anchor, cx);
        } else {
            cx.notify();
        }
    }

    pub fn toggle_expand(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if self.expanded_ids.contains(&node_id) {
            self.collapse(node_id, cx);
        } else {
            self.expand(node_id, cx);
        }
    }

    pub fn select_node_by_id(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        if matches!(self.model.selection_mode, TreeViewSelectionMode::None) {
            return;
        }

        let node_id = node_id.into();
        if !self.flat_cache.iter().any(|node| node.id == node_id && node.enabled) {
            return;
        }

        self.selected_ids.clear();
        self.selected_ids.insert(node_id.clone());
        if let Some(idx) = self.flat_index_for_id(&node_id) {
            self.set_active_index(Some(idx), false, cx);
        }
        cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        cx.notify();
    }

    fn populate_initial_expands(&mut self) {
        fn traverse<T>(node: &TreeNode<T>, expanded: &mut HashSet<SharedString>) {
            if node.initially_expanded {
                expanded.insert(node.id.clone());
            }
            for child in &node.children {
                traverse(child, expanded);
            }
        }

        for item in &self.model.items {
            traverse(item, &mut self.expanded_ids);
        }
    }

    fn sync_expand_transitions_for_tree(&mut self) {
        let mut branch_ids = HashSet::new();

        fn collect_branches<T>(node: &TreeNode<T>, branch_ids: &mut HashSet<SharedString>) {
            if !node.children.is_empty() || node.is_branch {
                branch_ids.insert(node.id.clone());
            }
            for child in &node.children {
                collect_branches(child, branch_ids);
            }
        }

        for item in &self.model.items {
            collect_branches(item, &mut branch_ids);
        }

        self.expand_transitions.retain(|id, _| branch_ids.contains(id));
        for id in branch_ids {
            let target = if self.expanded_ids.contains(&id) { 1.0 } else { 0.0 };
            self.expand_transitions
                .entry(id)
                .and_modify(|transition| {
                    if !transition.is_animating() {
                        *transition = DisclosureMotion::new(target, self.model.animated);
                    }
                })
                .or_insert_with(|| DisclosureMotion::new(target, self.model.animated));
        }
    }

    fn expand_progress_for(&self, node_id: &SharedString) -> f32 {
        self.expand_transitions
            .get(node_id)
            .map(DisclosureMotion::progress)
            .unwrap_or_else(|| if self.expanded_ids.contains(node_id) { 1.0 } else { 0.0 })
    }

    fn branch_children_visible(&self, node_id: &SharedString) -> bool {
        self.expand_progress_for(node_id) > EXPAND_VISIBLE_EPSILON
            || self.expand_transitions.get(node_id).is_some_and(DisclosureMotion::is_animating)
    }

    fn set_expand_target(&mut self, node_id: &SharedString, target: f32) {
        let transition = self
            .expand_transitions
            .entry(node_id.clone())
            .or_insert_with(|| DisclosureMotion::new(if target > 0.5 { 0.0 } else { 1.0 }, self.model.animated));
        transition.set_target(target);
    }

    fn sync_expand_transitions(&mut self) -> (bool, bool) {
        let mut was_animating = false;
        let mut is_animating = false;

        for transition in self.expand_transitions.values_mut() {
            was_animating |= transition.is_animating();
            is_animating |= transition.sync();
        }

        (was_animating, is_animating)
    }

    fn desired_flat_count(&self) -> usize {
        fn count_visible<T>(
            node: &TreeNode<T>,
            expanded: &HashSet<SharedString>,
            transitions: &HashMap<SharedString, DisclosureMotion>,
        ) -> usize {
            let expand_progress = transitions
                .get(&node.id)
                .map(DisclosureMotion::progress)
                .unwrap_or_else(|| if expanded.contains(&node.id) { 1.0 } else { 0.0 });
            let show_children = expand_progress > EXPAND_VISIBLE_EPSILON
                || transitions.get(&node.id).is_some_and(DisclosureMotion::is_animating);

            let mut total = 1;
            if show_children {
                for child in &node.children {
                    total += count_visible(child, expanded, transitions);
                }
            }
            total
        }

        self.model
            .items
            .iter()
            .map(|item| count_visible(item, &self.expanded_ids, &self.expand_transitions))
            .sum()
    }

    /// Rebuilds the flattened row list. When `splice_anchor` is the flat index of an
    /// expanded/collapsed branch, list scroll position is preserved via [`ListState::splice`]
    /// instead of [`ListState::reset`].
    fn rebuild_flat_cache(&mut self, splice_anchor: Option<usize>, cx: &mut Context<Self>) {
        let old_count = self.flat_cache.len();
        let active_before = self.active_node_id.clone();
        let mut flat = Vec::new();

        fn flatten<T: Clone>(
            node: &TreeNode<T>,
            depth: usize,
            expanded: &HashSet<SharedString>,
            transitions: &HashMap<SharedString, DisclosureMotion>,
            ancestor_height_factor: f32,
            flat: &mut Vec<FlatNodeWrapper<T>>,
        ) {
            let has_children = !node.children.is_empty() || node.is_branch;
            let expand_progress = transitions
                .get(&node.id)
                .map(DisclosureMotion::progress)
                .unwrap_or_else(|| if expanded.contains(&node.id) { 1.0 } else { 0.0 });

            flat.push(FlatNodeWrapper {
                id: node.id.clone(),
                label: node.label.clone(),
                icon: node.icon,
                depth,
                has_children,
                enabled: node.enabled,
                expand_progress,
                row_height_factor: ancestor_height_factor,
                data: node.data.clone(),
            });

            let show_children = expand_progress > EXPAND_VISIBLE_EPSILON
                || transitions.get(&node.id).is_some_and(DisclosureMotion::is_animating);
            if show_children {
                let child_factor = (ancestor_height_factor * expand_progress).clamp(0.0, 1.0);
                for child in &node.children {
                    flatten(child, depth + 1, expanded, transitions, child_factor, flat);
                }
            }
        }

        for item in &self.model.items {
            flatten(item, 0, &self.expanded_ids, &self.expand_transitions, 1.0, &mut flat);
        }

        self.flat_cache = flat;

        if let Some(active_id) = active_before
            && self.flat_index_for_id(&active_id).is_none()
        {
            self.active_node_id = None;
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: None });
        }

        self.sync_list_state_after_flat_change(old_count, splice_anchor);
        cx.notify();
    }

    fn refresh_flat_motion_factors(&mut self) {
        fn walk<T: Clone>(
            node: &TreeNode<T>,
            expanded: &HashSet<SharedString>,
            transitions: &HashMap<SharedString, DisclosureMotion>,
            ancestor_height_factor: f32,
            flat: &mut [FlatNodeWrapper<T>],
            index: &mut usize,
        ) {
            if *index >= flat.len() {
                return;
            }

            let expand_progress = transitions
                .get(&node.id)
                .map(DisclosureMotion::progress)
                .unwrap_or_else(|| if expanded.contains(&node.id) { 1.0 } else { 0.0 });

            flat[*index].expand_progress = expand_progress;
            flat[*index].row_height_factor = ancestor_height_factor;
            *index += 1;

            let show_children = expand_progress > EXPAND_VISIBLE_EPSILON
                || transitions.get(&node.id).is_some_and(DisclosureMotion::is_animating);
            if show_children {
                let child_factor = (ancestor_height_factor * expand_progress).clamp(0.0, 1.0);
                for child in &node.children {
                    walk(child, expanded, transitions, child_factor, flat, index);
                }
            }
        }

        let mut index = 0;
        for item in &self.model.items {
            walk(item, &self.expanded_ids, &self.expand_transitions, 1.0, &mut self.flat_cache, &mut index);
        }
    }

    fn sync_list_state_after_flat_change(&mut self, old_count: usize, splice_anchor: Option<usize>) {
        let new_count = self.flat_cache.len();
        if new_count == old_count {
            return;
        }

        match splice_anchor {
            Some(toggle_idx) if toggle_idx < old_count => {
                if new_count > old_count {
                    self.list_state.splice(toggle_idx + 1..toggle_idx + 1, new_count - old_count);
                } else {
                    let removed = old_count - new_count;
                    self.list_state.splice(toggle_idx + 1..toggle_idx + 1 + removed, 0);
                }
            }
            _ => {
                if self.list_state.item_count() != new_count {
                    self.list_state.reset(new_count);
                }
            }
        }
    }

    fn flat_index_for_id(&self, node_id: &SharedString) -> Option<usize> {
        self.flat_cache.iter().position(|node| &node.id == node_id)
    }

    fn data_for_id(&self, node_id: &SharedString) -> Option<T> {
        fn find_in_tree<T: Clone>(nodes: &[TreeNode<T>], node_id: &SharedString) -> Option<T> {
            for node in nodes {
                if &node.id == node_id {
                    return Some(node.data.clone());
                }
                if let Some(data) = find_in_tree(&node.children, node_id) {
                    return Some(data);
                }
            }
            None
        }

        find_in_tree(&self.model.items, node_id)
    }

    fn set_active_index(&mut self, index: Option<usize>, scroll: bool, cx: &mut Context<Self>) -> bool {
        let next = index.and_then(|idx| self.flat_cache.get(idx).map(|node| node.id.clone()));
        if self.active_node_id == next {
            if scroll && let Some(idx) = index {
                self.list_state.scroll_to_reveal_item(idx);
                self.emit_scroll_changed_if_needed(cx);
            }
            return false;
        }

        self.active_node_id = next.clone();
        if scroll && let Some(idx) = index {
            self.list_state.scroll_to_reveal_item(idx);
            self.emit_scroll_changed_if_needed(cx);
        }
        cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: next });
        true
    }

    pub fn toggle_node_at_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.flat_cache.len() {
            return;
        }

        let wrapper = &self.flat_cache[index];
        if !wrapper.has_children {
            return;
        }

        let id = wrapper.id.clone();
        if self.expanded_ids.contains(&id) {
            self.collapse(id, cx);
        } else {
            self.expand(id, cx);
        }
    }

    fn render_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(node) = self.flat_cache.get(index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let item_enabled = self.model.enabled && node.enabled;
        let selected = self.selected_ids.contains(&node.id);
        let active = self.active_node_id.as_ref() == Some(&node.id);
        let hovered = item_enabled && self.hovered_index == Some(index);
        let pressed = item_enabled && self.pressed_index == Some(index);

        let state = CompositeItemState {
            hovered,
            pressed,
            disabled: !item_enabled,
            selected,
            active: focus.focused && active,
            focus_visible: focus.focus_visible && active,
        };

        let render_node = FlatTreeNode {
            id: &node.id,
            label: &node.label,
            icon: node.icon,
            depth: node.depth,
            has_children: node.has_children,
            expanded: self.expanded_ids.contains(&node.id),
            expand_progress: node.expand_progress,
            disclosure_icons: &self.model.disclosure_icons,
            row_height_factor: node.row_height_factor,
            enabled: item_enabled,
            size: self.model.size,
            state,
            data: &node.data,
        };

        self.model.template.render_node(&render_node, self.template_row_handlers(index, cx), window, cx)
    }

    fn handle_row_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.flat_cache.get(index).is_some_and(|node| node.enabled) {
            return;
        }

        if hovered {
            if self.hovered_index != Some(index) {
                self.clear_hovered_index(cx);
                self.hovered_index = Some(index);
                let node_id = self.flat_cache[index].id.clone();
                cx.emit(TreeViewEvent::RowHoverChanged { node_id, index, hovered: true });
                cx.notify();
            }
        } else if self.hovered_index == Some(index) {
            self.clear_hovered_index(cx);
            if self.pressed_index == Some(index) {
                self.pressed_index = None;
            }
            cx.notify();
        }
    }

    fn template_row_handlers(&self, idx: usize, cx: &mut Context<Self>) -> TreeViewTemplateHandlers {
        TreeViewTemplateHandlers {
            hover: Box::new(cx.listener(move |this, hovered: &bool, _, cx| {
                this.handle_row_hover(idx, *hovered, cx);
            })),
            mouse_down: Box::new(cx.listener(move |this, _, window, cx| {
                if this.model.enabled && this.flat_cache.get(idx).is_some_and(|node| node.enabled) {
                    this.pressed_index = Some(idx);
                    this.set_active_index(Some(idx), false, cx);
                    this.focus_handle.focus(window, cx);
                    cx.notify();
                }
            })),
            mouse_up: Box::new(cx.listener(move |this, _, _, cx| {
                this.pressed_index = None;
                cx.notify();
            })),
            click: Box::new(cx.listener(move |this, event: &ClickEvent, _, cx| {
                if event.click_count() != 1 {
                    return;
                }

                let is_branch = this.flat_cache.get(idx).is_some_and(|node| node.has_children);
                if is_branch {
                    this.toggle_node_at_index(idx, cx);
                }
                this.handle_node_select(idx, cx);
            })),
        }
    }

    fn handle_node_select(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx >= self.flat_cache.len() {
            return;
        }

        let node = &self.flat_cache[idx];
        if !self.model.enabled || !node.enabled {
            return;
        }

        let id = node.id.clone();
        match self.model.selection_mode {
            TreeViewSelectionMode::None => {}
            TreeViewSelectionMode::Single => {
                self.selected_ids.clear();
                self.selected_ids.insert(id);
            }
            TreeViewSelectionMode::Multiple => {
                if self.selected_ids.contains(&id) {
                    self.selected_ids.remove(&id);
                } else {
                    self.selected_ids.insert(id);
                }
            }
        }

        self.set_active_index(Some(idx), false, cx);
        cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        cx.notify();
    }

    fn move_active(&mut self, direction: TreeDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let current = self.active_node_id.as_ref().and_then(|id| self.flat_index_for_id(id));
        let next = next_enabled_flat_index(&self.flat_cache, current, direction);
        if let Some(idx) = next {
            self.set_active_index(Some(idx), true, cx);
            cx.notify();
        }
    }

    fn handle_previous(&mut self, _: &SelectPreviousItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(TreeDirection::Previous, cx);
    }

    fn handle_next(&mut self, _: &SelectNextItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(TreeDirection::Next, cx);
    }

    fn handle_first(&mut self, _: &SelectFirstItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = first_enabled_flat_index(&self.flat_cache) {
            self.set_active_index(Some(idx), true, cx);
            cx.notify();
        }
    }

    fn handle_last(&mut self, _: &SelectLastItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = last_enabled_flat_index(&self.flat_cache) {
            self.set_active_index(Some(idx), true, cx);
            cx.notify();
        }
    }

    fn handle_expand_focused_node(&mut self, _: &ExpandNode, _: &mut Window, cx: &mut Context<Self>) {
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        let node = &self.flat_cache[idx];
        if !node.has_children {
            return;
        }

        if !self.expanded_ids.contains(&active_id) {
            self.toggle_node_at_index(idx, cx);
            return;
        }

        if idx + 1 < self.flat_cache.len() {
            self.set_active_index(Some(idx + 1), true, cx);
            cx.notify();
        }
    }

    fn handle_collapse_focused_node(&mut self, _: &CollapseNode, _: &mut Window, cx: &mut Context<Self>) {
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        let node = &self.flat_cache[idx];
        if node.has_children && self.expanded_ids.contains(&active_id) {
            self.toggle_node_at_index(idx, cx);
            return;
        }

        if node.depth == 0 {
            return;
        }

        let current_depth = node.depth;
        let mut search_idx = idx;
        while search_idx > 0 {
            search_idx -= 1;
            if self.flat_cache[search_idx].depth < current_depth {
                self.set_active_index(Some(search_idx), true, cx);
                cx.notify();
                break;
            }
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        if self.flat_cache.get(idx).is_some_and(|node| node.has_children) {
            self.toggle_node_at_index(idx, cx);
        } else {
            self.handle_node_select(idx, cx);
        }
    }

    fn clear_hovered_index(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.hovered_index.take()
            && let Some(node) = self.flat_cache.get(index)
        {
            cx.emit(TreeViewEvent::RowHoverChanged { node_id: node.id.clone(), index, hovered: false });
        }
    }

    fn handle_scroll_wheel(&mut self, _event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_scroll_changed_if_needed(cx) {
            cx.notify();
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    fn emit_scroll_changed_if_needed(&mut self, cx: &mut Context<Self>) -> bool {
        let top_index = self.list_state.logical_scroll_top().item_ix;
        if self.emitted_scroll_top_index == top_index {
            return false;
        }

        self.emitted_scroll_top_index = top_index;
        cx.emit(TreeViewEvent::ScrollChanged { top_index });
        true
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(TreeViewEvent::FocusChanged { focused });
        true
    }
}

impl<T> Focusable for TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let (was_animating, is_animating) = self.sync_expand_transitions();
        if self.desired_flat_count() != self.flat_cache.len() {
            self.rebuild_flat_cache(None, cx);
        } else if was_animating || is_animating {
            self.refresh_flat_motion_factors();
        }

        for transition in self.expand_transitions.values() {
            transition.schedule_frame(window, cx);
        }
        if was_animating || is_animating {
            cx.notify();
        }

        let render_model = TreeViewRenderModel {
            id: &self.model.id,
            selection_mode: self.model.selection_mode,
            enabled: self.model.enabled,
            size: self.model.size,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        };

        let body = list(self.list_state.clone(), cx.processor(Self::render_row)).size_full().into_any_element();

        self.model
            .template
            .render(&render_model, body, window, cx)
            .track_focus(&self.focus_handle)
            .key_context(ControlKeyProfile::Selector.context())
            .on_action(cx.listener(Self::handle_previous))
            .on_action(cx.listener(Self::handle_next))
            .on_action(cx.listener(Self::handle_first))
            .on_action(cx.listener(Self::handle_last))
            .on_action(cx.listener(Self::handle_expand_focused_node))
            .on_action(cx.listener(Self::handle_collapse_focused_node))
            .on_action(cx.listener(Self::handle_activate))
            .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel))
    }
}

fn first_enabled_flat_index<T>(flat: &[FlatNodeWrapper<T>]) -> Option<usize> {
    flat.iter().position(|node| node.enabled)
}

fn last_enabled_flat_index<T>(flat: &[FlatNodeWrapper<T>]) -> Option<usize> {
    flat.iter().rposition(|node| node.enabled)
}

fn next_enabled_flat_index<T>(
    flat: &[FlatNodeWrapper<T>],
    current_index: Option<usize>,
    direction: TreeDirection,
) -> Option<usize> {
    let len = flat.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        TreeDirection::Previous => len - 1,
        TreeDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(idx) => (idx + step) % len,
        None if direction == TreeDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if flat[index].enabled {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{TreeDirection, first_enabled_flat_index, next_enabled_flat_index};

    fn flat_nodes() -> Vec<super::FlatNodeWrapper<&'static str>> {
        vec![
            super::FlatNodeWrapper {
                id: "a".into(),
                label: "A".into(),
                icon: None,
                depth: 0,
                has_children: true,
                enabled: true,
                expand_progress: 1.0,
                row_height_factor: 1.0,
                data: "a",
            },
            super::FlatNodeWrapper {
                id: "b".into(),
                label: "B".into(),
                icon: None,
                depth: 1,
                has_children: false,
                enabled: false,
                expand_progress: 0.0,
                row_height_factor: 1.0,
                data: "b",
            },
            super::FlatNodeWrapper {
                id: "c".into(),
                label: "C".into(),
                icon: None,
                depth: 0,
                has_children: false,
                enabled: true,
                expand_progress: 0.0,
                row_height_factor: 1.0,
                data: "c",
            },
        ]
    }

    #[test]
    fn flat_navigation_skips_disabled_rows() {
        let flat = flat_nodes();

        assert_eq!(first_enabled_flat_index(&flat), Some(0));
        assert_eq!(next_enabled_flat_index(&flat, Some(0), TreeDirection::Next), Some(2));
        assert_eq!(next_enabled_flat_index(&flat, Some(2), TreeDirection::Next), Some(0));
    }
}
