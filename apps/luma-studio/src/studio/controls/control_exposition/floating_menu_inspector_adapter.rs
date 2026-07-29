use std::sync::Arc;

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::input::floating_menu_palette_rows;
use super::inspector::metrics::floating_menu_layout_section;
use super::inspector::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static FLOATING_MENU_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Floating Menu",
    id_prefix: "floating-menu-theme-inspector",
    variants: &[],
    states: &DEFAULT_INTERACTION_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct FloatingMenuInspectorAdapter;

impl FloatingMenuInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for FloatingMenuInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(floating_menu_layout_section(
                look,
                "floating-menu-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let size = match selection.size_id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
    };
    let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(size);
    floating_menu_palette_rows(&palette)
}
