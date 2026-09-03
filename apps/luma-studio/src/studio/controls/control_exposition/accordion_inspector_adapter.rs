use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{accordion_content_color_rows, accordion_trigger_color_rows};
use super::inspector::common::{accordion_content_expanded, interaction_state};
use super::inspector::metrics::accordion_layout_section;
use super::inspector::specs::{ACCORDION_STATES, ACCORDION_VARIANTS, CHOICE_SIZES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static ACCORDION_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Accordion",
    id_prefix: "accordion-theme-inspector",
    parts: &[],
    variants: &ACCORDION_VARIANTS,
    states: &ACCORDION_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "trigger",
    default_size_id: "md",
    default_value_id: "",
};

pub struct AccordionInspectorAdapter;

impl AccordionInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for AccordionInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(accordion_layout_section(
                look,
                "accordion-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.variant_id {
            "trigger" => matches!(state.id, "default" | "hover" | "pressed" | "disabled"),
            "content" => matches!(state.id, "expanded" | "collapsed"),
            _ => true,
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let inspect = ShadcnInspect::new(look);
    if selection.variant_id == "content" {
        let palette = inspect.inspect_accordion_content_color_palette(accordion_content_expanded(selection.state_id));
        return accordion_content_color_rows(&palette);
    }
    let palette = inspect.inspect_accordion_trigger_color_palette(interaction_state(selection.state_id));
    accordion_trigger_color_rows(&palette)
}
