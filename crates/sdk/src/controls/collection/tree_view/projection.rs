//! A display hierarchy of source indices; domain data and sibling order stay owned by the host.
use std::{cmp::Ordering, collections::HashSet, rc::Rc};
use gpui::SharedString;
use super::TreeNode;

pub(super) type Filter<T> = Rc<dyn Fn(&TreeNode<T>) -> bool>;
pub(super) type Sort<T> = Rc<dyn Fn(&TreeNode<T>, &TreeNode<T>) -> Ordering>;

struct ProjectedNode {
    source_index: usize,
    children: Vec<ProjectedNode>,
}

#[derive(Default)]
pub(super) struct Projection {
    roots: Vec<ProjectedNode>,
    pub forced_expanded: HashSet<SharedString>,
}

impl Projection {
    pub fn new<T>(items: &[TreeNode<T>], filter: Option<&Filter<T>>, sort: Option<&Sort<T>>) -> Self {
        fn project<T>(
            items: &[TreeNode<T>],
            filter: Option<&Filter<T>>,
            sort: Option<&Sort<T>>,
            forced: &mut HashSet<SharedString>,
        ) -> Vec<ProjectedNode> {
            let mut nodes = Vec::new();
            for (source_index, node) in items.iter().enumerate() {
                let children = project(&node.children, filter, sort, forced);
                if filter.is_none_or(|matches| matches(node)) || !children.is_empty() {
                    if filter.is_some() && !children.is_empty() {
                        forced.insert(node.id.clone());
                    }
                    nodes.push(ProjectedNode { source_index, children });
                }
            }
            if let Some(compare) = sort {
                // Stable ties retain source sibling order.
                nodes.sort_by(|a, b| compare(&items[a.source_index], &items[b.source_index]));
            }
            nodes
        }
        let mut projection = Self::default();
        projection.roots = project(items, filter, sort, &mut projection.forced_expanded);
        projection
    }

    /// Preorder traversal; returning false skips the node's projected descendants.
    pub fn visit<T>(&self, items: &[TreeNode<T>], mut visitor: impl FnMut(&TreeNode<T>, usize) -> bool) {
        fn walk<T>(
            nodes: &[ProjectedNode],
            items: &[TreeNode<T>],
            depth: usize,
            visitor: &mut impl FnMut(&TreeNode<T>, usize) -> bool,
        ) {
            for projected in nodes {
                let node = &items[projected.source_index];
                if visitor(node, depth) {
                    walk(&projected.children, &node.children, depth + 1, visitor);
                }
            }
        }
        walk(&self.roots, items, 0, &mut visitor);
    }

    pub fn visible<T>(&self, items: &[TreeNode<T>], expanded: &HashSet<SharedString>) -> Vec<SharedString> {
        let mut ids = Vec::new();
        self.visit(items, |node, _| {
            ids.push(node.id.clone());
            expanded.contains(&node.id) || self.forced_expanded.contains(&node.id)
        });
        ids
    }
}
