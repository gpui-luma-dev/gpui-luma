//! Look-owned tree-view builder. Spawn synthesizes the SDK [`luma::controls::tree_view::TreeView`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use luma::controls::tree_view::{TreeNode, TreeViewBuilder, TreeViewControl, TreeViewSelectionMode, TreeViewTemplate};
use luma::infra::icon::DisclosureIcons;
use luma::controls::tree_view::TreeViewError;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a tree view: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct TreeView<T>
where
    T: Clone + Send + Sync + 'static,
{
    look: Option<ShadcnLook>,
    builder: TreeViewBuilder<T>,
    custom_template: bool,
    size: ShadcnSize,
}

impl<T> TreeView<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: luma::controls::tree_view::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set initial items, retaining previous items if IDs are duplicated.
    /// Use [`Self::try_items`] when validation errors need to be reported.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    /// Validate IDs across all loaded descendants before constructing the tree.
    pub fn try_items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Result<Self, TreeViewError> {
        self.builder = self.builder.try_items(items)?;
        Ok(self)
    }

    /// Show loaded matches and ancestor paths, retaining user expansion separately.
    pub fn filter(mut self, predicate: impl Fn(&TreeNode<T>) -> bool + 'static) -> Self {
        self.builder = self.builder.filter(predicate);
        self
    }

    /// Stable display-only ordering within each sibling group.
    pub fn sort(mut self, compare: impl Fn(&TreeNode<T>, &TreeNode<T>) -> std::cmp::Ordering + 'static) -> Self {
        self.builder = self.builder.sort(compare);
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.builder = self.builder.selection_mode(mode);
        self
    }

    /// Configure selection mode, repeated-click deselection and keyboard selection.
    pub fn selection_policy(mut self, policy: luma::controls::tree_view::TreeViewSelectionPolicy) -> Self {
        self.builder = self.builder.selection_policy(policy);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    /// Animate small branches; transitions affecting over 256 displayed
    /// descendants settle before layout to preserve virtualization.
    pub fn animated(mut self, animated: bool) -> Self {
        self.builder = self.builder.animated(animated);
        self
    }

    /// Pass unfocused wheel input to the page; defaults to hover scrolling.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.builder = self.builder.require_focus_for_scroll(required);
        self
    }

    /// Opt out of the legacy Up/Down wrapping behavior.
    pub fn wrap_navigation(mut self, wrap: bool) -> Self {
        self.builder = self.builder.wrap_navigation(wrap);
        self
    }

    /// Set scrollbar chrome visibility. AutoHide (the default) shows while scrolling.
    pub fn scrollbar_visibility(mut self, visibility: luma::controls::ScrollbarVisibility) -> Self {
        self.builder = self.builder.scrollbar_visibility(visibility);
        self
    }

    /// Enable scoped dragging and host-owned drop transactions.
    /// The configuration can opt into dragging selected groups.
    pub fn drag_drop(mut self, config: luma::controls::tree_view::TreeViewDragDrop<T>) -> Self {
        self.builder = self.builder.drag_drop(config);
        self
    }

    /// Named or inline content inside the Shadcn branch row shell.
    pub fn branch_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&luma::controls::tree_view::FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.builder = self.builder.branch_content(content);
        self
    }

    /// Named or inline content inside the Shadcn leaf row shell.
    pub fn leaf_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&luma::controls::tree_view::FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.builder = self.builder.leaf_content(content);
        self
    }

    /// False reserves pointer expansion for the disclosure control.
    pub fn expand_on_row_click(mut self, expand: bool) -> Self {
        self.builder = self.builder.expand_on_row_click(expand);
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.builder = self.builder.disclosure_icons(icons);
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate<T>>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<TreeViewControl<T>> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TreeViewBuilder<T> {
        let builder = self.builder.size(self.size.control_size()).scrollbar_template(look.scrollbar_template());
        if self.custom_template {
            builder
        } else {
            builder.template(look.tree_view_template::<T>())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = TreeView::new("ok")
            .look(&look)
            .items([TreeNode::new("root", "Root", SharedString::from("root"))])
            .into_sdk_builder(look);
    }
}
