use lucide_icons::Icon as LucideIcon;

use super::schema::{InspectorCategory, InspectorSize, InspectorStateSpec, InspectorValueMode, InspectorVariant};

pub static PRIMARY_SECONDARY_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
];

pub static PRIMARY_SECONDARY_CONTENT_ONLY_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "content-only", label: "Content Only" },
];

pub static CHOICE_SIZES: [InspectorSize; 3] = [
    InspectorSize { id: "sm", label: "sm" },
    InspectorSize { id: "md", label: "md" },
    InspectorSize { id: "lg", label: "lg" },
];

pub static COLOR_LAYOUT_CATEGORIES: [InspectorCategory; 2] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
];

pub static COLOR_LAYOUT_ELEVATION_CATEGORIES: [InspectorCategory; 3] = [
    InspectorCategory { id: "color", label: "Color", icon: LucideIcon::Palette, expanded_default: true },
    InspectorCategory { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded_default: true },
    InspectorCategory { id: "elevation", label: "Elevation", icon: LucideIcon::Layers, expanded_default: true },
];

pub static COLOR_LAYOUT_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static CHOICE_INTERACTION_STATES: [InspectorStateSpec; 5] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focused",
        label: "Focused",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "pressed",
        label: "Pressed",
        icon: LucideIcon::MousePointerClick,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
];

pub static SELECTED_UNSELECTED_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "selected", label: "selected" },
    InspectorValueMode { id: "unselected", label: "unselected" },
];

pub static CHECKED_UNCHECKED_VALUES: [InspectorValueMode; 2] = [
    InspectorValueMode { id: "checked", label: "checked" },
    InspectorValueMode { id: "unchecked", label: "unchecked" },
];

pub static ON_OFF_VALUES: [InspectorValueMode; 2] =
    [InspectorValueMode { id: "on", label: "on" }, InspectorValueMode { id: "off", label: "off" }];

pub static TEXTFIELD_VARIANTS: [InspectorVariant; 3] = [
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "primary", label: "Primary" },
    InspectorVariant { id: "surface", label: "Surface" },
];

pub static TEXTFIELD_INTERACTION_STATES: [InspectorStateSpec; 4] = [
    InspectorStateSpec {
        id: "default",
        label: "Default",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "hover",
        label: "Hover",
        icon: LucideIcon::MousePointer2,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "focus",
        label: "Focus",
        icon: LucideIcon::Focus,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_ELEVATION_CATEGORIES,
    },
];

pub static DEFAULT_INTERACTION_STATES: [InspectorStateSpec; 1] = [InspectorStateSpec {
    id: "default",
    label: "Default",
    icon: LucideIcon::Circle,
    expanded_default: true,
    categories: &COLOR_LAYOUT_CATEGORIES,
}];

pub static BADGE_VARIANTS: [InspectorVariant; 4] = [
    InspectorVariant { id: "default", label: "Default" },
    InspectorVariant { id: "secondary", label: "Secondary" },
    InspectorVariant { id: "outline", label: "Outline" },
    InspectorVariant { id: "ghost", label: "Ghost" },
];

pub static PROGRESS_STATES: [InspectorStateSpec; 2] = [
    InspectorStateSpec {
        id: "enabled",
        label: "Enabled",
        icon: LucideIcon::Circle,
        expanded_default: true,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
    InspectorStateSpec {
        id: "disabled",
        label: "Disabled",
        icon: LucideIcon::CircleOff,
        expanded_default: false,
        categories: &COLOR_LAYOUT_CATEGORIES,
    },
];

pub static SCROLLBAR_ORIENTATIONS: [InspectorVariant; 2] = [
    InspectorVariant { id: "horizontal", label: "Horizontal" },
    InspectorVariant { id: "vertical", label: "Vertical" },
];

pub static POPUP_MENU_TRIGGER_VARIANTS: [InspectorVariant; 2] = [
    InspectorVariant { id: "outline", label: "Outline Trigger" },
    InspectorVariant { id: "ghost", label: "Ghost Trigger" },
];
