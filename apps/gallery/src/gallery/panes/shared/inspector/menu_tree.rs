use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn_inspect::FloatingMenuInspectPalette;

use super::types::ColorInspectTreeData;
use super::tree::color_field_nodes_optional;

pub(in crate::gallery) fn floating_menu_surface_branch(
    prefix: &str,
    palette: &FloatingMenuInspectPalette,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-menu-surface");
    TreeNode::new(id.clone(), "menu surface", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(color_field_nodes_optional(
            &id,
            &[
                ("background", Some(&palette.background)),
                ("foreground", Some(&palette.foreground)),
                ("border", Some(&palette.border)),
            ],
        ))
}

pub(in crate::gallery) fn floating_menu_item_hover_branch(
    prefix: &str,
    palette: &FloatingMenuInspectPalette,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-menu-item-hover");
    TreeNode::new(id.clone(), "item hover", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(color_field_nodes_optional(
            &id,
            &[
                ("item hover background", Some(&palette.item_hover_background)),
                ("item hover foreground", Some(&palette.item_hover_foreground)),
            ],
        ))
}

pub(in crate::gallery) fn floating_menu_item_disabled_branch(
    prefix: &str,
    palette: &FloatingMenuInspectPalette,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-menu-item-disabled");
    TreeNode::new(id.clone(), "item disabled", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(color_field_nodes_optional(
            &id,
            &[("item disabled foreground", Some(&palette.item_disabled_foreground))],
        ))
}

pub(in crate::gallery) fn ghost_trigger_color_nodes(
    prefix: &str,
    background: &gpui_luma_look_shadcn_inspect::ResolvedColor,
    foreground: &gpui_luma_look_shadcn_inspect::ResolvedColor,
    border: &gpui_luma_look_shadcn_inspect::ResolvedColor,
) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("trigger background", Some(background)),
            ("trigger foreground", Some(foreground)),
            ("trigger border", Some(border)),
        ],
    )
}
