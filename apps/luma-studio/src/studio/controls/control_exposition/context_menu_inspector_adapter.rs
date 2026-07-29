use std::sync::Arc;

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::interaction_state;
use super::inspector::input::{floating_menu_palette_rows, trigger_color_rows};
use super::inspector::metrics::context_menu_layout_section;
use super::inspector::specs::{CHOICE_SIZES, COLOR_LAYOUT_INTERACTION_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static CONTEXT_MENU_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static CONTEXT_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Context Menu",
    id_prefix: "context-menu-theme-inspector",
    variants: &[],
    states: &CONTEXT_MENU_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ContextMenuInspectorAdapter;

impl ContextMenuInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ContextMenuInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(context_menu_layout_section(
                look,
                "context-menu-theme-inspector-box-model",
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
    let palette =
        ShadcnInspect::new(look).inspect_context_menu_color_palette(interaction_state(selection.state_id), size);
    let mut rows = trigger_color_rows(
        &palette.target_background,
        &palette.target_foreground,
        &palette.target_border,
        palette.focus_ring.as_ref(),
    );
    rows.extend(floating_menu_palette_rows(&palette.menu));
    rows
}
