use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{
    choice_elevation_applies, choice_indicator_only, choice_variant_style, interaction_state, value_flag,
};
use super::inspector::metrics::checkbox_layout_section;
use super::inspector::provenance::{color_row, elevation_snapshot};
use super::inspector::specs::{CHOICE_INTERACTION_STATES, CHOICE_LAYOUT_PARTS, CHOICE_SIZES, CHECKED_UNCHECKED_VALUES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static CHECKBOX_STATES: [super::inspector::InspectorStateSpec; 5] = CHOICE_INTERACTION_STATES;

pub static CHECKBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Checkbox",
    id_prefix: "checkbox-theme-inspector",
    parts: &CHOICE_LAYOUT_PARTS,
    variants: &[],
    states: &CHECKBOX_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &CHECKED_UNCHECKED_VALUES,
    default_part_id: "labeled",
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
                choice_indicator_only(selection.part_id),
            )),
            "elevation" => InspectorCategoryContent::Elevation(resolve_elevation(look, selection)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        if category_id == "elevation" {
            return choice_elevation_applies(selection.variant_id, selection.state_id);
        }
        true
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let checked = value_flag(selection.value_id, "checked");
    let palette = ShadcnInspect::new(look).inspect_checkbox_color_palette(
        choice_variant_style(selection.variant_id),
        checked,
        interaction_state(selection.state_id),
    );
    let mut rows = vec![
        color_row("indicator background", &palette.indicator_background),
        color_row("indicator border", &palette.indicator_border),
        color_row("checkmark", &palette.checkmark_color),
    ];
    if !choice_indicator_only(selection.part_id) {
        rows.push(color_row("label", &palette.label_color));
    }
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
        choice_variant_style(selection.variant_id),
        checked,
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved checkbox look", "checkbox.elevation_rules[].style")
}
