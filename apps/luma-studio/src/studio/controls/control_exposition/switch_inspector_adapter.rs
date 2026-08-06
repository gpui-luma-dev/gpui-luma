use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{choice_elevation_applies, choice_variant_style, interaction_state, value_flag};
use super::inspector::metrics::switch_layout_section;
use super::inspector::provenance::{color_row, elevation_snapshot};
use super::inspector::specs::{
    CHOICE_INTERACTION_STATES, CHOICE_SIZES, ON_OFF_VALUES, PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static SWITCH_STATES: [super::inspector::InspectorStateSpec; 5] = CHOICE_INTERACTION_STATES;

pub static SWITCH_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Switch",
    id_prefix: "switch-theme-inspector",
    parts: &[],
    variants: &PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS,
    states: &SWITCH_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &ON_OFF_VALUES,
    default_part_id: "",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "on",
};

pub struct SwitchInspectorAdapter;

impl SwitchInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SwitchInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(switch_layout_section(
                look,
                "switch-theme-inspector-box-model",
                selection.variant_id,
                selection.size_id,
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
    let on = value_flag(selection.value_id, "on");
    let palette = ShadcnInspect::new(look).inspect_switch_color_palette(
        choice_variant_style(selection.variant_id),
        on,
        interaction_state(selection.state_id),
    );
    let rows = vec![
        color_row("track background", &palette.track_background),
        color_row("track border", &palette.track_border),
        color_row("thumb background", &palette.thumb_background),
        color_row("thumb border", &palette.thumb_border),
        color_row("label", &palette.label_color),
    ];
    rows
}

fn resolve_elevation(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectElevationSnapshot {
    let on = value_flag(selection.value_id, "on");
    let elevation = ShadcnInspect::new(look).inspect_switch_elevation(
        choice_variant_style(selection.variant_id),
        on,
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved switch look", "switch.elevation_rules[].style")
}
