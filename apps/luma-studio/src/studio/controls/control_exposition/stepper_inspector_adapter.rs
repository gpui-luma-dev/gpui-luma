use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::stepper_enabled;
use super::inspector::metrics::{stepper_horizontal_layout_section, stepper_vertical_layout_section};
use super::inspector::provenance::color_row;
use super::inspector::schema::InspectLayoutSection;
use super::inspector::specs::{STEPPER_STATES, STEPPER_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static STEPPER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Stepper",
    id_prefix: "stepper-theme-inspector",
    parts: &[],
    variants: &STEPPER_VARIANTS,
    states: &STEPPER_STATES,
    sizes: &[],
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "horizontal",
    default_size_id: "",
    default_value_id: "",
};

pub struct StepperInspectorAdapter;

impl StepperInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for StepperInspectorAdapter {
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
    let metrics = ShadcnInspect::new(look).inspect_stepper_metrics();
    let diagram_id = "stepper-theme-inspector-box-model";
    if selection.variant_id == "vertical" {
        stepper_vertical_layout_section(look, diagram_id, &metrics)
    } else {
        stepper_horizontal_layout_section(look, diagram_id, &metrics)
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let enabled = stepper_enabled(selection.state_id);
    let inspect = ShadcnInspect::new(look);
    let progress = inspect.inspect_progress_color_palette(enabled);
    let palette = inspect.inspect_stepper_color_palette(enabled);
    vec![
        color_row("track", &progress.track_color),
        color_row("progress", &progress.progress_color),
        color_row("incomplete bg", &palette.incomplete_bg),
        color_row("incomplete border", &palette.incomplete_border),
        color_row("incomplete fg", &palette.incomplete_fg),
    ]
}
