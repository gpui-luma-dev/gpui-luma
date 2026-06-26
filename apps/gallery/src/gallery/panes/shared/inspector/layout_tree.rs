use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use super::metrics::toggle_layout_data;
use super::types::{ColorInspectTreeData, InspectLayoutSizeData, inspect_slug};

pub(in crate::gallery) fn sized_layout_branch(
    prefix: &str,
    label: &str,
    expand: bool,
    look: &ShadcnLook,
    build: fn(&ShadcnLook, ControlSize) -> InspectLayoutSizeData,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}", inspect_slug(label));
    let sizes = [(ControlSize::Sm, "sm"), (ControlSize::Md, "md"), (ControlSize::Lg, "lg")];
    let children: Vec<_> = sizes
        .into_iter()
        .map(|(size, size_label)| {
            let node_id = format!("{id}-{size_label}");
            TreeNode::new(node_id, size_label.to_owned(), ColorInspectTreeData::LayoutSize(build(look, size)))
        })
        .collect();

    TreeNode::new(id.clone(), label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

pub(in crate::gallery) fn fixed_layout_branch(
    prefix: &str,
    expand: bool,
    properties: Vec<super::types::InspectMetricPropertyData>,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-layout");
    TreeNode::new(id.clone(), "layout", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children([TreeNode::new(
            format!("{id}-metrics"),
            "metrics",
            ColorInspectTreeData::LayoutSize(super::types::InspectLayoutSizeData {
                size: ControlSize::Md,
                properties,
                box_model: None,
            }),
        )])
}

pub(in crate::gallery) fn choice_layout_branch(
    prefix: &str,
    expand: bool,
    look: &ShadcnLook,
    build: fn(&ShadcnLook, ControlSize) -> InspectLayoutSizeData,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-layout");
    let sizes = [(ControlSize::Sm, "sm"), (ControlSize::Md, "md"), (ControlSize::Lg, "lg")];
    let children: Vec<_> = sizes
        .into_iter()
        .map(|(size, label)| {
            let node_id = format!("{id}-{label}");
            TreeNode::new(node_id, label.to_owned(), ColorInspectTreeData::LayoutSize(build(look, size)))
        })
        .collect();

    TreeNode::new(id.clone(), "layout", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

pub(in crate::gallery) fn toggle_layout_branch(
    prefix: &str,
    expand: bool,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-layout");
    let sizes = [(ControlSize::Sm, "sm"), (ControlSize::Md, "md"), (ControlSize::Lg, "lg")];
    let children: Vec<_> = sizes
        .into_iter()
        .map(|(size, label)| {
            let node_id = format!("{id}-{label}");
            TreeNode::new(
                node_id,
                label.to_owned(),
                ColorInspectTreeData::LayoutSize(toggle_layout_data(look, style, size)),
            )
        })
        .collect();

    TreeNode::new(id.clone(), "layout", ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}
