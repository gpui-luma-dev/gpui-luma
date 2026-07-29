use std::sync::Arc;

use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::selector_textfield_state;
use super::inspector::input::{floating_menu_color_rows, textfield_trigger_color_rows};
use super::inspector::metrics::selector_layout_section;
use super::inspector::specs::{CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static SELECTOR_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Selector",
    id_prefix: "selector-theme-inspector",
    parts: &[],
    variants: &[],
    states: &SELECTOR_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct SelectorInspectorAdapter;

impl SelectorInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SelectorInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(selector_layout_section(
                look,
                "selector-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let enabled = selection.state_id != "disabled";
    let field_state = selector_textfield_state(selection.state_id);
    let trigger =
        ShadcnInspect::new(look).inspect_textfield_color_palette(ShadcnTextFieldStyle::Input, field_state, enabled);
    let mut rows = textfield_trigger_color_rows(&trigger);
    rows.extend(floating_menu_color_rows(look));
    rows
}
