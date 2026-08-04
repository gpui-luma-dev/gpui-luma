use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{textfield_elevation_applies, textfield_enabled, textfield_variant_style, textarea_state};
use super::inspector::input::textfield_color_rows;
use super::inspector::metrics::textarea_layout_section;
use super::inspector::provenance::elevation_snapshot;
use super::inspector::specs::{CHOICE_SIZES, TEXTFIELD_INTERACTION_STATES, TEXTFIELD_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static TEXTAREA_STATES: [super::inspector::InspectorStateSpec; 3] = TEXTFIELD_INTERACTION_STATES;

pub static TEXTAREA_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "TextArea",
    id_prefix: "textarea-theme-inspector",
    parts: &[],
    variants: &TEXTFIELD_VARIANTS,
    states: &TEXTAREA_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "outline",
    default_size_id: "md",
    default_value_id: "",
};

pub struct TextAreaInspectorAdapter;

impl TextAreaInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for TextAreaInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        let style = textfield_variant_style(selection.variant_id);
        let enabled = textfield_enabled(selection.state_id);
        let state = textarea_state(selection.state_id);
        match category_id {
            "color" => {
                let palette = ShadcnInspect::new(look).inspect_textarea_color_palette(style, state, enabled);
                InspectorCategoryContent::Colors(textfield_color_rows(&palette))
            }
            "layout" => InspectorCategoryContent::Layout(textarea_layout_section(
                look,
                "textarea-theme-inspector-box-model",
                selection.size_id,
            )),
            "elevation" => InspectorCategoryContent::Elevation(elevation_snapshot(
                &ShadcnInspect::new(look).inspect_textfield_elevation(style, enabled),
                "resolved textarea look",
                "textfield.elevation_rules[].style",
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        if category_id == "elevation" {
            return textfield_elevation_applies(selection.variant_id, selection.state_id);
        }
        true
    }
}
