use std::sync::Arc;

use gpui_luma::controls::overlay_window::OverlayWindowMode;
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::input::overlay_window_palette_rows;
use super::inspector::metrics::overlay_window_layout_section;
use super::inspector::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES, OVERLAY_WINDOW_MODE_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

pub static OVERLAY_WINDOW_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Overlay Window",
    id_prefix: "overlay-window-theme-inspector",
    parts: &[],
    variants: &OVERLAY_WINDOW_MODE_VARIANTS,
    states: &DEFAULT_INTERACTION_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "modeless",
    default_size_id: "md",
    default_value_id: "",
};

pub struct OverlayWindowInspectorAdapter;

impl OverlayWindowInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for OverlayWindowInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(overlay_window_layout_section(
                look,
                "overlay-window-theme-inspector-box-model",
                selection.variant_id,
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<super::inspector::InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_overlay_window_color_palette(
        overlay_window_size(selection.size_id),
        overlay_window_mode(selection.variant_id),
    );
    overlay_window_palette_rows(&palette)
}

fn overlay_window_size(size_id: &str) -> ControlSize {
    match size_id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
    }
}

fn overlay_window_mode(variant_id: &str) -> OverlayWindowMode {
    match variant_id {
        "modal" => OverlayWindowMode::Modal,
        _ => OverlayWindowMode::Modeless,
    }
}
