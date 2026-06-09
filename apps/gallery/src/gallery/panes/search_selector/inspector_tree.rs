use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::combobox::build_textfield_menu_inspect_tree;
use crate::gallery::panes::shared::inspector::ColorInspectTreeData;

pub(in crate::gallery) fn build_search_selector_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    build_textfield_menu_inspect_tree("inspect-search-selector", "SearchSelector", look)
}
