use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::input::{autocomplete_chrome_color_rows, floating_menu_color_rows};
use super::inspector::metrics::autocomplete_layout_section;
use super::inspector::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static AUTOCOMPLETE_STATES: [super::inspector::InspectorStateSpec; 1] = DEFAULT_INTERACTION_STATES;

pub static AUTOCOMPLETE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Autocomplete",
    id_prefix: "autocomplete-theme-inspector",
    parts: &[],
    variants: &[],
    states: &AUTOCOMPLETE_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct AutocompleteInspectorAdapter;

impl AutocompleteInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for AutocompleteInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look)),
            "layout" => InspectorCategoryContent::Layout(autocomplete_layout_section(
                look,
                "autocomplete-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
    let chrome = ShadcnInspect::new(look).inspect_autocomplete_chrome_color_palette();
    let mut rows = autocomplete_chrome_color_rows(&chrome);
    rows.extend(floating_menu_color_rows(look));
    rows
}
