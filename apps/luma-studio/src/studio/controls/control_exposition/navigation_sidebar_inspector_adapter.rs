use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{
    navigation_sidebar_container_color_rows, navigation_sidebar_item_color_rows, navigation_sidebar_section_color_rows,
};
use super::inspector::common::{interaction_state, navigation_sidebar_item_selected};
use super::inspector::metrics::navigation_sidebar_layout_section;
use super::inspector::specs::{CHOICE_SIZES, NAVIGATION_SIDEBAR_STATES, NAVIGATION_SIDEBAR_VARIANTS, NAV_ITEM_VALUE_MODES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static NAVIGATION_SIDEBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Navigation Sidebar",
    id_prefix: "navigation-sidebar-theme-inspector",
    parts: &[],
    variants: &NAVIGATION_SIDEBAR_VARIANTS,
    states: &NAVIGATION_SIDEBAR_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &NAV_ITEM_VALUE_MODES,
    default_part_id: "",
    default_variant_id: "container",
    default_size_id: "md",
    default_value_id: "unselected",
};

pub struct NavigationSidebarInspectorAdapter;

impl NavigationSidebarInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for NavigationSidebarInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(navigation_sidebar_layout_section(
                look,
                "navigation-sidebar-theme-inspector-box-model",
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
            let palette = inspect.inspect_navigation_sidebar_section_color_palette();
            navigation_sidebar_section_color_rows(&palette)
        }
        "branch" => {
            let palette =
                inspect.inspect_navigation_sidebar_branch_color_palette(interaction_state(selection.state_id));
            navigation_sidebar_item_color_rows(&palette)
        }
        "nav-item" => {
            let palette = inspect.inspect_navigation_sidebar_item_color_palette(
                navigation_sidebar_item_selected(selection.value_id),
                interaction_state(selection.state_id),
            );
            navigation_sidebar_item_color_rows(&palette)
        }
        _ => {
            let palette = inspect.inspect_navigation_sidebar_container_color_palette();
            navigation_sidebar_container_color_rows(&palette)
        }
    }
}
