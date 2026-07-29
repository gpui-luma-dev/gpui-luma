use std::sync::Arc;

use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::common::interaction_state;
use super::inspector::metrics::slider_layout_section;
use super::inspector::provenance::color_row;
use super::inspector::specs::COLOR_LAYOUT_INTERACTION_STATES;
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorSelection,
    SharedInspectorResolver,
};

static SLIDER_STATES: [super::inspector::InspectorStateSpec; 5] = COLOR_LAYOUT_INTERACTION_STATES;

pub static SLIDER_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Slider",
    id_prefix: "slider-theme-inspector",
    variants: &[],
    states: &SLIDER_STATES,
    sizes: &[],
    value_modes: &[],
    default_variant_id: "",
    default_size_id: "",
    default_value_id: "",
};

pub struct SliderInspectorAdapter;

impl SliderInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SliderInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => {
                InspectorCategoryContent::Layout(slider_layout_section(look, "slider-theme-inspector-box-model"))
            }
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_slider_color_palette(interaction_state(selection.state_id));
    let mut rows = vec![
        color_row("track background", &palette.track_background),
        color_row("fill background", &palette.fill_background),
        color_row("thumb background", &palette.thumb_background),
        color_row("thumb border", &palette.thumb_border),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}
