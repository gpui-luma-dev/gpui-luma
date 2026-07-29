use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::{interaction_state, scrollbar_style};
use super::inspector::metrics::scrollbar_layout_section;
use super::inspector::provenance::color_row;
use super::inspector::specs::{COLOR_LAYOUT_INTERACTION_STATES, SCROLLBAR_STYLE_VARIANTS};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static SCROLLBAR_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static SCROLLBAR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Scrollbar",
    id_prefix: "scrollbar-theme-inspector",
    parts: &[],
    variants: &SCROLLBAR_STYLE_VARIANTS,
    states: &SCROLLBAR_STATES,
    sizes: &[],
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "ghost",
    default_size_id: "",
    default_value_id: "",
};

pub struct ScrollbarInspectorAdapter;

impl ScrollbarInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ScrollbarInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(scrollbar_layout_section(
                look,
                "scrollbar-theme-inspector-box-model",
                selection.variant_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let style = scrollbar_style(selection.variant_id);
    let palette =
        ShadcnInspect::new(look).inspect_scrollbar_color_palette(style, interaction_state(selection.state_id));
    let mut rows = vec![
        color_row("track background", &palette.track_background),
        color_row("thumb background", &palette.thumb_background),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}
