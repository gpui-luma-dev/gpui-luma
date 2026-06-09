use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, floating_menu_item_disabled_branch, floating_menu_item_hover_branch,
    floating_menu_layout_data, floating_menu_surface_branch, ghost_trigger_color_nodes, inspect_slug,
    popup_menu_trigger_layout_data, sized_layout_branch,
};

pub(in crate::gallery) fn build_selector_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    matrix_states.iter().map(|(state_label, state)| state_branch(state_label, *state, look)).collect()
}

fn state_branch(state_label: &str, state: InteractionState, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-{}", inspect_slug(state_label));
    let expand = state_label == "default";
    let palette = ShadcnInspect::new(look).inspect_selector_color_palette(state, ControlSize::Md);
    let mut children = ghost_trigger_color_nodes(
        &id,
        &palette.trigger_background,
        &palette.trigger_foreground,
        &palette.trigger_border,
        palette.focus_ring.as_ref(),
    );
    children.push(floating_menu_surface_branch(&id, &palette.items_panel, expand));
    children.push(floating_menu_item_hover_branch(&id, &palette.items_panel, expand));
    children.push(floating_menu_item_disabled_branch(&id, &palette.items_panel, expand));
    children.push(selector_layout_branch(&id, expand, look));

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn selector_layout_branch(prefix: &str, expand: bool, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-layout");
    TreeNode::new(id.clone(), "layout".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([
            sized_layout_branch(&id, "trigger", expand, look, popup_menu_trigger_layout_data),
            sized_layout_branch(&id, "items panel", expand, look, floating_menu_layout_data),
        ])
}
