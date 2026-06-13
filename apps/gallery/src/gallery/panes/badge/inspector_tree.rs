use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn::{BadgeVariant, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{BadgeInspectPalette, ShadcnInspect};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, color_field_nodes_optional, inspect_slug, sized_layout_branch,
};

use super::super::shared::inspector::badge_layout_data;

pub(in crate::gallery) fn build_badge_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let variants = [
        (BadgeVariant::Default, "Default"),
        (BadgeVariant::Secondary, "Secondary"),
        (BadgeVariant::Outline, "Outline"),
        (BadgeVariant::Ghost, "Ghost"),
    ];

    let mut nodes: Vec<_> = variants.into_iter().map(|(variant, label)| variant_branch(variant, label, look)).collect();
    nodes.push(sized_layout_branch("inspect-badge", "layout", true, look, badge_layout_data));
    nodes
}

fn variant_branch(variant: BadgeVariant, label: &'static str, look: &ShadcnLook) -> TreeNode<ColorInspectTreeData> {
    let id = format!("inspect-badge-{}", inspect_slug(label));
    let palette = ShadcnInspect::new(look).inspect_badge_color_palette(variant);

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(matches!(variant, BadgeVariant::Default))
        .children(badge_color_nodes(&id, &palette))
}

fn badge_color_nodes(prefix: &str, palette: &BadgeInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("foreground", Some(&palette.foreground)),
            ("border", palette.border.as_ref()),
        ],
    )
}
