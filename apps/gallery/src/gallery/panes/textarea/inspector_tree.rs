use gpui_luma::controls::textarea::TextAreaState;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::{TextFieldInspectPalette};

use crate::gallery::panes::shared::inspector::{
    ColorInspectTreeData, choice_layout_branch, color_field_nodes_optional, inspect_slug, textarea_layout_data,
};

struct TextAreaMatrixState {
    label: &'static str,
    state: TextAreaState,
    enabled: bool,
}

pub(in crate::gallery) fn build_textarea_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let variants = [
        (ShadcnTextFieldStyle::Surface, "Surface"),
        (ShadcnTextFieldStyle::Filled, "Filled"),
        (ShadcnTextFieldStyle::Soft, "Soft"),
    ];
    let matrix_states = [
        TextAreaMatrixState { label: "default", state: TextAreaState::default(), enabled: true },
        TextAreaMatrixState {
            label: "hover",
            state: TextAreaState { hovered: true, ..TextAreaState::default() },
            enabled: true,
        },
        TextAreaMatrixState {
            label: "focus",
            state: TextAreaState { focused: true, focus_visible: true, ..TextAreaState::default() },
            enabled: true,
        },
        TextAreaMatrixState { label: "disabled", state: TextAreaState::default(), enabled: false },
    ];

    variants
        .iter()
        .map(|(style, label)| {
            let style_slug = inspect_slug(label);
            let state_nodes: Vec<_> =
                matrix_states.iter().map(|sample| state_branch(&style_slug, label, sample, *style, look)).collect();
            let id = format!("inspect-{style_slug}");
            TreeNode::new(id.clone(), *label, ColorInspectTreeData::Branch)
                .branch(true)
                .expanded(*label == "Surface")
                .children(state_nodes)
        })
        .collect()
}

fn state_branch(
    prefix: &str,
    style_label: &str,
    sample: &TextAreaMatrixState,
    style: ShadcnTextFieldStyle,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}-{}", inspect_slug(style_label), sample.label);
    let expand = style_label == "Surface" && sample.label == "default";
    let palette = ShadcnInspect::new(look).inspect_textarea_color_palette(style, sample.state, sample.enabled);
    let mut children = textarea_color_nodes(&id, &palette);
    children.push(choice_layout_branch(&id, expand, look, textarea_layout_data));

    TreeNode::new(id.clone(), sample.label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(children)
}

fn textarea_color_nodes(prefix: &str, palette: &TextFieldInspectPalette) -> Vec<TreeNode<ColorInspectTreeData>> {
    color_field_nodes_optional(
        prefix,
        &[
            ("background", Some(&palette.background)),
            ("foreground", Some(&palette.foreground)),
            ("border", Some(&palette.border)),
            ("placeholder", Some(&palette.placeholder)),
            ("selection background", Some(&palette.selection_background)),
            ("selection foreground", Some(&palette.selection_foreground)),
            ("caret", Some(&palette.caret)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
