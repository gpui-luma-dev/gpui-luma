use gpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::{ScrollbarInspectPalette, ShadcnInspect};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, fixed_layout_branch, inspect_slug, scrollbar_layout_data,
};

pub(in crate::gallery) fn build_scrollbar_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let id = "inspect-scrollbar";
    let ghost = style_branch(id, "ghost", ScrollbarStyle::Ghost, look, true);
    let soft = style_branch(id, "soft", ScrollbarStyle::Soft, look, false);
    let layout = fixed_layout_branch(
        id,
        false,
        scrollbar_layout_data(look, ScrollbarOrientation::Vertical, ScrollbarStyle::Ghost).properties,
    );

    vec![
        TreeNode::new(id.to_owned(), "scrollbar", ColorInspectTreeData::Branch)
            .branch(true)
            .expanded(true)
            .children([ghost, soft, layout]),
    ]
}

fn style_branch(
    prefix: &str,
    label: &str,
    style: ScrollbarStyle,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{label}");
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .iter()
        .map(|(state_label, state)| {
            state_branch(&id, state_label, style, *state, look, expand && *state_label == "default")
        })
        .collect();

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(state_nodes)
}

fn state_branch(
    prefix: &str,
    state_label: &str,
    style: ScrollbarStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_scrollbar_color_palette(style, state);
    let mut children = scrollbar_color_nodes(&id, &palette);
    children.push(fixed_layout_branch(
        &id,
        expand,
        scrollbar_layout_data(look, ScrollbarOrientation::Vertical, style).properties,
    ));

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
