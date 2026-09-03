mod control;
mod model;
mod template;
mod theme;

pub use control::{CollapseNode, ExpandNode, TreeViewControl, TreeViewEvent};
pub use model::{FlatTreeNode, TreeNode, TreeViewBuilder, TreeViewModel, TreeViewRenderModel, TreeViewSelectionMode};
pub use template::{
    TreeViewTemplate, TreeViewTemplateHandlers, TreeViewTemplateModifier, ThemedTreeViewTemplate,
    default_tree_view_template,
};
pub use theme::{DefaultTreeViewTheme, TreeViewPalette, TreeViewScale, TreeViewTheme, default_tree_view_theme};

pub use crate::infra::state::{CompositeItemState as TreeViewItemState, ControlFocusState};
use gpui::{Entity, SharedString};

/// Entity handle for a tree view control.
pub type TreeView<T> = Entity<TreeViewControl<T>>;

/// Creates a new builder for a [`TreeView`].
pub fn new<T>(id: impl Into<SharedString>) -> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    TreeViewBuilder::new(id)
}
