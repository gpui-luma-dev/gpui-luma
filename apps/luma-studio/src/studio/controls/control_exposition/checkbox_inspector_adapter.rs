use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{inspector_elevation_applies, interaction_state, primary_secondary_style, value_flag};
use super::inspector::metrics::checkbox_layout_section;
use super::inspector::provenance::{color_row, elevation_snapshot};
use super::inspector::specs::{
    CHOICE_INTERACTION_STATES, CHOICE_SIZES, CHECKED_UNCHECKED_VALUES, PRIMARY_SECONDARY_VARIANTS,
};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static CHECKBOX_STATES: [super::inspector::InspectorStateSpec; 5] = CHOICE_INTERACTION_STATES;

pub static CHECKBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Checkbox",
    id_prefix: "checkbox-theme-inspector",
    parts: &[],
    variants: &PRIMARY_SECONDARY_VARIANTS,
    states: &CHECKBOX_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &CHECKED_UNCHECKED_VALUES,
    default_part_id: "",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "checked",
};

pub struct CheckboxInspectorAdapter;

impl CheckboxInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for CheckboxInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(checkbox_layout_section(
                look,
                "checkbox-theme-inspector-box-model",
                selection.size_id,
            )),
            "elevation" => InspectorCategoryContent::Elevation(resolve_elevation(look, selection)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        if category_id == "elevation" {
            return inspector_elevation_applies(selection.state_id);
        }
        true
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let checked = value_flag(selection.value_id, "checked");
    let palette = ShadcnInspect::new(look).inspect_checkbox_color_palette(
        primary_secondary_style(selection.variant_id),
        checked,
        interaction_state(selection.state_id),
    );
    let mut rows = vec![
        color_row("indicator background", &palette.indicator_background),
        color_row("indicator border", &palette.indicator_border),
        color_row("checkmark", &palette.checkmark_color),
        color_row("label", &palette.label_color),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}

fn resolve_elevation(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectElevationSnapshot {
    let checked = value_flag(selection.value_id, "checked");
    let elevation = ShadcnInspect::new(look).inspect_checkbox_elevation(
        primary_secondary_style(selection.variant_id),
        checked,
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved checkbox look", "checkbox.elevation_rules[].style")
}
