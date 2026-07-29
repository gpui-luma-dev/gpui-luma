use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnButtonStyle;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::collection::pager_shell_color_rows;
use super::inspector::common::{interaction_state, pager_button_role, pager_shell_enabled, pager_style};
use super::inspector::metrics::{pager_button_layout_section, pager_shell_layout_section};
use super::inspector::provenance::color_row;
use super::inspector::specs::{PAGER_PARTS, PAGER_STATES, SELECTED_UNSELECTED_VALUES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    InspectorStateSpec, SharedInspectorResolver,
};

pub static PAGER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Pager",
    id_prefix: "pager-theme-inspector",
    parts: &PAGER_PARTS,
    variants: &[],
    states: &PAGER_STATES,
    sizes: &[],
    value_modes: &SELECTED_UNSELECTED_VALUES,
    default_part_id: "shell",
    default_variant_id: "",
    default_size_id: "",
    default_value_id: "selected",
};

pub struct PagerInspectorAdapter;

impl PagerInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for PagerInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn state_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, state: &InspectorStateSpec) -> bool {
        match selection.part_id {
            "shell" => matches!(state.id, "enabled" | "disabled"),
            "button" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
            _ => true,
        }
    }

    fn value_modes_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>) -> bool {
        selection.part_id == "button" && selection.variant_id == "page"
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    if selection.part_id == "button" {
        return resolve_button_color_rows(look, selection);
    }

    let palette = ShadcnInspect::new(look)
        .inspect_pager_shell_color_palette(pager_shell_enabled(selection.state_id), pager_style(selection.variant_id));
    pager_shell_color_rows(&palette)
}

fn resolve_button_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        ShadcnButtonStyle::Outline,
        pager_button_role(selection.variant_id, selection.value_id),
        interaction_state(selection.state_id),
    );
    let mut rows = vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}

fn resolve_layout_section(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectLayoutSection {
    match selection.part_id {
        "button" => pager_button_layout_section(look, "pager-theme-inspector-button-box-model"),
        _ => pager_shell_layout_section(look, "pager-theme-inspector-shell-box-model", selection.variant_id),
    }
}
