use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ListViewInspectPalette, ListViewRowInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, list_view_layout_data,
};

pub(in crate::gallery) fn build_list_view_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![surface_branch(look), row_branch(look)]
}

fn surface_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-surface";
    let enabled = surface_state_branch(format!("{id}-enabled"), "enabled", true, look, true);
    let disabled = surface_state_branch(format!("{id}-disabled"), "disabled", false, look, false);

    TreeNode::new(id.to_owned(), "surface", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([enabled, disabled, choice_layout_branch(id, false, look, list_view_layout_data)])
}

fn surface_state_branch(
    id: String,
    label: &str,
    enabled: bool,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_list_view_color_palette(enabled);
    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(list_view_surface_color_nodes(&id, &palette))
}

fn row_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-row";
    let matrix_states = [
        ("default", InteractionState::default(), false),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }, false),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }, false),
        ("selected", InteractionState::default(), true),
        ("keyboard active", InteractionState { focused: true, ..InteractionState::default() }, false),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }, false),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state, selected)| {
            row_state_branch(id, state_label, selected, state, look, state_label == "default")
        })
        .collect();

    TreeNode::new(id.to_owned(), "row", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn row_state_branch(
    prefix: &str,
    state_label: &str,
    selected: bool,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_list_view_row_color_palette(selected, state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(list_view_row_color_nodes(&id, &palette))
}

fn list_view_surface_color_nodes(
    prefix: &str,
    palette: &ListViewInspectPalette,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("border", Some(&palette.border)),
            ("header background", Some(&palette.header_background)),
            ("header label", Some(&palette.header_label_color)),
        ],
    )
}

fn list_view_row_color_nodes(prefix: &str, palette: &ListViewRowInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("label", Some(&palette.label_color)),
            ("divider", Some(&palette.divider)),
        ],
    )
}
