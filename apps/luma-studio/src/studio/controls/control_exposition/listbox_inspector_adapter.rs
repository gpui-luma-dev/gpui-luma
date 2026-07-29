use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::{listbox_list_color_rows, listbox_row_color_rows};
use super::inspector::common::{listbox_list_enabled, listbox_list_focused, listbox_row_state};
use super::inspector::metrics::listbox_layout_section;
use super::inspector::specs::{CHOICE_SIZES, LISTBOX_STATES, LISTBOX_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection, InspectorStateSpec,
    SharedInspectorResolver,
};

pub static LISTBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "ListBox",
    id_prefix: "listbox-theme-inspector",
    parts: &[],
    variants: &LISTBOX_VARIANTS,
    states: &LISTBOX_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "list",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ListBoxInspectorAdapter;

impl ListBoxInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ListBoxInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(listbox_layout_section(
                look,
                "listbox-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.variant_id {
            "list" => matches!(state.id, "enabled" | "disabled" | "focused"),
            "row" => matches!(state.id, "default" | "hover" | "pressed" | "keyboard-active" | "disabled"),
            _ => true,
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let inspect = ShadcnInspect::new(look);
    match selection.variant_id {
        "row" => {
            let palette = inspect.inspect_listbox_row_color_palette(listbox_row_state(selection.state_id));
            listbox_row_color_rows(&palette)
        }
        _ => {
            let palette = inspect.inspect_listbox_list_color_palette(
                listbox_list_enabled(selection.state_id),
                listbox_list_focused(selection.state_id),
            );
            listbox_list_color_rows(&palette)
        }
    }
}
