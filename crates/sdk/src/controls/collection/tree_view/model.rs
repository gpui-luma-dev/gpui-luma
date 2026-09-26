use std::{sync::Arc, rc::Rc};

/// Owned UI content callback; captures need not be Send or Sync.
pub type TreeViewContent<T> = Rc<dyn Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> gpui::AnyElement>;

use gpui::{AppContext, Entity, SharedString};
use lucide_svg_static::Icon as LucideIcon;

use super::{TreeViewControl, TreeViewTemplate, default_tree_view_template};
use super::template::modified_tree_view_template;
use super::state::TreeIndex;
use super::TreeViewError;
use crate::theme::ControlSize;
use crate::infra::icon::DisclosureIcons;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeViewSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
    /// Plain clicks replace selection; Ctrl/Cmd toggles and Shift extends a
    /// stable-ID range in displayed preorder. Enter activates; Space selects.
    Extended,
}

/// Selection settings; defaults preserve the existing Single/Multiple behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeViewSelectionPolicy {
    pub mode: TreeViewSelectionMode,
    /// Repeated plain clicks deselect in Single or Extended. Extended retains
    /// other selected nodes; modifier and range gestures are unchanged.
    pub toggle_off: bool,
    /// Keyboard navigation selects its target in Single mode only.
    /// Enabling this does not immediately change selection.
    pub selection_follows_active: bool,
}

impl TreeViewSelectionPolicy {
    pub fn new(mode: TreeViewSelectionMode) -> Self {
        Self { mode, toggle_off: false, selection_follows_active: false }
    }
}

/// A node in the hierarchical tree model.
#[derive(Clone)]
pub struct TreeNode<T> {
    /// Stable identity, unique across every loaded node in this tree. Keep it
    /// unchanged when moving or renaming a node; do not derive it from its path.
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
    /// Branch expand progress (`0..1`) for chevron rotation.
    pub expand_progress: f32,
    pub disclosure_icons: &'a DisclosureIcons,
    /// Layout height factor (`0..1`) for clipped expand/collapse of descendant rows.
    pub row_height_factor: f32,
    pub enabled: bool,
    pub size: ControlSize,
    pub state: crate::infra::state::CompositeItemState,
    pub data: &'a T,
}

#[derive(Clone)]
pub struct TreeViewModel<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) items: Vec<TreeNode<T>>,
    pub(super) index: TreeIndex,
    pub(super) filter: Option<super::projection::Filter<T>>,
    pub(super) sort: Option<super::projection::Sort<T>>,
    pub(crate) selection_policy: TreeViewSelectionPolicy,
    pub(crate) enabled: bool,
    pub(crate) size: ControlSize,
    pub(crate) animated: bool,
    pub(super) drag_drop: Option<super::TreeViewDragDrop<T>>,
    pub(super) branch_content: Option<TreeViewContent<T>>,
    pub(super) leaf_content: Option<TreeViewContent<T>>,
    pub(super) expand_on_row_click: bool,
    pub(super) require_focus_for_scroll: bool,
    pub(super) wrap_navigation: bool,
    pub(super) scrollbar_visibility: crate::controls::ScrollbarVisibility,
    pub(super) scrollbar_template: Option<Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>>,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn TreeViewTemplate<T>>,
}

pub struct TreeViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub selection_mode: TreeViewSelectionMode,
    pub enabled: bool,
    pub size: ControlSize,
    pub focus: crate::infra::state::ControlFocusState,
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
                index: TreeIndex::default(),
                filter: None,
                sort: None,
                selection_policy: TreeViewSelectionPolicy::new(TreeViewSelectionMode::Single),
                enabled: true,
                size: ControlSize::Md,
                animated: true,
                drag_drop: None,
                branch_content: None,
                leaf_content: None,
                expand_on_row_click: true,
                require_focus_for_scroll: false,
                wrap_navigation: true,
                scrollbar_visibility: crate::controls::ScrollbarVisibility::AutoHide,
                scrollbar_template: None,
                disclosure_icons: DisclosureIcons::default(),
                template: default_tree_view_template::<T>(),
            },
        }
    }

    /// Set initial items. Invalid duplicate IDs leave the previous items intact.
    /// Use [`Self::try_items`] to report validation errors to the caller.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        if let Ok(index) = TreeIndex::new(&items) {
            self.model.items = items;
            self.model.index = index;
        }
        self
    }

    /// Validate IDs across the entire loaded tree before setting initial items.
    pub fn try_items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Result<Self, TreeViewError> {
        let items: Vec<_> = items.into_iter().collect();
        let index = TreeIndex::new(&items)?;
        self.model.items = items;
        self.model.index = index;
        Ok(self)
    }

    /// Show loaded matches and their ancestor paths. Matching a branch does not
    /// include unmatched descendants. Ancestors open without changing user expansion.
    /// The predicate is reevaluated on data replacement or `set_filter`.
    pub fn filter(mut self, predicate: impl Fn(&TreeNode<T>) -> bool + 'static) -> Self {
        self.model.filter = Some(Rc::new(predicate));
        self
    }

    /// Stable display-only sibling ordering; source items are never reordered.
    pub fn sort(mut self, compare: impl Fn(&TreeNode<T>, &TreeNode<T>) -> std::cmp::Ordering + 'static) -> Self {
        self.model.sort = Some(Rc::new(compare));
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.model.selection_policy.mode = mode;
        self
    }

    /// Configure optional plain-click deselection and Single-mode keyboard selection.
    pub fn selection_policy(mut self, policy: TreeViewSelectionPolicy) -> Self {
        self.model.selection_policy = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    /// Animate small branch transitions. When simultaneous transitions affect
    /// more than 256 displayed descendants, settle before layout to keep row
    /// construction bounded as descendant heights shrink.
    pub fn animated(mut self, animated: bool) -> Self {
        self.model.animated = animated;
        self
    }

    /// Enable scoped dragging and synchronous host-owned drops.
    /// The configuration can opt into dragging selected groups.
    pub fn drag_drop(mut self, config: super::TreeViewDragDrop<T>) -> Self {
        self.model.drag_drop = Some(config);
        self
    }

    /// Render branch content inside the themed row shell. Wrap interactive
    /// children in `DragDropElementExt::drag_boundary()` to isolate their input.
    pub fn branch_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.branch_content = Some(Rc::new(move |node, window, cx| content(node, window, cx).into_any_element()));
        self
    }

    /// Render leaf content inside the themed row shell.
    pub fn leaf_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.leaf_content = Some(Rc::new(move |node, window, cx| content(node, window, cx).into_any_element()));
        self
    }

    /// Defaults to true for compatibility. False reserves expansion for the
    /// disclosure control and Left/Right keys; clicking a row only selects it.
    pub fn expand_on_row_click(mut self, expand: bool) -> Self {
        self.model.expand_on_row_click = expand;
        self
    }

    /// When true, unfocused wheel input scrolls ancestors. Focused input stays
    /// in the tree even at its endpoints. Defaults to hover scrolling (false).
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.model.require_focus_for_scroll = required;
        self
    }

    /// Whether Up/Down wrap at the first/last eligible row. Defaults to true.
    pub fn wrap_navigation(mut self, wrap: bool) -> Self {
        self.model.wrap_navigation = wrap;
        self
    }

    /// Control scrollbar chrome independently of scrolling. Defaults to AutoHide,
    /// which reveals on scrolling and hides after the toolkit's idle delay.
    /// Hidden reserves no gutter; AlwaysVisible shows only when scrollable.
    pub fn scrollbar_visibility(mut self, visibility: crate::controls::ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    /// Add a vertical SDK scrollbar connected to this tree's list viewport.
    pub fn scrollbar_template(mut self, template: Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = Some(template);
        self
    }

    /// Sets the expanded and collapsed disclosure icons used by branch rows.
    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TreeViewRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_tree_view_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<TreeViewControl<T>> {
        cx.new(|cx| TreeViewControl::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_tree_view_template::<()>();
        let builder = TreeViewBuilder::new("tree-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn duplicate_ids_are_rejected_across_collapsed_branches() {
        let nodes =
            vec![TreeNode::new("a", "A", ()).children([TreeNode::new("b", "B", ())]), TreeNode::new("b", "B", ())];
        assert!(
            matches!(TreeViewBuilder::new("test").try_items(nodes.clone()), Err(TreeViewError::DuplicateId(id)) if id.as_ref() == "b")
        );
        let builder = TreeViewBuilder::new("test").items([TreeNode::new("valid", "Valid", ())]).items(nodes);
        assert_eq!(builder.model.items.len(), 1);
        assert_eq!(builder.model.items[0].id.as_ref(), "valid");
    }
}
