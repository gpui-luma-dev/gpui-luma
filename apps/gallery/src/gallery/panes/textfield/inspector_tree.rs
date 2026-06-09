use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::{TextFieldInspectPalette};

use crate::gallery::panes::shared::inspector::{ColorInspectTreeData, color_field_nodes_optional, inspect_slug};

struct TextFieldMatrixState {
    label: &'static str,
    state: TextFieldState,
    enabled: bool,
}

pub(in crate::gallery) fn build_textfield_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>> {
    let variants = [(ShadcnTextFieldStyle::Surface, "Surface"), (ShadcnTextFieldStyle::Soft, "Soft")];
    let matrix_states = [
        TextFieldMatrixState { label: "default", state: TextFieldState::default(), enabled: true },
        TextFieldMatrixState {
            label: "hover",
            state: TextFieldState { hovered: true, ..TextFieldState::default() },
            enabled: true,
        },
        TextFieldMatrixState {
            label: "focus",
            state: TextFieldState { focused: true, focus_visible: true, ..TextFieldState::default() },
            enabled: true,
        },
        TextFieldMatrixState { label: "disabled", state: TextFieldState::default(), enabled: false },
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
    sample: &TextFieldMatrixState,
    style: ShadcnTextFieldStyle,
    look: &ShadcnLook,
) -> TreeNode<ColorInspectTreeData> {
    let id = format!("{prefix}-{}-{}", inspect_slug(style_label), sample.label);
    let palette = ShadcnInspect::new(look).inspect_textfield_color_palette(style, sample.state, sample.enabled);
    let expand = style_label == "Surface" && sample.label == "default";

    TreeNode::new(id.clone(), sample.label.to_owned(), ColorInspectTreeData::Branch)
        .branch(true)
        .expanded(expand)
        .children(textfield_color_nodes(&id, &palette))
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
            ("selection background", Some(&palette.selection_background)),
            ("selection foreground", Some(&palette.selection_foreground)),
            ("caret", Some(&palette.caret)),
            ("focus ring", palette.focus_ring.as_ref()),
        ],
    )
}
