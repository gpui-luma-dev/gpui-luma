use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{table_row_color_rows, table_surface_color_rows};
use super::inspector::common::{table_row_selected, listbox_row_state, progress_enabled};
use super::inspector::metrics::table_layout_section;
use super::inspector::specs::{CHOICE_SIZES, TABLE_ROW_VALUE_MODES, TABLE_STATES, TABLE_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static TABLE_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Table",
    id_prefix: "table-theme-inspector",
    parts: &[],
    variants: &TABLE_VARIANTS,
    states: &TABLE_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &TABLE_ROW_VALUE_MODES,
    default_part_id: "",
    default_variant_id: "surface",
    default_size_id: "md",
    default_value_id: "unselected",
};

pub struct TableInspectorAdapter;

impl TableInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for TableInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(table_layout_section(
                look,
                "table-theme-inspector-box-model",
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
            let palette = inspect.inspect_table_row_color_palette(
                table_row_selected(selection.value_id),
                listbox_row_state(selection.state_id),
            );
            table_row_color_rows(&palette)
        }
        _ => {
            let palette = inspect.inspect_table_color_palette(progress_enabled(selection.state_id));
            table_surface_color_rows(&palette)
        }
    }
}
