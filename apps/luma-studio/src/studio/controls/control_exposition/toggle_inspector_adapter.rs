use std::sync::Arc;

use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{interaction_state, primary_secondary_style, value_flag};
use super::inspector::metrics::toggle_layout_section;
use super::inspector::provenance::color_row;
use super::inspector::specs::{
    CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES, PRIMARY_SECONDARY_VARIANTS, SELECTED_UNSELECTED_VALUES,
};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static TOGGLE_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static TOGGLE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Toggle",
    id_prefix: "toggle-theme-inspector",
    variants: &PRIMARY_SECONDARY_VARIANTS,
    states: &TOGGLE_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &SELECTED_UNSELECTED_VALUES,
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "selected",
};

pub struct ToggleInspectorAdapter;

impl ToggleInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ToggleInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(toggle_layout_section(
                look,
                "toggle-theme-inspector-box-model",
                selection.variant_id,
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let selected = value_flag(selection.value_id, "selected");
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        primary_secondary_style(selection.variant_id),
        ButtonFamilyRole::Toggle { selected },
        interaction_state(selection.state_id),
    );
    let mut rows = vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}
