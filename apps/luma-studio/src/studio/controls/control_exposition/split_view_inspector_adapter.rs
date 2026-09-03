use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::split_view_color_rows;
use super::inspector::common::progress_enabled;
use super::inspector::metrics::split_view_layout_section;
use super::inspector::specs::PROGRESS_STATES;
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static SPLIT_VIEW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Split View",
    id_prefix: "split-view-theme-inspector",
    parts: &[],
    variants: &[],
    states: &PROGRESS_STATES,
    sizes: &[],
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "",
    default_value_id: "",
};

pub struct SplitViewInspectorAdapter;

impl SplitViewInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SplitViewInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(split_view_layout_section(
                look,
                "split-view-theme-inspector-box-model",
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_split_view_color_palette(progress_enabled(selection.state_id));
    split_view_color_rows(&palette)
}
