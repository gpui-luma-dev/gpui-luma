use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{
    choice_elevation_applies, choice_indicator_only, choice_variant_style, interaction_state, value_flag,
};
use super::inspector::metrics::radio_layout_section;
use super::inspector::provenance::{color_row, elevation_snapshot};
use super::inspector::specs::{CHOICE_INTERACTION_STATES, CHOICE_LAYOUT_PARTS, CHOICE_SIZES, SELECTED_UNSELECTED_VALUES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static RADIO_BUTTON_STATES: [super::inspector::InspectorStateSpec; 5] = CHOICE_INTERACTION_STATES;

pub static RADIO_BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Radio Button",
    id_prefix: "radio-button-theme-inspector",
    parts: &CHOICE_LAYOUT_PARTS,
    variants: &[],
    states: &RADIO_BUTTON_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &SELECTED_UNSELECTED_VALUES,
    default_part_id: "labeled",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "selected",
};

pub struct RadioButtonInspectorAdapter;

impl RadioButtonInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for RadioButtonInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(radio_layout_section(
                look,
                "radio-button-theme-inspector-box-model",
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
    let selected = value_flag(selection.value_id, "selected");
    let palette = ShadcnInspect::new(look).inspect_radio_button_color_palette(
        choice_variant_style(selection.variant_id),
        selected,
        interaction_state(selection.state_id),
    );
    let mut rows = vec![
        color_row("indicator background", &palette.indicator_background),
        color_row("indicator border", &palette.indicator_border),
        color_row("dot", &palette.dot_color),
    ];
    if !choice_indicator_only(selection.part_id) {
        rows.push(color_row("label", &palette.label_color));
    }
    rows
}

fn resolve_elevation(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectElevationSnapshot {
    let selected = value_flag(selection.value_id, "selected");
    let elevation = ShadcnInspect::new(look).inspect_radio_button_elevation(
        choice_variant_style(selection.variant_id),
        selected,
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved radio button look", "radio.elevation_rules[].style")
}
