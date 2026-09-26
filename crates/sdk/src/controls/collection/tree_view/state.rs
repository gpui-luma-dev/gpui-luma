//! Validated identity and hierarchy metadata, independent of rendering and motion.
use std::collections::{HashMap, HashSet};
use std::fmt;

use gpui::SharedString;

use super::TreeNode;

/// Invalid tree data. Failed replacements leave the entire control unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TreeViewError {
    /// IDs must be unique across all loaded nodes, including collapsed descendants.
    DuplicateId(SharedString),
}

impl fmt::Display for TreeViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate tree node ID: {id}"),
        }
    }
}

impl std::error::Error for TreeViewError {}

#[derive(Clone)]
pub(super) struct NodeInfo {
    pub parent: Option<SharedString>,
    pub branch: bool,
    pub enabled: bool,
    initially_expanded: bool,
}

#[derive(Clone, Default)]
pub(super) struct TreeIndex {
    pub nodes: HashMap<SharedString, NodeInfo>,
}

impl TreeIndex {
    pub fn new<T>(items: &[TreeNode<T>]) -> Result<Self, TreeViewError> {
        let mut index = Self::default();
        let mut pending: Vec<_> = items.iter().rev().map(|node| (node, None)).collect();
        while let Some((node, parent)) = pending.pop() {
            let info = NodeInfo {
                parent,
                branch: node.is_branch || !node.children.is_empty(),
                enabled: node.enabled,
                initially_expanded: node.initially_expanded,
            };
            if index.nodes.insert(node.id.clone(), info).is_some() {
                return Err(TreeViewError::DuplicateId(node.id.clone()));
            }
            pending.extend(node.children.iter().rev().map(|child| (child, Some(node.id.clone()))));
        }
        Ok(index)
    }

    pub fn is_in_subtrees(&self, id: &SharedString, roots: &HashSet<&SharedString>) -> bool {
        let mut current = Some(id);
        while let Some(id) = current {
            if roots.contains(id) {
                return true;
            }
            current = self.nodes.get(id).and_then(|node| node.parent.as_ref());
        }
        false
    }

    pub fn initial_expanded(&self) -> HashSet<SharedString> {
        self.nodes
            .iter()
            .filter(|(_, info)| info.branch && info.initially_expanded)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn reconcile_expanded(&self, old: &Self, expanded: &HashSet<SharedString>) -> HashSet<SharedString> {
        self.nodes
            .iter()
            .filter(|(id, info)| {
                info.branch && (expanded.contains(*id) || (!old.nodes.contains_key(*id) && info.initially_expanded))
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Prefer the retained active key, then a visible ancestor (new ancestry for
    /// reparenting, old ancestry for removal), then the closest displayed row.
    /// An inactive control stays inactive across replacement.
    pub fn reconcile_active(
        &self,
        old: &Self,
        active: Option<&SharedString>,
        old_visible: &[SharedString],
        visible: &[SharedString],
    ) -> Option<SharedString> {
        let active = active?;
        let eligible: HashSet<_> = visible.iter().filter(|id| self.nodes[*id].enabled).collect();
        if eligible.contains(active) {
            return Some(active.clone());
        }
        for index in [self, old] {
            let mut parent = index.nodes.get(active).and_then(|info| info.parent.as_ref());
            while let Some(id) = parent {
                if eligible.contains(id) {
                    return Some(id.clone());
                }
                parent = index.nodes.get(id).and_then(|info| info.parent.as_ref());
            }
        }
        let previous_index = old_visible.iter().position(|id| id == active).unwrap_or(0);
        visible
            .iter()
            .enumerate()
            .filter(|(_, id)| eligible.contains(*id))
            .min_by_key(|(index, _)| index.abs_diff(previous_index))
            .map(|(_, id)| id.clone())
    }
}
