use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::{TextFieldInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, floating_menu_item_disabled_branch,
    floating_menu_item_hover_branch, floating_menu_surface_branch, textfield_and_menu_layout_data,
};

pub(in crate::gallery) fn build_combobox_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    textfield_menu_tree("inspect-combobox", "ComboBox", look)
}

pub(in crate::gallery) fn build_textfield_menu_inspect_tree(
    id: &str,
    title: &str,
    look: &ShadcnLook,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    textfield_menu_tree(id, title, look)
}

fn textfield_menu_tree(id: &str, title: &str, look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let textfield = textfield_branch(id, look);
    let menu = menu_branch(id, look);
    let layout = choice_layout_branch(id, false, look, textfield_and_menu_layout_data);

    vec![TreeNode::new(id.to_owned(), title.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([textfield, menu, layout])]
}

fn textfield_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-textfield");
    let palette = ShadcnInspect::new(look).inspect_textfield_color_palette(ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);
    TreeNode::new(id.clone(), "textfield".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children(textfield_color_nodes(&id, &palette))
}

fn menu_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-menu");
    let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(ControlSize::Md);
    TreeNode::new(id.clone(), "menu".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children([
            floating_menu_surface_branch(&id, &palette, false),
            floating_menu_item_hover_branch(&id, &palette, false),
            floating_menu_item_disabled_branch(&id, &palette, false),
        ])
}

fn textfield_color_nodes(prefix: &str, palette: &TextFieldInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("foreground", Some(&palette.foreground)),
            ("border", Some(&palette.border)),
            ("placeholder", Some(&palette.placeholder)),
            ("icon", Some(&palette.icon)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
