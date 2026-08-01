use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::progress_enabled;
use super::inspector::metrics::{progress_circular_layout_section, progress_linear_layout_section};
use super::inspector::provenance::color_row;
use super::inspector::schema::InspectLayoutSection;
use super::inspector::specs::{PROGRESS_STATES, PROGRESS_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static PROGRESS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Progress",
    id_prefix: "progress-theme-inspector",
    parts: &[],
    variants: &PROGRESS_VARIANTS,
    states: &PROGRESS_STATES,
    sizes: &[],
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "circular",
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
            "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_layout_section(look: &ShadcnLook, selection: InspectorSelection<'_>) -> InspectLayoutSection {
    let metrics = ShadcnInspect::new(look).inspect_progress_metrics();
    let diagram_id = "progress-theme-inspector-box-model";
    if selection.variant_id == "linear" {
        progress_linear_layout_section(look, diagram_id, &metrics)
    } else {
        progress_circular_layout_section(look, diagram_id, &metrics)
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_progress_color_palette(progress_enabled(selection.state_id));
    let mut rows = vec![color_row("track", &palette.track_color), color_row("progress", &palette.progress_color)];
    if selection.variant_id == "linear" {
        rows.push(color_row("thumb", &palette.thumb_color));
    }
    rows
}
