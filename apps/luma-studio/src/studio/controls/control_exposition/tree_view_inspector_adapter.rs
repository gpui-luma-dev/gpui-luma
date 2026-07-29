use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::tree_view_row_color_rows;
use super::inspector::common::{interaction_state, listbox_row_state};
use super::inspector::metrics::tree_view_layout_section;
use super::inspector::specs::{CHOICE_SIZES, TREE_VIEW_ROW_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static TREE_VIEW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "TreeView",
    id_prefix: "tree-view-theme-inspector",
    parts: &[],
    variants: &[],
    states: &TREE_VIEW_ROW_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct TreeViewInspectorAdapter;

impl TreeViewInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for TreeViewInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(tree_view_layout_section(
                look,
                "tree-view-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let state = if selection.state_id == "disabled" {
        interaction_state("disabled")
    } else {
        listbox_row_state(selection.state_id)
    };
    let palette = ShadcnInspect::new(look).inspect_tree_view_row_color_palette(state);
    tree_view_row_color_rows(&palette)
}
