use gpui_luma::controls::scrollbar::ScrollbarOrientation;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ScrollbarInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, fixed_layout_branch, inspect_slug, scrollbar_layout_data,
};

pub(in crate::gallery) fn build_scrollbar_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    [(ScrollbarOrientation::Horizontal, "Horizontal"), (ScrollbarOrientation::Vertical, "Vertical")]
        .into_iter()
        .map(|(orientation, label)| orientation_branch(orientation, label, look))
        .collect()
}

fn orientation_branch(
    orientation: ScrollbarOrientation,
    label: &'static str,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let slug = inspect_slug(label);
    let id = format!("inspect-{slug}");
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let expand = label == "Horizontal";
    let state_nodes: Vec<_> = matrix_states
        .iter()
        .map(|(state_label, state)| state_branch(&id, label, state_label, orientation, *state, look, expand))
        .collect();

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(state_nodes)
}

fn state_branch(
    prefix: &str,
    orientation_label: &str,
    state_label: &str,
    orientation: ScrollbarOrientation,
    state: InteractionState,
    look: &ShadcnLook,
    expand_orientation: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let expand = expand_orientation && orientation_label == "Horizontal" && state_label == "default";
    let palette = ShadcnInspect::new(look).inspect_scrollbar_color_palette(orientation, state);
    let mut children = scrollbar_color_nodes(&id, &palette);
    children.push(fixed_layout_branch(&id, expand, scrollbar_layout_data(look, orientation).properties));

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn scrollbar_color_nodes(prefix: &str, palette: &ScrollbarInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("track background", Some(&palette.track_background)),
            ("thumb background", Some(&palette.thumb_background)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
