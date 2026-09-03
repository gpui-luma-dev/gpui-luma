use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::toolbar_shell_color_rows;
use super::inspector::common::{progress_enabled, toolbar_variant};
use super::inspector::metrics::toolbar_layout_section;
use super::inspector::specs::{CHOICE_SIZES, PROGRESS_STATES, TOOLBAR_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static TOOLBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Toolbar",
    id_prefix: "toolbar-theme-inspector",
    parts: &[],
    variants: &TOOLBAR_VARIANTS,
    states: &PROGRESS_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "outline",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ToolbarInspectorAdapter;

impl ToolbarInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ToolbarInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(toolbar_layout_section(
                look,
                "toolbar-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let palette = ShadcnInspect::new(look)
        .inspect_toolbar_color_palette(progress_enabled(selection.state_id), toolbar_variant(selection.variant_id));
    toolbar_shell_color_rows(&palette)
}
