use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::resizable_panels_color_rows;
use super::inspector::common::interaction_state;
use super::inspector::metrics::resizable_panels_layout_section;
use super::inspector::specs::{RESIZE_HANDLE_SIZES, TREE_VIEW_ROW_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static RESIZABLE_PANELS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Resizable Panels",
    id_prefix: "resizable-panels-theme-inspector",
    parts: &[],
    variants: &[],
    states: &TREE_VIEW_ROW_STATES,
    sizes: &RESIZE_HANDLE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ResizablePanelsInspectorAdapter;

impl ResizablePanelsInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ResizablePanelsInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(resizable_panels_layout_section(
                look,
                "resizable-panels-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let palette =
        ShadcnInspect::new(look).inspect_resizable_panels_color_palette(interaction_state(selection.state_id));
    resizable_panels_color_rows(&palette)
}
