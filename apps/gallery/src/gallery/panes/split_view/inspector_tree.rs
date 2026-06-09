use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{SplitViewInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, fixed_layout_branch, inspect_slug, split_view_layout_data,
};

pub(in crate::gallery) fn build_split_view_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    [("enabled", true), ("disabled", false)]
        .into_iter()
        .map(|(label, enabled)| enabled_branch(label, enabled, look))
        .chain([fixed_layout_branch("inspect", false, split_view_layout_data(look))])
        .collect()
}

fn enabled_branch(label: &str, enabled: bool, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-{}", inspect_slug(label));
    let palette = ShadcnInspect::new(look).inspect_split_view_color_palette(enabled);

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(enabled)
        .children(split_view_color_nodes(&id, &palette))
}

fn split_view_color_nodes(prefix: &str, palette: &SplitViewInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[("separator", Some(&palette.separator)), ("separator hover", Some(&palette.separator_hover))],
    )
}
