use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui::{Context, SharedString, Window};

use super::super::{NavNode, NavNodeKind};
use super::SidebarPanelEngine;
use crate::motion::{DEFAULT_TRANSITION_DURATION, VisualTransition};

impl SidebarPanelEngine {
    pub(super) fn transition_duration(&self) -> Duration {
        if self.model.animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        }
    }

    pub(super) fn sync_branch_state(&mut self) {
        let mut branch_ids = HashSet::new();
        fn collect(nodes: &[NavNode], ids: &mut HashSet<SharedString>) {
            for node in nodes {
                if !node.children.is_empty() {
                    ids.insert(node.id.clone());
                }
                collect(&node.children, ids);
            }
        }
        collect(&self.model.header_nodes, &mut branch_ids);
        collect(&self.model.nodes, &mut branch_ids);
        collect(&self.model.footer_nodes, &mut branch_ids);
        self.branch_transitions.retain(|id, _| branch_ids.contains(id));
        self.branch_heights_px.retain(|id, _| branch_ids.contains(id));
        let duration = self.transition_duration();
        for id in branch_ids {
            let expanded = self.find_node(&id).is_some_and(|node| node.expanded);
            self.branch_transitions
                .entry(id)
                .or_insert_with(|| VisualTransition::new(if expanded { 1.0 } else { 0.0 }, duration));
        }
    }

    pub(super) fn set_branch_target(&mut self, node_id: &SharedString, expanded: bool) {
        let duration = self.transition_duration();
        let transition = self
            .branch_transitions
            .entry(node_id.clone())
            .or_insert_with(|| VisualTransition::new(if expanded { 0.0 } else { 1.0 }, duration));
        if self.model.animated {
            transition.set_target(if expanded { 1.0 } else { 0.0 });
        } else {
            transition.snap_to(if expanded { 1.0 } else { 0.0 });
        }
    }

    pub(super) fn sync_branch_transitions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut animating = false;
        for transition in self.branch_transitions.values_mut() {
            transition.sync();
            animating |= transition.is_animating();
        }
        if animating {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
    }

    pub(super) fn set_branch_height(&mut self, node_id: SharedString, height: f32, cx: &mut Context<Self>) {
        let height = height.max(0.0);
        let settled_open = self
            .branch_transitions
            .get(&node_id)
            .is_some_and(|transition| !transition.is_animating() && transition.progress() >= 1.0 - f32::EPSILON);
        let cached = self.branch_heights_px.entry(node_id).or_insert(0.0);
        if (settled_open || height > *cached + 0.5) && (height - *cached).abs() > 0.5 {
            *cached = height;
            cx.notify();
        }
    }

    pub(super) fn can_interact_with_default_node(&self, node_id: &SharedString) -> bool {
        self.model.enabled
            && self
                .find_node(node_id)
                .is_some_and(|node| node.enabled && node.kind == NavNodeKind::Item && node.presenter.is_none())
    }

    pub(super) fn find_node(&self, node_id: &SharedString) -> Option<&NavNode> {
        find_node_in_nodes(&self.model.nodes, node_id)
            .or_else(|| find_node_in_nodes(&self.model.header_nodes, node_id))
            .or_else(|| find_node_in_nodes(&self.model.footer_nodes, node_id))
    }

    pub(super) fn find_top_level_main_node(&self, node_id: &SharedString) -> Option<&NavNode> {
        self.model.nodes.iter().find(|node| node.id == *node_id)
    }

    pub(super) fn can_interact_with_rail_node(&self, node_id: &SharedString) -> bool {
        self.model.enabled
            && self
                .find_top_level_main_node(node_id)
                .is_some_and(|node| collapsed_rail_node_visible(node) && node.enabled)
    }

    pub(super) fn node_label(&self, node_id: &SharedString) -> SharedString {
        self.find_node(node_id).and_then(|node| node.label.clone()).unwrap_or_else(|| node_id.clone())
    }
}

pub(super) fn branch_progress_for(
    transitions: &HashMap<SharedString, VisualTransition>,
    node_id: &SharedString,
    expanded: bool,
) -> f32 {
    transitions.get(node_id).map(VisualTransition::progress).unwrap_or(if expanded { 1.0 } else { 0.0 })
}

pub(super) fn branch_visible_for(
    transitions: &HashMap<SharedString, VisualTransition>,
    node_id: &SharedString,
    expanded: bool,
) -> bool {
    branch_progress_for(transitions, node_id, expanded) > f32::EPSILON
        || transitions.get(node_id).is_some_and(VisualTransition::is_animating)
}
pub(super) fn collapsed_rail_node_count(nodes: &[NavNode]) -> usize {
    nodes.iter().filter(|node| collapsed_rail_node_visible(node)).count()
}

pub(super) fn collapsed_rail_node_visible(node: &NavNode) -> bool {
    node.visible && node.kind == NavNodeKind::Item && node.icon.is_some()
}
pub(super) fn set_expanded_in_nodes(nodes: &mut [NavNode], node_id: &SharedString, expanded: bool) -> bool {
    for node in nodes {
        if node.id == *node_id && !node.children.is_empty() && node.enabled && node.expanded != expanded {
            node.expanded = expanded;
            return true;
        }

        if set_expanded_in_nodes(&mut node.children, node_id, expanded) {
            return true;
        }
    }

    false
}

pub(super) fn find_node_in_nodes<'a>(nodes: &'a [NavNode], node_id: &SharedString) -> Option<&'a NavNode> {
    for node in nodes {
        if node.id == *node_id {
            return Some(node);
        }

        if let Some(node) = find_node_in_nodes(&node.children, node_id) {
            return Some(node);
        }
    }

    None
}

pub(super) fn toggle_expanded_in_nodes(nodes: &mut [NavNode], node_id: &SharedString) -> Option<bool> {
    for node in nodes {
        if node.id == *node_id && !node.children.is_empty() && node.enabled {
            node.expanded = !node.expanded;
            return Some(node.expanded);
        }

        if let Some(expanded) = toggle_expanded_in_nodes(&mut node.children, node_id) {
            return Some(expanded);
        }
    }

    None
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use gpui::SharedString;
    use lucide_svg_static::Icon as LucideIcon;

    use crate::motion::VisualTransition;

    use super::{
        branch_progress_for, branch_visible_for, collapsed_rail_node_count, set_expanded_in_nodes,
        toggle_expanded_in_nodes,
    };
    use super::super::NavNode;

    #[test]
    fn toggles_enabled_branch() {
        let branch_id: SharedString = "inputs".into();
        let mut nodes = vec![NavNode::new("inputs").child(NavNode::new("button"))];

        assert_eq!(toggle_expanded_in_nodes(&mut nodes, &branch_id), Some(true));
        assert!(nodes[0].is_expanded());
    }

    #[test]
    fn ignores_disabled_branch() {
        let branch_id: SharedString = "inputs".into();
        let mut nodes = vec![NavNode::new("inputs").child(NavNode::new("button")).enabled(false)];

        assert_eq!(toggle_expanded_in_nodes(&mut nodes, &branch_id), None);
        assert!(!set_expanded_in_nodes(&mut nodes, &branch_id, true));
    }

    #[test]
    fn collapsed_rail_only_counts_visible_top_level_item_nodes_with_icons() {
        let nodes = vec![
            NavNode::section("section", "Section").icon(LucideIcon::List),
            NavNode::new("icon-leaf").icon(LucideIcon::House),
            NavNode::new("icon-branch")
                .icon(LucideIcon::Settings)
                .child(NavNode::new("child").icon(LucideIcon::Circle)),
            NavNode::new("label-only"),
            NavNode::new("hidden").icon(LucideIcon::Search).visible(false),
        ];

        assert_eq!(collapsed_rail_node_count(&nodes), 2);
    }

    #[test]
    fn closing_branch_remains_visible_while_transition_is_active() {
        let branch_id: SharedString = "inputs".into();
        let mut transitions =
            HashMap::from([(branch_id.clone(), VisualTransition::new(1.0, Duration::from_millis(200)))]);
        transitions.get_mut(&branch_id).unwrap().set_target(0.0);

        assert!(branch_visible_for(&transitions, &branch_id, false));
        assert!(branch_progress_for(&transitions, &branch_id, false) > 0.0);
    }
}
