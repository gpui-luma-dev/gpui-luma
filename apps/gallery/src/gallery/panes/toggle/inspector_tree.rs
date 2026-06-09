use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ButtonInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, inspect_slug, toggle_layout_branch,
};

pub(in crate::gallery) fn build_toggle_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let styles = [(ShadcnButtonStyle::Primary, "Primary"), (ShadcnButtonStyle::Secondary, "Secondary")];
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
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
    let unselected_id = format!("{id}-unselected");
    let selected_id = format!("{id}-selected");
    let expand = style_label == "Primary" && state_label == "default";

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
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        style,
        ButtonFamilyRole::Toggle { selected: false },
        state,
    );
    TreeNode::new(id.to_owned(), "unselected", ColorInspectTreeData::Branch)
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
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        style,
        ButtonFamilyRole::Toggle { selected: true },
        state,
    );
    TreeNode::new(id.to_owned(), "selected", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(toggle_color_nodes(id, &palette))
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
