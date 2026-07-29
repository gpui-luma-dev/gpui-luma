use gpui_luma::controls::toolbar::ToolbarVariant;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::{ShadcnInspect, ToolbarInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, toolbar_layout_data,
};

pub(in crate::gallery) fn build_toolbar_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    vec![shell_branch(look)]
}

fn shell_branch(look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = "inspect-toolbar";
    let outline = variant_branch(id, "outline", ToolbarVariant::Outline, look, true);
    let ghost = variant_branch(id, "ghost", ToolbarVariant::Ghost, look, false);
    let layout = choice_layout_branch(id, false, look, toolbar_layout_data);

    TreeNode::new(id.to_owned(), "toolbar shell", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(true)
        .children([outline, ghost, layout])
}

fn variant_branch(
    prefix: &str,
    label: &str,
    variant: ToolbarVariant,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{label}");
    let enabled = shell_state_branch(format!("{id}-enabled"), "enabled", true, variant, look, expand);
    let disabled = shell_state_branch(format!("{id}-disabled"), "disabled", false, variant, look, false);

    TreeNode::new(id, label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([enabled, disabled])
}

fn shell_state_branch(
    id: String,
    label: &str,
    enabled: bool,
    variant: ToolbarVariant,
    look: &ShadcnLook,
    expand: bool,
) -> TreeNode<ColorInspectTreeData> {
    let palette = ShadcnInspect::new(look).inspect_toolbar_color_palette(enabled, variant);
    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(shell_color_nodes(&id, &palette))
}

fn shell_color_nodes(prefix: &str, palette: &ToolbarInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("border", Some(&palette.border)),
            ("separator", Some(&palette.separator)),
        ],
    )
}
