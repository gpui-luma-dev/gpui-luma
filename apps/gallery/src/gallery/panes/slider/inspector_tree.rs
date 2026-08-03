use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{SliderInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, fixed_layout_branch, inspect_slug, slider_layout_data,
};

pub(in crate::gallery) fn build_slider_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    matrix_states.iter().map(|(state_label, state)| state_branch(state_label, *state, look)).collect()
}

fn state_branch(state_label: &str, state: InteractionState, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-{}", inspect_slug(state_label));
    let expand = state_label == "default";
    let palette = ShadcnInspect::new(look).inspect_slider_color_palette(state);
    let mut children = slider_color_nodes(&id, &palette);
    children.push(fixed_layout_branch(&id, expand, slider_layout_data(look).properties));

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn slider_color_nodes(prefix: &str, palette: &SliderInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("track background", Some(&palette.track_background)),
            ("fill background", Some(&palette.fill_background)),
            ("thumb background", Some(&palette.thumb_background)),
            ("thumb border", Some(&palette.thumb_border)),
        ],
    )
}
