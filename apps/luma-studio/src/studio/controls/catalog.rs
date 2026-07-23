#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlCategory {
    Command,
    Choice,
    Inputs,
    Selection,
    NavigationPanels,
    OverlaysDialogs,
}

impl ControlCategory {
    pub const ALL: [Self; 6] = [
        Self::Command,
        Self::Choice,
        Self::Inputs,
        Self::Selection,
        Self::NavigationPanels,
        Self::OverlaysDialogs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::Choice => "Choice",
            Self::Inputs => "Inputs",
            Self::Selection => "Selection",
            Self::NavigationPanels => "Navigation & Panels",
            Self::OverlaysDialogs => "Overlays & Dialogs",
        }
    }

    pub fn index_order(self) -> usize {
        match self {
            Self::Command => 0,
            Self::Choice => 1,
            Self::Inputs => 2,
            Self::Selection => 3,
            Self::NavigationPanels => 4,
            Self::OverlaysDialogs => 5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ControlDocEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
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
        id: "textfield",
        title: "Text Field",
        description: "Single-line text entry for labels, search, and form values. Supports placeholder text, full-width layout, compact density, and look overrides for monospace or token-style fields. Focus, selection, and clipboard behavior live in the SDK editing engine; the parent owns the string value and updates the field programmatically when model data changes.",
        category: ControlCategory::Inputs,
        snippet: "look\n    .textfield(\"email\")\n    .placeholder(\"Email\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 200,
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

pub fn entries_for_category(category: ControlCategory) -> impl Iterator<Item = &'static ControlDocEntry> {
    CONTROL_CATALOG.iter().filter(move |entry| entry.category == category)
}

pub fn catalog_entry(id: &str) -> Option<&'static ControlDocEntry> {
    CONTROL_CATALOG.iter().find(|entry| entry.id == id)
}
