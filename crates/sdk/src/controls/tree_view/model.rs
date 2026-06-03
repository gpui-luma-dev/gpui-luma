use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{TreeViewControl, TreeViewTemplate, default_tree_view_template};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeViewSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
}

/// A node in the hierarchical tree model.
#[derive(Clone)]
pub struct TreeNode<T> {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<LucideIcon>,
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    /// Show a branch chevron even when `children` is empty (lazy loading).
    pub is_branch: bool,
    pub initially_expanded: bool,
    pub enabled: bool,
}

impl<T> TreeNode<T> {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, data: T) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            data,
            children: Vec::new(),
            is_branch: false,
            initially_expanded: false,
            enabled: true,
        }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn branch(mut self, is_branch: bool) -> Self {
        self.is_branch = is_branch;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.children = children.into_iter().collect();
        self.is_branch = !self.children.is_empty() || self.is_branch;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Read-only view of one flattened row passed to templates.
pub struct FlatTreeNode<'a, T> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub icon: Option<LucideIcon>,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub enabled: bool,
    pub state: crate::controls::state::CompositeItemState,
    pub data: &'a T,
}

#[derive(Clone)]
pub struct TreeViewModel<T>
where
    T: Send + Sync + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) items: Vec<TreeNode<T>>,
    pub(crate) selection_mode: TreeViewSelectionMode,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn TreeViewTemplate<T>>,
}

pub struct TreeViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub selection_mode: TreeViewSelectionMode,
    pub enabled: bool,
    pub focus: crate::controls::state::ControlFocusState,
}

pub struct TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) model: TreeViewModel<T>,
}

impl<T> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: TreeViewModel {
                id: id.into(),
                items: Vec::new(),
                selection_mode: TreeViewSelectionMode::Single,
                enabled: true,
                template: default_tree_view_template::<T>(),
            },
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TreeViewControl<T>> {
        cx.new(|cx| TreeViewControl::from_builder(self, cx))
    }
}
