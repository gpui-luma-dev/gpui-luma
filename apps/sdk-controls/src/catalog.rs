//! Control labels and ordering mirror docs/navigation.js. Unimplemented entries stay visible as deferred.
pub const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "Controls",
        &[
            ("accordion", "Accordion"),
            ("autocomplete", "Autocomplete"),
            ("button", "Button"),
            ("checkbox", "Checkbox"),
            ("combobox", "ComboBox"),
            ("context-menu", "Context Menu"),
            ("control-group", "Control Group"),
            ("icon-button", "Icon Button"),
            ("icon-group", "Icon Group"),
            ("listbox", "List Box"),
            ("overlay-window", "Overlay Window"),
            ("pager", "Pager"),
            ("popover-button", "Popover Button"),
            ("popup-menu", "Popup Menu"),
            ("progress", "Progress"),
            ("radio-button", "Radio Button"),
            ("radio-group", "Radio Group"),
            ("scrollbar", "Scrollbar"),
            ("search-selector", "Search Selector"),
            ("selection-panel", "Selection Panel"),
            ("selector", "Selector"),
            ("sidebar", "Sidebar"),
            ("slide-panel", "Slide Panel"),
            ("slider", "Slider"),
            ("split-button", "Split Button"),
            ("stepper", "Stepper"),
            ("switch", "Switch"),
            ("table", "Table"),
            ("tabs", "Tabs"),
            ("textarea", "Text Area"),
            ("textfield", "Text Field"),
            ("toggle", "Toggle"),
            ("toolbar", "Toolbar"),
            ("tooltip", "Tooltip"),
            ("tree-view", "Tree View"),
        ],
    ),
    (
        "Layout",
        &[
            ("frame", "Frame"),
            ("scroll-container", "Scroll Container"),
            ("dock-splitter", "Dock Splitter"),
            ("resizable-panels", "Resizable Panels"),
            ("split-view", "Split View"),
        ],
    ),
    (
        "Color controls",
        &[
            ("color-slider", "Color Slider"),
            ("color-field", "Color Field"),
            ("color-ring", "Color Ring"),
            ("color-arc", "Color Arc"),
            ("color-swatch", "Color Swatch"),
        ],
    ),
    (
        "Composition & helpers",
        &[
            ("floating-menu", "Floating Menu"),
            ("popup-scroll-surface", "Popup Scroll Surface"),
            ("selector-list", "Selector List"),
        ],
    ),
];
pub fn label(id: &str) -> Option<&'static str> {
    GROUPS.iter().flat_map(|(_, items)| *items).find(|(key, _)| *key == id).map(|(_, label)| *label)
}
