use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{tabs_item_color_rows, tabs_list_color_rows};
use super::inspector::common::{interaction_state, progress_enabled, tabs_active};
use super::inspector::metrics::tabs_layout_section;
use super::inspector::specs::{CHOICE_SIZES, TABS_NAVIGATION_STATES, TABS_NAVIGATION_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static TABS_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Tabs Navigation",
    id_prefix: "tabs-navigation-theme-inspector",
    parts: &[],
    variants: &TABS_NAVIGATION_VARIANTS,
    states: &TABS_NAVIGATION_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "inactive",
    default_size_id: "md",
    default_value_id: "",
};

pub struct TabsInspectorAdapter;

impl TabsInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for TabsInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(tabs_layout_section(
                look,
                "tabs-navigation-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.variant_id {
            "list" => matches!(state.id, "enabled" | "disabled"),
            "inactive" | "active" => {
                matches!(state.id, "default" | "hover" | "focused" | "pressed" | "item-disabled")
            }
            _ => true,
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let inspect = ShadcnInspect::new(look);
    if selection.variant_id == "list" {
        let palette = inspect.inspect_tabs_list_color_palette(progress_enabled(selection.state_id));
        return tabs_list_color_rows(&palette);
    }

    let state_id = if selection.state_id == "item-disabled" {
        "disabled"
    } else {
        selection.state_id
    };
    let palette =
        inspect.inspect_tabs_item_color_palette(tabs_active(selection.variant_id), interaction_state(state_id));
    tabs_item_color_rows(&palette)
}
