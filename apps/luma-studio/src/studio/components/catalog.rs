use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone, Copy, Debug)]
pub struct ComponentCatalogEntry {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ComponentCatalogGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: LucideIcon,
    pub entries: &'static [ComponentCatalogEntry],
}

const COMMAND_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "button", label: "Button" },
    ComponentCatalogEntry { id: "custom-button", label: "Custom Button" },
    ComponentCatalogEntry { id: "toolbar", label: "Toolbar" },
];

const CHOICE_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "accordion", label: "Accordion" },
    ComponentCatalogEntry { id: "checkbox", label: "Checkbox" },
    ComponentCatalogEntry { id: "pager", label: "Pager" },
    ComponentCatalogEntry { id: "radio-button", label: "Radio Button" },
    ComponentCatalogEntry { id: "radio-group", label: "Radio Group" },
    ComponentCatalogEntry { id: "switch", label: "Switch" },
    ComponentCatalogEntry { id: "toggle", label: "Toggle" },
    ComponentCatalogEntry { id: "toggle-group", label: "Toggle Group" },
];

const COLLECTION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "listbox", label: "ListBox" },
    ComponentCatalogEntry { id: "paging-table", label: "Paging Table" },
    ComponentCatalogEntry { id: "scrolling-table", label: "Scrolling Table" },
    ComponentCatalogEntry { id: "tree-view", label: "Tree View" },
];

const INPUT_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "scrollbar", label: "Scrollbar" },
    ComponentCatalogEntry { id: "slider", label: "Slider" },
    ComponentCatalogEntry { id: "textarea", label: "Text Area" },
    ComponentCatalogEntry { id: "textfield", label: "Text Field" },
];

const COLOR_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "color-arc", label: "Color Arc" },
    ComponentCatalogEntry { id: "color-field", label: "Color Field" },
    ComponentCatalogEntry { id: "color-ring", label: "Color Ring" },
    ComponentCatalogEntry { id: "color-slider", label: "Color Slider" },
    ComponentCatalogEntry { id: "color-slider-revealed", label: "Color Slider Revealed" },
    ComponentCatalogEntry { id: "color-multi-mixer", label: "Multi Mixer" },
];

const MENU_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "context-menu", label: "Context Menu" },
    ComponentCatalogEntry { id: "floating-menu", label: "Floating Menu" },
    ComponentCatalogEntry { id: "popup-menu", label: "Popup Menu" },
];

const LAYOUT_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "dock-panel", label: "DockPanel" },
    ComponentCatalogEntry { id: "resizable-panels", label: "Resizable Panels" },
    ComponentCatalogEntry { id: "split-view-inset", label: "Split View: Inset" },
    ComponentCatalogEntry { id: "split-view-unified", label: "Split View: Unified" },
    ComponentCatalogEntry { id: "slide-panel", label: "Slide Panel" },
];

const NAVIGATION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "sidebar", label: "Sidebar" },
    ComponentCatalogEntry { id: "tabs-navigation", label: "Tabs Navigation" },
];

const FEEDBACK_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "badge", label: "Badge" },
    ComponentCatalogEntry { id: "progress", label: "Progress" },
    ComponentCatalogEntry { id: "stepper", label: "Stepper" },
    ComponentCatalogEntry { id: "dialog-modal", label: "Dialog Modal" },
    ComponentCatalogEntry { id: "dialog-modeless", label: "Dialog Modeless" },
    ComponentCatalogEntry { id: "dialog-positioning", label: "Dialog Positioning" },
    ComponentCatalogEntry { id: "dialog-draggable", label: "Dialog Draggable" },
];

const SELECTION_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "autocomplete-textfield", label: "Autocomplete TextBox" },
    ComponentCatalogEntry { id: "combobox", label: "ComboBox" },
    ComponentCatalogEntry { id: "search-selector", label: "SearchSelector" },
    ComponentCatalogEntry { id: "popup-selector", label: "Selector" },
    ComponentCatalogEntry { id: "selection-panel", label: "Selection Panel" },
];

const PROTOTYPES_ENTRIES: &[ComponentCatalogEntry] = &[
    ComponentCatalogEntry { id: "shadow-button", label: "Shadow Button" },
    ComponentCatalogEntry { id: "split-button", label: "Split Button" },
];

/// Control catalog for the Controls tab (grouped SDK families and exposition ids).
pub const COMPONENT_CATALOG: &[ComponentCatalogGroup] = &[
    ComponentCatalogGroup { id: "command", label: "COMMAND", icon: LucideIcon::Command, entries: COMMAND_ENTRIES },
    ComponentCatalogGroup { id: "choice", label: "CHOICE", icon: LucideIcon::ListChecks, entries: CHOICE_ENTRIES },
    ComponentCatalogGroup {
        id: "collection",
        label: "COLLECTION",
        icon: LucideIcon::Table,
        entries: COLLECTION_ENTRIES,
    },
    ComponentCatalogGroup { id: "input", label: "INPUT", icon: LucideIcon::SlidersHorizontal, entries: INPUT_ENTRIES },
    ComponentCatalogGroup { id: "color", label: "COLOR", icon: LucideIcon::Palette, entries: COLOR_ENTRIES },
    ComponentCatalogGroup { id: "menu", label: "MENU", icon: LucideIcon::Menu, entries: MENU_ENTRIES },
    ComponentCatalogGroup { id: "layout", label: "LAYOUT", icon: LucideIcon::Columns2, entries: LAYOUT_ENTRIES },
    ComponentCatalogGroup {
        id: "navigation",
        label: "NAVIGATION",
        icon: LucideIcon::PanelTop,
        entries: NAVIGATION_ENTRIES,
    },
    ComponentCatalogGroup {
        id: "feedback",
        label: "FEEDBACK",
        icon: LucideIcon::MessageSquare,
        entries: FEEDBACK_ENTRIES,
    },
    ComponentCatalogGroup {
        id: "selection",
        label: "SELECTION",
        icon: LucideIcon::ListFilter,
        entries: SELECTION_ENTRIES,
    },
    ComponentCatalogGroup {
        id: "prototypes",
        label: "PROTOTYPES",
        icon: LucideIcon::Command,
        entries: PROTOTYPES_ENTRIES,
    },
];

/// Catalog columns, with each category shown once.
/// Group IDs define the category order within each column.
const COLUMN_GROUP_IDS: [&[&str]; 4] = [
    &["command", "input", "selection"],
    &["choice", "collection", "prototypes"],
    &["color", "menu"],
    &["layout", "navigation", "feedback"],
];

pub fn catalog_group(id: &str) -> Option<&'static ComponentCatalogGroup> {
    COMPONENT_CATALOG.iter().find(|group| group.id == id)
}

pub fn groups_for_column(column: usize) -> impl Iterator<Item = &'static ComponentCatalogGroup> {
    COLUMN_GROUP_IDS
        .get(column)
        .into_iter()
        .flat_map(|ids| ids.iter())
        .filter_map(|id| catalog_group(id))
}

/// Maps gallery page ids to Controls-tab exposition ids when a live doc panel exists.
pub fn controls_exposition_id(gallery_id: &str) -> Option<&'static str> {
    match gallery_id {
        "button" => Some("button"),
        "accordion" => Some("accordion"),
        "custom-button" => Some("custom-button"),
        "shadow-button" => Some("shadow-button"),
        "split-button" => Some("split-button"),
        "toolbar" => Some("toolbar"),
        "checkbox" => Some("checkbox"),
        "listbox" => Some("listbox"),
        "paging-table" => Some("paging-table"),
        "radio-button" => Some("radio-button"),
        "radio-group" => Some("radio-group"),
        "scrolling-table" => Some("scrolling-table"),
        "switch" => Some("switch"),
        "toggle" => Some("toggle"),
        "toggle-group" => Some("toggle-group"),
        "pager" => Some("pager"),
        "tree-view" => Some("tree-view"),
        "badge" => Some("badge"),
        "progress" => Some("progress"),
        "stepper" => Some("stepper"),
        "dock-panel" => Some("dock-panel"),
        "resizable-panels" => Some("resizable-panels"),
        "sidebar" => Some("sidebar"),
        "tabs-navigation" => Some("tabs-navigation"),
        "slide-panel" => Some("slide-panel"),
        "textfield" => Some("textfield"),
        "textarea" => Some("textarea"),
        "slider" => Some("slider"),
        "scrollbar" => Some("scrollbar"),
        "color-slider" => Some("color-slider"),
        "color-field" => Some("color-field"),
        "color-ring" => Some("color-ring"),
        "color-arc" => Some("color-arc"),
        "color-multi-mixer" => Some("color-multi-mixer"),
        "color-slider-revealed" => Some("color-slider-revealed"),
        "dialog-modal" => Some("modal-overlay"),
        "dialog-modeless" => Some("modeless-overlay"),
        "dialog-positioning" => Some("overlay-positioning"),
        "dialog-draggable" => Some("draggable-overlay"),
        "context-menu" => Some("context-menu"),
        "floating-menu" => Some("floating-menu"),
        "popup-menu" => Some("popup-menu"),
        "autocomplete-textfield" => Some("autocomplete-textfield"),
        "combobox" => Some("combobox"),
        "search-selector" => Some("search-selector"),
        "popup-selector" => Some("popup-selector"),
        "selection-panel" => Some("selection-panel"),
        _ => None,
    }
}

/// First gallery catalog entry that has a Controls-tab exposition, in gallery order.
pub fn first_controls_exposition_id() -> Option<&'static str> {
    COMPONENT_CATALOG
        .iter()
        .flat_map(|group| group.entries.iter())
        .find_map(|entry| controls_exposition_id(entry.id))
}
