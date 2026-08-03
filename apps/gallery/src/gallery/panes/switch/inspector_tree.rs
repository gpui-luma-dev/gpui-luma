use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{SwitchInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, switch_layout_data,
};

pub(in crate::gallery) fn build_switch_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let styles = [(ShadcnButtonStyle::Primary, "Primary"), (ShadcnButtonStyle::Secondary, "Secondary")];
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    styles
        .iter()
        .map(|(style, label)| {
            let style_slug = inspect_slug(label);
            let state_nodes: Vec<_> = matrix_states
                .iter()
                .map(|(state_label, state)| state_branch(&style_slug, label, state_label, *style, *state, look))
                .collect();
            let id = format!("inspect-{style_slug}");
            TreeNode::new(id.clone(), *label, ColorInspectTreeData::Branch)
                .branch(true)
                .expanded(*label == "Primary")
                .children(state_nodes)
        })
        .collect()
}

fn state_branch(
    prefix: &str,
    style_label: &str,
    state_label: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}-{state_label}", inspect_slug(style_label));
    let off_id = format!("{id}-off");
    let on_id = format!("{id}-on");
    let expand = style_label == "Primary" && state_label == "default";

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([
            on_branch(&on_id, style, state, look, expand),
            off_branch(&off_id, style, state, look, expand),
            choice_layout_branch(&id, expand, look, switch_layout_data),
        ])
}

fn off_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_switch_color_palette(style, false, state);
    TreeNode::new(id.to_owned(), "off", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(switch_color_nodes(id, &palette))
}

fn on_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_switch_color_palette(style, true, state);
    TreeNode::new(id.to_owned(), "on", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(switch_color_nodes(id, &palette))
}

fn switch_color_nodes(prefix: &str, palette: &SwitchInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("track background", Some(&palette.track_background)),
            ("track border", Some(&palette.track_border)),
            ("thumb background", Some(&palette.thumb_background)),
            ("thumb border", Some(&palette.thumb_border)),
            ("label", Some(&palette.label_color)),
        ],
    )
}
