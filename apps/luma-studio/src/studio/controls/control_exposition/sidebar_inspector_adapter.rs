use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{sidebar_container_color_rows, sidebar_item_color_rows, sidebar_section_color_rows};
use super::inspector::common::{interaction_state, sidebar_item_selected};
use super::inspector::metrics::sidebar_layout_section;
use super::inspector::specs::{CHOICE_SIZES, SIDEBAR_STATES, SIDEBAR_VARIANTS, NAV_ITEM_VALUE_MODES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static SIDEBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Sidebar",
    id_prefix: "sidebar-theme-inspector",
    parts: &[],
    variants: &SIDEBAR_VARIANTS,
    states: &SIDEBAR_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &NAV_ITEM_VALUE_MODES,
    default_part_id: "",
    default_variant_id: "container",
    default_size_id: "md",
    default_value_id: "unselected",
};

pub struct SidebarInspectorAdapter;

impl SidebarInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SidebarInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(sidebar_layout_section(
                look,
                "sidebar-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.variant_id {
            "container" | "section" => state.id == "default",
            "branch" => matches!(state.id, "default" | "hover" | "pressed" | "focused" | "disabled"),
            "nav-item" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
            _ => true,
        }
    }

    fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
        selection.variant_id == "nav-item"
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let inspect = ShadcnInspect::new(look);
    match selection.variant_id {
        "section" => {
            let palette = inspect.inspect_sidebar_section_color_palette();
            sidebar_section_color_rows(&palette)
        }
        "branch" => {
            let palette = inspect.inspect_sidebar_branch_color_palette(interaction_state(selection.state_id));
            sidebar_item_color_rows(&palette)
        }
        "nav-item" => {
            let palette = inspect.inspect_sidebar_item_color_palette(
                sidebar_item_selected(selection.value_id),
                interaction_state(selection.state_id),
            );
            sidebar_item_color_rows(&palette)
        }
        _ => {
            let palette = inspect.inspect_sidebar_container_color_palette();
            sidebar_container_color_rows(&palette)
        }
    }
}
