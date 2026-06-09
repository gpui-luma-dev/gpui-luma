use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ButtonInspectPalette, ControlGroupListInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, control_group_layout_data, inspect_slug,
    toggle_layout_branch,
};

pub(in crate::gallery) fn build_toggle_group_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![list_branch(look), item_branch(look)]
}

fn list_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-list";
    let enabled = list_state_branch(format!("{id}-enabled"), "enabled", true, true, look);
    let disabled = list_state_branch(format!("{id}-disabled"), "disabled", false, false, look);
    let layout = choice_layout_branch(id, false, look, control_group_layout_data);

    TreeNode::new(id.to_owned(), "list".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([enabled, disabled, layout])
}

fn list_state_branch(
    id: String,
    label: &str,
    enabled: bool,
    expand: bool,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_control_group_list_color_palette(enabled);
    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(list_color_nodes(&id, &palette))
}

fn item_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let style = ShadcnButtonStyle::Ghost;
    let id = "inspect-ghost";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .iter()
        .map(|(state_label, state)| state_branch(id, state_label, style, *state, look))
        .collect();

    TreeNode::new(id.to_owned(), "Ghost item".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn state_branch(
    prefix: &str,
    state_label: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let unselected_id = format!("{id}-unselected");
    let selected_id = format!("{id}-selected");
    let expand = state_label == "default";

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([
            selected_branch(&selected_id, style, state, look, expand),
            unselected_branch(&unselected_id, style, state, look, expand),
            toggle_layout_branch(&id, expand, look, style),
        ])
}

fn unselected_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(style, ButtonFamilyRole::Toggle { selected: false }, state);
    TreeNode::new(id.to_owned(), "unselected".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(toggle_color_nodes(id, &palette))
}

fn selected_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(style, ButtonFamilyRole::Toggle { selected: true }, state);
    TreeNode::new(id.to_owned(), "selected".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(toggle_color_nodes(id, &palette))
}

fn list_color_nodes(prefix: &str, palette: &ControlGroupListInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("border", Some(&palette.border)),
        ],
    )
}

fn toggle_color_nodes(prefix: &str, palette: &ButtonInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("foreground", Some(&palette.foreground)),
            ("border", Some(&palette.border)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
