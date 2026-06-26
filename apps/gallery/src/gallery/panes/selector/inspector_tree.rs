use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use crate::gallery::panes::combobox::textfield_color_nodes;
use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, floating_menu_item_disabled_branch, floating_menu_item_hover_branch,
    floating_menu_layout_data, floating_menu_surface_branch, inspect_slug, popup_menu_outline_trigger_layout_data,
    sized_layout_branch,
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

fn selector_textfield_state(state: InteractionState) -> TextFieldState {
    TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focused,
        ..TextFieldState::default()
    }
}

fn state_branch(state_label: &str, state: InteractionState, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-{}", inspect_slug(state_label));
    let expand = state_label == "default";
    let inspect = ShadcnInspect::new(look);
    let trigger_palette = inspect.inspect_textfield_color_palette(
        ShadcnTextFieldStyle::Input,
        selector_textfield_state(state),
        !state.disabled,
    );
    let items_panel = inspect.inspect_floating_menu_color_palette(ControlSize::Md);
    let mut children = textfield_color_nodes(&format!("{id}-trigger"), &trigger_palette);
    children.push(floating_menu_surface_branch(&id, &items_panel, expand));
    children.push(floating_menu_item_hover_branch(&id, &items_panel, expand));
    children.push(floating_menu_item_disabled_branch(&id, &items_panel, expand));
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
            sized_layout_branch(&id, "trigger", expand, look, popup_menu_outline_trigger_layout_data),
            sized_layout_branch(&id, "items panel", expand, look, floating_menu_layout_data),
        ])
}
