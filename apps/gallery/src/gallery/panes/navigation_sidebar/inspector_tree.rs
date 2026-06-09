use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{NavigationSidebarContainerInspectPalette, NavigationSidebarItemInspectPalette, NavigationSidebarSectionInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug,
    navigation_sidebar_layout_data,
};

pub(in crate::gallery) fn build_navigation_sidebar_inspect_tree(
    look: &ShadcnLook,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![
        container_branch(look),
        section_branch(look),
        branch_item_branch(look),
        nav_item_branch(look),
    ]
}

fn container_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-container";
    let palette = ShadcnInspect::new(look).inspect_navigation_sidebar_container_color_palette();
    TreeNode::new(id.to_owned(), "container".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children(container_color_nodes(id, &palette))
}

fn section_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-section";
    let palette = ShadcnInspect::new(look).inspect_navigation_sidebar_section_color_palette();
    TreeNode::new(id.to_owned(), "section".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(section_color_nodes(id, &palette))
}

fn branch_item_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-branch";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| {
            item_state_branch(id, state_label, state, look, false, state_label == "default")
        })
        .collect();

    TreeNode::new(id.to_owned(), "branch item".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn nav_item_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-item";
    let selected = selected_item_branch(id, look);
    let unselected = unselected_item_branch(id, look);
    let layout = choice_layout_branch(id, false, look, navigation_sidebar_layout_data);

    TreeNode::new(id.to_owned(), "nav item".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children([selected, unselected, layout])
}

fn selected_item_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-selected");
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| item_state_branch(&id, state_label, state, look, true, state_label == "default"))
        .collect();

    TreeNode::new(id.clone(), "selected".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn unselected_item_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-unselected");
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| item_state_branch(&id, state_label, state, look, false, false))
        .collect();

    TreeNode::new(id.clone(), "unselected".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(state_nodes)
}

fn item_state_branch(
    prefix: &str,
    state_label: &str,
    state: InteractionState,
    look: &ShadcnLook,
    selected: bool,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = if prefix.contains("branch") {
        ShadcnInspect::new(look).inspect_navigation_sidebar_branch_color_palette(state)
    } else {
        ShadcnInspect::new(look).inspect_navigation_sidebar_item_color_palette(selected, state)
    };

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(item_color_nodes(&id, &palette))
}

fn container_color_nodes(
    prefix: &str,
    palette: &NavigationSidebarContainerInspectPalette,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("foreground", Some(&palette.foreground)),
            ("border", Some(&palette.border)),
        ],
    )
}

fn section_color_nodes(
    prefix: &str,
    palette: &NavigationSidebarSectionInspectPalette,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(prefix, &[("label", Some(&palette.label_color))])
}

fn item_color_nodes(prefix: &str, palette: &NavigationSidebarItemInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", palette.background.as_ref()),
            ("foreground", Some(&palette.foreground)),
            ("icon", Some(&palette.icon_color)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
