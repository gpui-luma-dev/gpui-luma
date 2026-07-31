use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{list_view_row_color_rows, list_view_surface_color_rows};
use super::inspector::common::{list_view_row_selected, listbox_row_state, progress_enabled};
use super::inspector::metrics::list_view_layout_section;
use super::inspector::specs::{CHOICE_SIZES, LIST_VIEW_ROW_VALUE_MODES, LIST_VIEW_STATES, LIST_VIEW_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static LIST_VIEW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "ListView",
    id_prefix: "list-view-theme-inspector",
    parts: &[],
    variants: &LIST_VIEW_VARIANTS,
    states: &LIST_VIEW_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &LIST_VIEW_ROW_VALUE_MODES,
    default_part_id: "",
    default_variant_id: "surface",
    default_size_id: "md",
    default_value_id: "unselected",
};

pub struct ListViewInspectorAdapter;

impl ListViewInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ListViewInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(list_view_layout_section(
                look,
                "list-view-theme-inspector-box-model",
                selection.variant_id,
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.variant_id {
            "surface" => matches!(state.id, "enabled" | "disabled"),
            "row" | "grid-cell" => {
                matches!(state.id, "default" | "hover" | "pressed" | "keyboard-active" | "disabled")
            }
            _ => true,
        }
    }

    fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
        selection.variant_id == "row"
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let inspect = ShadcnInspect::new(look);
    match selection.variant_id {
        "row" | "grid-cell" => {
            let palette = inspect.inspect_list_view_row_color_palette(
                list_view_row_selected(selection.value_id),
                listbox_row_state(selection.state_id),
            );
            list_view_row_color_rows(&palette)
        }
        _ => {
            let palette = inspect.inspect_list_view_color_palette(progress_enabled(selection.state_id));
            list_view_surface_color_rows(&palette)
        }
    }
}
