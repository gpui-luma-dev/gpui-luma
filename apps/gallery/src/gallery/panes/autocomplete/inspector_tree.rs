use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{AutocompleteChromeInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, floating_menu_item_disabled_branch,
    floating_menu_item_hover_branch, floating_menu_surface_branch, autocomplete_layout_data,
};

pub(in crate::gallery) fn build_autocomplete_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let id = "inspect-autocomplete";
    let chrome = chrome_branch(id, look);
    let menu = menu_branch(id, look);
    let layout = choice_layout_branch(id, false, look, autocomplete_layout_data);

    vec![TreeNode::new(id.to_owned(), "Autocomplete".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([chrome, menu, layout])]
}

fn chrome_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-chrome");
    let palette = ShadcnInspect::new(look).inspect_autocomplete_chrome_color_palette();
    TreeNode::new(id.clone(), "chrome".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children(chrome_color_nodes(&id, &palette))
}

fn menu_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-menu");
    let palette = ShadcnInspect::new(look).inspect_autocomplete_menu_color_palette(ControlSize::Md);
    TreeNode::new(id.clone(), "menu".to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(false)
        .children([
            floating_menu_surface_branch(&id, &palette, false),
            floating_menu_item_hover_branch(&id, &palette, false),
            floating_menu_item_disabled_branch(&id, &palette, false),
        ])
}

fn chrome_color_nodes(prefix: &str, palette: &AutocompleteChromeInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("status", Some(&palette.status_color)),
            ("muted text", Some(&palette.muted_text_color)),
            ("clear icon", Some(&palette.clear_icon_color)),
            ("clear icon hover", Some(&palette.clear_icon_hover_color)),
        ],
    )
}
