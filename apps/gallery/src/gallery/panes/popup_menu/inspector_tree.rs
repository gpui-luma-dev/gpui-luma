use gpui_luma::controls::popup_menu::PopupMenuTriggerStyle;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, floating_menu_item_disabled_branch, floating_menu_item_hover_branch,
    floating_menu_layout_data, floating_menu_surface_branch, ghost_trigger_color_nodes, inspect_slug,
    popup_menu_ghost_trigger_layout_data, popup_menu_outline_trigger_layout_data, sized_layout_branch,
};

pub(in crate::gallery) fn build_popup_menu_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    [PopupMenuTriggerStyle::Outline, PopupMenuTriggerStyle::Ghost]
        .into_iter()
        .map(|trigger_style| trigger_style_branch(trigger_style, look))
        .collect()
}

fn trigger_style_branch(trigger_style: PopupMenuTriggerStyle, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let style_label = match trigger_style {
        PopupMenuTriggerStyle::Outline => "outline trigger",
        PopupMenuTriggerStyle::Ghost => "ghost trigger",
    };
    let id = format!("inspect-{}", inspect_slug(style_label));
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    TreeNode::new(id.clone(), style_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(trigger_style == PopupMenuTriggerStyle::Outline)
        .children(
            matrix_states
                .iter()
                .map(|(state_label, state)| state_branch(&id, state_label, trigger_style, *state, look))
                .collect::<Vec<_>>(),
        )
}

fn state_branch(
    prefix: &str,
    state_label: &str,
    trigger_style: PopupMenuTriggerStyle,
    state: InteractionState,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(state_label));
    let expand = state_label == "default";
    let palette = ShadcnInspect::new(look).inspect_popup_menu_color_palette(trigger_style, state, ControlSize::Md);
    let mut children = ghost_trigger_color_nodes(
        &id,
        &palette.trigger_background,
        &palette.trigger_foreground,
        &palette.trigger_border,
    );
    children.push(floating_menu_surface_branch(&id, &palette.menu, expand));
    children.push(floating_menu_item_hover_branch(&id, &palette.menu, expand));
    children.push(floating_menu_item_disabled_branch(&id, &palette.menu, expand));
    children.push(popup_menu_layout_branch(&id, trigger_style, expand, look));

    TreeNode::new(id.clone(), state_label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn popup_menu_layout_branch(
    prefix: &str,
    trigger_style: PopupMenuTriggerStyle,
    expand: bool,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-layout");
    let trigger_layout = match trigger_style {
        PopupMenuTriggerStyle::Outline => popup_menu_outline_trigger_layout_data,
        PopupMenuTriggerStyle::Ghost => popup_menu_ghost_trigger_layout_data,
    };

    TreeNode::new(id.clone(), "layout", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([
            sized_layout_branch(&id, "trigger", expand, look, trigger_layout),
            sized_layout_branch(&id, "menu", expand, look, floating_menu_layout_data),
        ])
}
