use std::sync::Arc;

use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use lucide_icons::Icon as LucideIcon;

use super::inspector::common::{button_variant_style, control_size, interaction_state};
use super::inspector::metrics::button_layout_section;
use super::inspector::provenance::{color_row, elevation_snapshot};
use super::inspector::specs::CHOICE_SIZES;
use super::inspector::{
    ControlInspectorResolver, ControlInspectorSpec, InspectColorRow, InspectPropertyRow, InspectorCategory,
    InspectorCategoryContent, InspectorSelection, InspectorStateSpec, InspectorVariant, SharedInspectorResolver,
};

const BUTTON_ROLE: ButtonFamilyRole = ButtonFamilyRole::Text;

static BUTTON_VARIANTS: [InspectorVariant; 4] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
];

static BUTTON_CATEGORIES: [InspectorCategory; 4] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "elevation", label: "Elevation", icon: LucideIcon::Layers, expanded_default: true },
    InspectorCategory { id: "typography", label: "Typography", icon: LucideIcon::Type, expanded_default: true },
];

static BUTTON_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &BUTTON_CATEGORIES,
    },
];

pub static BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Button",
    id_prefix: "button-theme-inspector",
    parts: &[],
    variants: &BUTTON_VARIANTS,
    states: &BUTTON_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "primary",
    default_size_id: "md",
    default_value_id: "",
};

pub struct ButtonInspectorAdapter;

impl ButtonInspectorAdapter {
    pub fn shared() -> SharedInspectorResolver {
        Arc::new(Self)
    }
}

impl ControlInspectorResolver for ButtonInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look, selection)),
            "layout" => InspectorCategoryContent::Layout(resolve_layout_section(look, selection)),
            "elevation" => InspectorCategoryContent::Elevation(resolve_elevation(look, selection)),
            "typography" => InspectorCategoryContent::Typography(resolve_typography_rows(look)),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }

    fn category_applies(&self, _look: &ShadcnLook, selection: InspectorSelection<'_>, category_id: &str) -> bool {
        if category_id == "elevation" {
            return super::inspector::common::button_elevation_applies(selection.variant_id, selection.state_id);
        }
        true
    }
}

fn resolve_color_rows(look: &ShadcnLook, selection: InspectorSelection<'_>) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(
        button_variant_style(selection.variant_id),
        BUTTON_ROLE,
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
    button_layout_section(
        look,
        "button-theme-inspector-box-model",
        button_variant_style(selection.variant_id),
        BUTTON_ROLE,
        control_size(selection.size_id),
        interaction_state(selection.state_id),
    )
}

fn resolve_elevation(
    look: &ShadcnLook,
    selection: InspectorSelection<'_>,
) -> super::inspector::InspectElevationSnapshot {
    let elevation = ShadcnInspect::new(look).inspect_button_elevation(
        button_variant_style(selection.variant_id),
        BUTTON_ROLE,
        interaction_state(selection.state_id),
    );
    elevation_snapshot(&elevation, "resolved button look", "button.elevation_rules[].style")
}

fn resolve_typography_rows(look: &ShadcnLook) -> Vec<InspectPropertyRow> {
    let typography = ShadcnInspect::new(look).inspect_button_typography();
    vec![
        InspectPropertyRow::new(
            "font family",
            typography.font_family.value.as_str(),
            "typography.text.body.font_family",
        ),
        InspectPropertyRow::new("font size", typography.font_size.value.as_str(), "typography.text.body.size"),
        InspectPropertyRow::new("font weight", typography.font_weight.value.as_str(), "typography.text.body.weight"),
        InspectPropertyRow::new(
            "line height",
            typography.line_height.value.as_str(),
            "typography.text.body.line_height",
        ),
    ]
}
