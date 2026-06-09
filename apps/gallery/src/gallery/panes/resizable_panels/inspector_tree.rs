use gpui_luma::controls::resizable_panels::ResizeHandleSize;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ResizablePanelsInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, inspect_slug, resizable_panels_layout_data,
};

pub(in crate::gallery) fn build_resizable_panels_inspect_tree(
    look: &ShadcnLook,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![appearance_branch(look), handle_layout_branch(look)]
}

fn appearance_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-appearance";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| state_branch(id, state_label, state, look, state_label == "default"))
        .collect();

    TreeNode::new(id.to_owned(), "appearance", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children(state_nodes)
}

fn handle_layout_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-handle-layout";
    let sizes = [(ResizeHandleSize::Sm, "sm"), (ResizeHandleSize::Md, "md"), (ResizeHandleSize::Lg, "lg")];
    let children: Vec<_> = sizes
        .into_iter()
        .map(|(size, label)| {
            let node_id = format!("{id}-{}", inspect_slug(label));
            TreeNode::new(
                node_id,
                label.to_owned(),
                ColorInspectTreeData::LayoutSize(resizable_panels_layout_data(look, size)),
            )
        })
        .collect();

    TreeNode::new(id.to_owned(), "handle layout", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children(children)
}

fn state_branch(
    prefix: &str,
    state_label: &str,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_resizable_panels_color_palette(state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(appearance_color_nodes(&id, &palette))
}

fn appearance_color_nodes(
    prefix: &str,
    palette: &ResizablePanelsInspectPalette,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("border", Some(&palette.border)),
            ("divider", Some(&palette.divider)),
            ("grip", Some(&palette.grip)),
            ("grip emphasis", Some(&palette.grip_emphasis)),
        ],
    )
}
