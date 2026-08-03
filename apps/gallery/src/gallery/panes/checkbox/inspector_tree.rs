use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{CheckboxInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, checkbox_layout_data, choice_layout_branch, color_field_nodes_optional, inspect_slug,
};

pub(in crate::gallery) fn build_checkbox_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
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
    let unchecked_id = format!("{id}-unchecked");
    let checked_id = format!("{id}-checked");
    let expand = style_label == "Primary" && state_label == "default";

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([
            checked_branch(&checked_id, style, state, true, look, expand),
            unchecked_branch(&unchecked_id, style, state, look, expand),
            choice_layout_branch(&id, expand, look, checkbox_layout_data),
        ])
}

fn unchecked_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_checkbox_color_palette(style, false, state);
    TreeNode::new(id.to_owned(), "unchecked", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(checkbox_color_nodes(id, &palette))
}

fn checked_branch(
    id: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    checked: bool,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_checkbox_color_palette(style, checked, state);
    TreeNode::new(id.to_owned(), "checked", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(checkbox_color_nodes(id, &palette))
}

fn checkbox_color_nodes(prefix: &str, palette: &CheckboxInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("indicator background", Some(&palette.indicator_background)),
            ("indicator border", Some(&palette.indicator_border)),
            ("checkmark", Some(&palette.checkmark_color)),
            ("label", Some(&palette.label_color)),
        ],
    )
}
