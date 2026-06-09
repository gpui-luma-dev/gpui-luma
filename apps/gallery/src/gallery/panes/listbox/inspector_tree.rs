use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ListBoxListInspectPalette, ListBoxRowInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, listbox_layout_data,
};

pub(in crate::gallery) fn build_listbox_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![list_branch(look), row_branch(look)]
}

fn list_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-list";
    let enabled = list_state_branch(format!("{id}-enabled"), "enabled", true, false, look, true);
    let disabled = list_state_branch(format!("{id}-disabled"), "disabled", false, false, look, false);
    let focused = list_state_branch(format!("{id}-focused"), "focused", true, true, look, false);

    TreeNode::new(id.to_owned(), "list", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([enabled, disabled, focused, choice_layout_branch(id, false, look, listbox_layout_data)])
}

fn list_state_branch(
    id: String,
    label: &str,
    enabled: bool,
    focused: bool,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_listbox_list_color_palette(enabled, focused);
    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(list_color_nodes(&id, &palette))
}

fn row_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-row";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("keyboard active", InteractionState { focused: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| row_state_branch(id, state_label, state, look, state_label == "default"))
        .collect();

    TreeNode::new(id.to_owned(), "row", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn row_state_branch(
    prefix: &str,
    state_label: &str,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_listbox_row_color_palette(state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(row_color_nodes(&id, &palette))
}

fn list_color_nodes(prefix: &str, palette: &ListBoxListInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("border", Some(&palette.border)),
            ("divider", Some(&palette.divider)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}

fn row_color_nodes(prefix: &str, palette: &ListBoxRowInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[("background", Some(&palette.background)), ("label", Some(&palette.label_color))],
    )
}
