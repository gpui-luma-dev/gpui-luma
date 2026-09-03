use std::sync::Arc;

use luma_look_shadcn::ShadcnLook;
use luma_look_shadcn_inspect::ShadcnInspect;
use lucide_svg_static::Icon as LucideIcon;
use luma::theme::InteractionState;

use super::inspector::common::{control_size, interaction_state};
use super::inspector::input::floating_menu_palette_rows;
use super::inspector::metrics::selector_layout_section;
use super::inspector::provenance::color_row;
use super::inspector::specs::{CHOICE_SIZES, COLOR_LAYOUT_CATEGORIES};
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectorCategoryContent, InspectorPart,
    InspectorSelection, SharedInspectorResolver,
};

static SELECTOR_PARTS: [InspectorPart; 2] = [
    InspectorPart { id: "trigger", label: "Trigger", variants: &[], default_variant_id: "" },
    InspectorPart { id: "panel", label: "Popup / Items", variants: &[], default_variant_id: "" },
];

static SELECTOR_STATES: [super::inspector::InspectorStateSpec; 7] = [
    super::inspector::InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "focus",
        label: "Focus",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "open",
        label: "Open",
        icon: LucideIcon::ChevronUp,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "invalid",
        label: "Invalid",
        icon: LucideIcon::CircleAlert,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    super::inspector::InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static SELECTOR_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Selector",
    id_prefix: "selector-theme-inspector",
    parts: &SELECTOR_PARTS,
    variants: &[],
    states: &SELECTOR_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "trigger",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub struct SelectorInspectorAdapter;

impl SelectorInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for SelectorInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(selector_layout_section(
                look,
                "selector-theme-inspector-box-model",
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    if selection.part_id == "panel" {
        return floating_menu_palette_rows(
            &ShadcnInspect::new(look).inspect_floating_menu_color_palette(control_size(selection.size_id)),
        );
    }

    let palette = ShadcnInspect::new(look).inspect_selector_color_palette(
        selector_interaction_state(selection.state_id),
        control_size(selection.size_id),
    );
    vec![
        color_row("trigger background", &palette.trigger_background),
        color_row("trigger foreground", &palette.trigger_foreground),
        color_row("trigger border", &palette.trigger_border),
    ]
}

fn selector_interaction_state(state_id: &str) -> InteractionState {
    let mut state = interaction_state(state_id);
    if state_id == "focus" || state_id == "open" {
        state.focused = true;
    }
    if state_id == "open" {
        state.hovered = true;
    }
    if state_id == "invalid" {
        state.invalid = true;
    }
    state
}
