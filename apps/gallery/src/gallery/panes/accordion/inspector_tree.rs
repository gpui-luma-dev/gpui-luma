use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{AccordionContentInspectPalette, AccordionTriggerInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, accordion_layout_data,
};

pub(in crate::gallery) fn build_accordion_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![trigger_branch(look), content_branch(look)]
}

fn trigger_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-trigger";
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];
    let state_nodes: Vec<_> = matrix_states
        .into_iter()
        .map(|(state_label, state)| trigger_state_branch(id, state_label, state, look, state_label == "default"))
        .collect();

    TreeNode::new(id.to_owned(), "trigger", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([state_nodes, vec![choice_layout_branch(id, false, look, accordion_layout_data)]].concat())
}

fn content_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-content";
    let expanded = content_state_branch(format!("{id}-expanded"), "expanded", true, look, false);
    let collapsed = content_state_branch(format!("{id}-collapsed"), "collapsed", false, look, false);

    TreeNode::new(id.to_owned(), "content", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children([expanded, collapsed])
}

fn trigger_state_branch(
    prefix: &str,
    state_label: &str,
    state: InteractionState,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let palette = ShadcnInspect::new(look).inspect_accordion_trigger_color_palette(state);

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(trigger_color_nodes(&id, &palette))
}

fn content_state_branch(
    id: String,
    label: &str,
    expanded: bool,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_accordion_content_color_palette(expanded);

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(content_color_nodes(&id, &palette))
}

fn trigger_color_nodes(prefix: &str, palette: &AccordionTriggerInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", palette.background.as_ref()),
            ("foreground", Some(&palette.foreground)),
            ("border", Some(&palette.border_color)),
            ("icon", Some(&palette.icon_color)),
            ("chevron", Some(&palette.chevron_color)),
        ],
    )
}

fn content_color_nodes(prefix: &str, palette: &AccordionContentInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[("background", palette.background.as_ref()), ("foreground", Some(&palette.foreground))],
    )
}
