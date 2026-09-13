//! Look-owned tree-view builder. Spawn synthesizes the SDK [`luma::controls::tree_view::TreeView`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use luma::controls::tree_view::{TreeNode, TreeViewBuilder, TreeViewControl, TreeViewSelectionMode, TreeViewTemplate};
use luma::infra::icon::DisclosureIcons;
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

    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.builder = self.builder.selection_mode(mode);
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

    pub fn animated(mut self, animated: bool) -> Self {
        self.builder = self.builder.animated(animated);
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
        let builder = self.builder.size(self.size.control_size());
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
