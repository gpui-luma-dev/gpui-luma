use std::sync::Arc;

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{interaction_state, popup_menu_trigger_style};
use super::inspector::input::{floating_menu_palette_rows, trigger_color_rows};
use super::inspector::metrics::popup_menu_layout_section;
use super::inspector::specs::{CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES, POPUP_MENU_TRIGGER_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static POPUP_MENU_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static POPUP_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Popup Menu",
    id_prefix: "popup-menu-theme-inspector",
    variants: &POPUP_MENU_TRIGGER_VARIANTS,
    states: &POPUP_MENU_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_variant_id: "outline",
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
            "layout" => InspectorCategoryContent::Layout(popup_menu_layout_section(
                look,
                "popup-menu-theme-inspector-box-model",
                selection.variant_id,
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let size = match selection.size_id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
    };
    let trigger_style = popup_menu_trigger_style(selection.variant_id);
    let palette = ShadcnInspect::new(look).inspect_popup_menu_color_palette(
        trigger_style,
        interaction_state(selection.state_id),
        size,
    );
    let mut rows = trigger_color_rows(
        &palette.trigger_background,
        &palette.trigger_foreground,
        &palette.trigger_border,
        palette.focus_ring.as_ref(),
    );
    rows.extend(floating_menu_palette_rows(&palette.menu));
    rows
}
