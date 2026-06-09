use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, floating_menu_item_disabled_branch, floating_menu_item_hover_branch,
    floating_menu_layout_data, floating_menu_surface_branch,
};

pub(in crate::gallery) fn build_floating_menu_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(ControlSize::Md);
    let id = "inspect-floating-menu";

    vec![
        TreeNode::new(id.to_owned(), "Floating Menu".to_owned(), ColorInspectTreeData::Branch)
            .branch(true)
            .expanded(true)
            .children([
                floating_menu_surface_branch(id, &palette, true),
                floating_menu_item_hover_branch(id, &palette, false),
                floating_menu_item_disabled_branch(id, &palette, false),
                choice_layout_branch(id, true, look, floating_menu_layout_data),
            ]),
    ]
}
