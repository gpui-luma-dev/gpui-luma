use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::panes::shared::inspector::{ColorInspectTreeData, card_layout_data, choice_layout_branch};

pub(in crate::gallery) fn build_card_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let id = "inspect-surface";
    let children = vec![choice_layout_branch(id, true, look, card_layout_data)];

    vec![
        TreeNode::new(id.to_owned(), "surface", ColorInspectTreeData::Branch)
            .branch(true)
            .expanded(true)
            .children(children),
    ]
}
