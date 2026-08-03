use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{control_size, interaction_state, popup_menu_trigger_style};
use super::inspector::input::trigger_color_rows;
use super::inspector::metrics::{floating_menu_layout_section, popup_menu_trigger_layout_section};
use super::inspector::provenance::color_row;
use super::inspector::specs::{CHOICE_SIZES, POPUP_MENU_PARTS, POPUP_MENU_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    InspectorStateSpec, SharedInspectorResolver,
};

pub static POPUP_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Popup Menu",
    id_prefix: "popup-menu-theme-inspector",
    parts: &POPUP_MENU_PARTS,
    variants: &[],
    states: &POPUP_MENU_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "trigger",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct PopupMenuInspectorAdapter;

impl PopupMenuInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for PopupMenuInspectorAdapter {
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
            "trigger" => matches!(state.id, "default" | "hover" | "focused" | "pressed" | "disabled"),
            "panel" => matches!(state.id, "panel-default" | "panel-hover" | "panel-disabled"),
            _ => true,
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let size = control_size(selection.size_id);

    if selection.part_id == "panel" {
        let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(size);
        return resolve_panel_color_rows(&palette, selection.state_id);
    }

    let palette = ShadcnInspect::new(look).inspect_popup_menu_color_palette(
        popup_menu_trigger_style(selection.variant_id),
        interaction_state(selection.state_id),
        size,
    );
    trigger_color_rows(&palette.trigger_background, &palette.trigger_foreground, &palette.trigger_border)
}

fn resolve_panel_color_rows(
    palette: &gpui_luma_look_shadcn_inspect::FloatingMenuInspectPalette,
    state_id: &str,
) -> Vec<InspectColorRow> {
    match state_id {
        "panel-hover" => vec![
            color_row("item hover background", &palette.item_hover_background),
            color_row("item hover foreground", &palette.item_hover_foreground),
        ],
        "panel-disabled" => vec![color_row("item disabled foreground", &palette.item_disabled_foreground)],
        _ => vec![
            color_row("background", &palette.background),
            color_row("foreground", &palette.foreground),
            color_row("border", &palette.border),
        ],
    }
}

fn resolve_layout_section(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectLayoutSection {
    match selection.part_id {
        "panel" => floating_menu_layout_section(look, "popup-menu-theme-inspector-panel-box-model", selection.size_id),
        _ => popup_menu_trigger_layout_section(
            look,
            "popup-menu-theme-inspector-trigger-box-model",
            selection.variant_id,
            selection.size_id,
        ),
    }
}
