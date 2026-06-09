use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ProgressInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, fixed_layout_branch, inspect_slug, progress_layout_data,
};

pub(in crate::gallery) fn build_progress_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    [("enabled", true), ("disabled", false)]
        .into_iter()
        .map(|(label, enabled)| enabled_branch(label, enabled, look))
        .collect()
}

fn enabled_branch(label: &str, enabled: bool, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-{}", inspect_slug(label));
    let expand = enabled;
    let palette = ShadcnInspect::new(look).inspect_progress_color_palette(enabled);
    let mut children = progress_color_nodes(&id, &palette);
    children.push(fixed_layout_branch(&id, expand, progress_layout_data(look).properties));

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn progress_color_nodes(prefix: &str, palette: &ProgressInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[("track", Some(&palette.track_color)), ("progress", Some(&palette.progress_color))],
    )
}
