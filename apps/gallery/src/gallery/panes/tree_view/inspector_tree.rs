use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{TreeViewRowInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, tree_view_layout_data,
};

pub(in crate::gallery) fn build_tree_view_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![row_branch(look)]
}

fn row_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-row";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| row_state_branch(id, state_label, state, look, state_label == "default"))
        .collect();

    TreeNode::new(id.to_owned(), "row", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([state_nodes, vec![choice_layout_branch(id, false, look, tree_view_layout_data)]].concat())
}

fn row_state_branch(
    prefix: &str,
    state_label: &str,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_tree_view_row_color_palette(state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(row_color_nodes(&id, &palette))
}

fn row_color_nodes(prefix: &str, palette: &TreeViewRowInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", palette.background.as_ref()),
            ("foreground", Some(&palette.foreground)),
            ("icon", Some(&palette.icon_color)),
            ("chevron", Some(&palette.chevron_color)),
        ],
    )
}
