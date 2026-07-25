#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ControlCategory {
    Command,
    Choice,
    Inputs,
    Color,
    Selection,
    NavigationPanels,
    OverlaysDialogs,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlDocEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    #[allow(dead_code)]
    pub category: ControlCategory,
    pub snippet: &'static str,
    pub section_order: usize,
}

pub const CONTROL_CATALOG: &[ControlDocEntry] = &[
    ControlDocEntry {
        id: "button",
        title: "Button",
        description: "Command buttons commit actions in forms, dialogs, and toolbars. The look exposes primary, secondary, outline, and ghost builders that map to Shadcn emphasis tiers—use primary for the main call to action and ghost for low-priority utilities. Each button is a GPUI entity: spawn once per id, filter ButtonEvent::Click for activation, and optionally observe focus, hover, and enabled transitions.",
        category: ControlCategory::Command,
        snippet: "let save = look.primary_button(\"save\").label(\"Save\").spawn(cx);\n\ncx.subscribe(&save, |_, _, event: &ButtonEvent, _| {\n    match event {\n        ButtonEvent::Click => { /* commit action */ }\n        ButtonEvent::FocusChanged { focused } => { let _ = focused; }\n        ButtonEvent::HoverChanged { hovered } => { let _ = hovered; }\n        ButtonEvent::EnabledChanged { enabled } => { let _ = enabled; }\n        _ => {}\n    }\n});",
        section_order: 100,
    },
    ControlDocEntry {
        id: "checkbox",
        title: "Checkbox",
        description: "Boolean choice controls for multi-select forms and settings. Primary and secondary builders map to Shadcn emphasis tiers; add a content closure for labeled checkboxes or omit it for indicator-only rows. Each checkbox is a GPUI entity—subscribe to CheckboxEvent::Change and assign local state from the payload; use set_data for programmatic sync.",
        category: ControlCategory::Choice,
        snippet: "let terms = look\n    .primary_checkbox(\"terms\")\n    .with_data(false)\n    .content(|_, _| div().child(\"Accept terms\").into_any_element())\n    .spawn(cx);\n\ncx.subscribe(&terms, |_, _, event: &CheckboxEvent, _| {\n    if let CheckboxEvent::Change { checked } = event {\n        let _ = checked;\n    }\n});",
        section_order: 110,
    },
    ControlDocEntry {
        id: "radio-button",
        title: "Radio Button",
        description: "Single-select choice within a mutually exclusive set. Standalone radio buttons emit RadioButtonEvent::Change when selected; coordinate sibling deselection in the parent or use a radio group for managed value binding. Primary and secondary builders support indicator-only or labeled content layouts.",
        category: ControlCategory::Choice,
        snippet: "let option_a = look\n    .primary_radio(\"option-a\")\n    .with_data(false)\n    .content(|_, _| div().child(\"Option A\").into_any_element())\n    .spawn(cx);\n\ncx.subscribe(&option_a, |_, _, event: &RadioButtonEvent, _| {\n    if let RadioButtonEvent::Change { selected } = event {\n        let _ = selected;\n    }\n});",
        section_order: 120,
    },
    ControlDocEntry {
        id: "switch",
        title: "Switch",
        description: "On/off toggles for settings and feature flags. Switches use track-and-thumb presentation with primary and secondary emphasis tiers; optional content closures add inline labels. Subscribe to SwitchEvent::Change for user toggles and set_data for programmatic updates.",
        category: ControlCategory::Choice,
        snippet: "let notifications = look\n    .primary_switch(\"notifications\")\n    .with_data(true)\n    .spawn(cx);\n\ncx.subscribe(&notifications, |_, _, event: &SwitchEvent, _| {\n    if let SwitchEvent::Change { on } = event {\n        let _ = on;\n    }\n});",
        section_order: 130,
    },
    ControlDocEntry {
        id: "textfield",
        title: "Text Field",
        description: "Single-line text entry for labels, search, and form values. Supports placeholder text, full-width layout, compact density, and look overrides for monospace or token-style fields. Focus, selection, and clipboard behavior live in the SDK editing engine; the parent owns the string value and updates the field programmatically when model data changes.",
        category: ControlCategory::Inputs,
        snippet: "look\n    .textfield(\"email\")\n    .placeholder(\"Email\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 200,
    },
    ControlDocEntry {
        id: "color-slider",
        title: "Color Slider",
        description: "Spectrum sliders built on the unified Slider engine with color-specific delegates for hue, saturation, alpha, channel, and gradient interpolation. ColorSliderBuilder returns Slider entities—subscribe to SliderEvent::Change for live preview and SliderEvent::Release to commit values. Linked saturation and alpha tracks refresh when the base color changes.",
        category: ControlCategory::Color,
        snippet: "ColorSliderBuilder::hue(\"accent-hue\", 210.0)\n    .size(ControlSize::Sm)\n    .thumb_medium()\n    .spawn(cx);\n\ncx.subscribe(&hue_slider, |_, _, event: &SliderEvent, _| {\n    if let SliderEvent::Release { value, .. } = event {\n        let _ = value;\n    }\n});",
        section_order: 210,
    },
    ControlDocEntry {
        id: "modal-overlay",
        title: "Modal Overlay",
        description: "Modal overlay windows block the workspace for confirm/cancel or short-form tasks. They trap focus, dim the background, and restore focus to the opener on dismiss. Compose header, body, and footer in the content closure; register persistent child controls with theme_children so token changes refresh without app-level notify plumbing.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"confirm-archive\")\n    .mode(OverlayWindowMode::Modal)\n    .content(|_, _, _| { /* body */ })\n    .theme_children([cancel.clone(), confirm.clone()])\n    .spawn(cx);",
        section_order: 300,
    },
];

pub fn catalog_entry(id: &str) -> Option<&'static ControlDocEntry> {
    CONTROL_CATALOG.iter().find(|entry| entry.id == id)
}
