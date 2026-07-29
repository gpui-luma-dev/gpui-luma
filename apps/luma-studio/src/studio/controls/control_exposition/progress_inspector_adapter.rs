use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::progress_enabled;
use super::inspector::metrics::progress_layout_section;
use super::inspector::provenance::color_row;
use super::inspector::specs::PROGRESS_STATES;
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static PROGRESS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Progress",
    id_prefix: "progress-theme-inspector",
    variants: &[],
    states: &PROGRESS_STATES,
    sizes: &[],
    value_modes: &[],
    default_variant_id: "",
    default_size_id: "",
    default_value_id: "",
};

pub struct ProgressInspectorAdapter;

impl ProgressInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ProgressInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => {
                InspectorCategoryContent::Layout(progress_layout_section(look, "progress-theme-inspector-box-model"))
            }
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_progress_color_palette(progress_enabled(selection.state_id));
    vec![color_row("track", &palette.track_color), color_row("progress", &palette.progress_color)]
}
