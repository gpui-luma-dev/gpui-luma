use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{TabsNavigationItemInspectPalette, TabsNavigationListInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, tabs_navigation_layout_data,
};

pub(in crate::gallery) fn build_tabs_navigation_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    vec![inactive_branch(&matrix_states, look), active_branch(&matrix_states, look), list_branch(look)]
}

fn inactive_branch(matrix_states: &[(&str, InteractionState)], look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-inactive";
    let state_nodes: Vec<_> = matrix_states
        .iter()
        .map(|(state_label, state)| item_state_branch(id, state_label, false, *state, look, *state_label == "default"))
        .collect();

    TreeNode::new(id.to_owned(), "inactive", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children(state_nodes)
}

fn active_branch(matrix_states: &[(&str, InteractionState)], look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-active";
    let state_nodes: Vec<_> = matrix_states
        .iter()
        .map(|(state_label, state)| item_state_branch(id, state_label, true, *state, look, false))
        .collect();

    TreeNode::new(id.to_owned(), "active", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn list_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let enabled_palette = ShadcnInspect::new(look).inspect_tabs_navigation_list_color_palette(true);
    let disabled_palette = ShadcnInspect::new(look).inspect_tabs_navigation_list_color_palette(false);

    TreeNode::new("inspect-list".to_owned(), "list", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children([
            list_state_branch("inspect-list-enabled", "enabled", &enabled_palette, false),
            list_state_branch("inspect-list-disabled", "disabled", &disabled_palette, false),
            choice_layout_branch("inspect-list", false, look, tabs_navigation_layout_data),
        ])
}

fn item_state_branch(
    prefix: &str,
    state_label: &str,
    active: bool,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_tabs_navigation_item_color_palette(active, state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(item_color_nodes(&id, &palette))
}

fn list_state_branch(
    id: &str,
    label: &str,
    palette: &TabsNavigationListInspectPalette,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    TreeNode::new(id.to_owned(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(color_field_nodes_optional(id, &[("list background", palette.background.as_ref())]))
}

fn item_color_nodes(prefix: &str, palette: &TabsNavigationItemInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[("label", Some(&palette.label_color)), ("indicator", palette.indicator.as_ref())],
    )
}
