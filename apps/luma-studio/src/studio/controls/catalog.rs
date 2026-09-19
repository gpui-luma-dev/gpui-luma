#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ControlCategory {
    Command,
    Choice,
    Inputs,
    Color,
    ColorCompositions,
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
        snippet: "let save = shadcn::Button::new(\"save\").look(look.as_ref()).primary().label(\"Save\").spawn(cx);\n\ncx.subscribe(&save, |_, _, event: &ButtonEvent, _| {\n    match event {\n        ButtonEvent::Click => { /* commit action */ }\n        ButtonEvent::FocusChanged { focused } => { let _ = focused; }\n        ButtonEvent::HoverChanged { hovered } => { let _ = hovered; }\n        ButtonEvent::EnabledChanged { enabled } => { let _ = enabled; }\n        _ => {}\n    }\n});",
        section_order: 100,
    },
    ControlDocEntry {
        id: "split-button",
        title: "Split Button (Prototype)",
        description: "Issue #17 prototype for a cohesive command button with a separate menu trigger. The action face executes immediately while the adjacent face opens alternate actions; this Studio surface is validating focus, seams, menu ownership, and event semantics before SDK promotion.",
        category: ControlCategory::Command,
        snippet: "use luma::infra::menu_item::MenuItem;\nuse luma::controls::split_button::SplitButtonEvent;\n\nlet save = shadcn::SplitButton::new(\"save\").look(look.as_ref()).primary()\n    .label(\"Save\")\n    .items([\n        MenuItem::new(\"save-as\").label(\"Save as\"),\n        MenuItem::new(\"duplicate\").label(\"Duplicate\"),\n    ])\n    .spawn(cx);\n\ncx.subscribe(&save, |_, _, event: &SplitButtonEvent, _| {\n    match event {\n        SplitButtonEvent::ActionClick => { /* save immediately */ }\n        SplitButtonEvent::Select { item_id, .. } => { let _ = item_id; }\n        _ => {}\n    }\n});",
        section_order: 101,
    },
    ControlDocEntry {
        id: "toolbar",
        title: "Toolbar",
        description: "Horizontal control_group hosting buttons, toggles, selectors, menus, and text fields. ShadcnToolbarItemExt factories compose editing toolbars with roving or sequential keyboard focus.",
        category: ControlCategory::Command,
        snippet: "shadcn::Toolbar::new(\"editor\").look(look.as_ref())\n    .item(look.toolbar_toggle(\"bold\", LucideIcon::Bold, cx))\n    .item(look.toolbar_button(\"link\", LucideIcon::Link, cx))\n    .spawn(cx);",
        section_order: 102,
    },
    ControlDocEntry {
        id: "custom-button",
        title: "Custom Button",
        description: "Advanced Button architecture — custom templates with layered modifiers, free-form content presenters, typed reactive data, and factory icon/text layouts beyond Shadcn presets.",
        category: ControlCategory::Command,
        snippet: "Button::new(\"custom\")\n    .content(|model, _| { /* layout */ })\n    .template(custom_template())\n    .spawn(cx);",
        section_order: 105,
    },
    ControlDocEntry {
        id: "shadow-button",
        title: "Shadow Button",
        description: "Gallery prototype for a CSS-style box shadow wrapped around the primary button template. Tune offset, blur, spread, and opacity live; preview default, hover, pressed, focus, and disabled states.",
        category: ControlCategory::Command,
        snippet: "Button::new(\"fab\")\n    .label(\"Floating Action\")\n    .template(prototype_shadow_button_template(look, controls, color))\n    .spawn(cx);",
        section_order: 106,
    },
    ControlDocEntry {
        id: "checkbox",
        title: "Checkbox",
        description: "Boolean choice controls for multi-select forms and settings. Primary and secondary builders map to Shadcn emphasis tiers; add a content closure for labeled checkboxes or omit it for indicator-only rows. Each checkbox is a GPUI entity—subscribe to CheckboxEvent::Change and assign local state from the payload; use set_data for programmatic sync.",
        category: ControlCategory::Choice,
        snippet: "let terms = shadcn::Checkbox::new(\"terms\").look(look.as_ref()).primary()\n    .with_data(false)\n    .content(|_, _| div().child(\"Accept terms\").into_any_element())\n    .spawn(cx);\n\ncx.subscribe(&terms, |_, _, event: &CheckboxEvent, _| {\n    if let CheckboxEvent::Change { checked } = event {\n        let _ = checked;\n    }\n});",
        section_order: 110,
    },
    ControlDocEntry {
        id: "radio-button",
        title: "Radio Button",
        description: "Single-select choice within a mutually exclusive set. Standalone radio buttons emit RadioButtonEvent::Change when selected; coordinate sibling deselection in the parent or use a radio group for managed value binding. Primary and secondary builders support indicator-only or labeled content layouts.",
        category: ControlCategory::Choice,
        snippet: "let option_a = shadcn::Radio::new(\"option-a\").look(look.as_ref()).primary()\n    .with_data(false)\n    .content(|_, _| div().child(\"Option A\").into_any_element())\n    .spawn(cx);\n\ncx.subscribe(&option_a, |_, _, event: &RadioButtonEvent, _| {\n    if let RadioButtonEvent::Change { selected } = event {\n        let _ = selected;\n    }\n});",
        section_order: 120,
    },
    ControlDocEntry {
        id: "switch",
        title: "Switch",
        description: "On/off toggles for settings and feature flags. Switches use track-and-thumb presentation with primary and secondary emphasis tiers; optional content closures add inline labels. Subscribe to SwitchEvent::Change for user toggles and set_data for programmatic updates.",
        category: ControlCategory::Choice,
        snippet: "let notifications = shadcn::Switch::new(\"notifications\").look(look.as_ref()).primary()\n    .with_data(true)\n    .spawn(cx);\n\ncx.subscribe(&notifications, |_, _, event: &SwitchEvent, _| {\n    if let SwitchEvent::Change { on } = event {\n        let _ = on;\n    }\n});",
        section_order: 130,
    },
    ControlDocEntry {
        id: "radio-group",
        title: "Radio Group",
        description: "Single-selection groups with vertical, horizontal, custom indent, and card-style item layouts. Built on control_group with RadioGroupEvent::Change for value binding.",
        category: ControlCategory::Choice,
        snippet: "shadcn::RadioGroup::new(\"density\").look(look.as_ref())\n    .items(items)\n    .selected(\"comfortable\")\n    .spawn(cx);",
        section_order: 121,
    },
    ControlDocEntry {
        id: "toggle",
        title: "Toggle",
        description: "Boolean selection controls using button-family toggle chrome. Primary and secondary builders support text labels or round icon layouts.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Toggle::new(\"bold\").look(look.as_ref()).secondary()\n    .with_data(false)\n    .content(|_, _| div().child(\"Bold\").into_any_element())\n    .spawn(cx);",
        section_order: 131,
    },
    ControlDocEntry {
        id: "toggle-group",
        title: "Toggle Group",
        description: "Icon toggle button clusters for toolbar-style single or multi selection. Uses control_group with toggle_button_item_template and custom pill shell layouts.",
        category: ControlCategory::Choice,
        snippet: "shadcn::IconGroup::new(\"placement\").look(look.as_ref())\n    .items(items)\n    .selected(\"bottom\")\n    .spawn(cx);",
        section_order: 132,
    },
    ControlDocEntry {
        id: "pager",
        title: "Pager",
        description: "Standalone pagination control with Minimal, MinimalEdge, and Numeric styles. Derives page count from total items and page size; emits PageChanged and PageSizeChanged.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Pager::new(\"results\").look(look.as_ref())\n    .style(PagerStyle::Numeric)\n    .page_count(12)\n    .current_page(0)\n    .spawn(cx);",
        section_order: 133,
    },
    ControlDocEntry {
        id: "tree-view",
        title: "Tree View",
        description: "Virtualized hierarchical list for file explorers and nested navigation. Supports expand/collapse, single selection, and keyboard traversal.",
        category: ControlCategory::Choice,
        snippet: "shadcn::TreeView::new(\"files\").look(look.as_ref())\n    .items(tree_nodes)\n    .selection_mode(TreeViewSelectionMode::Single)\n    .spawn(cx);",
        section_order: 134,
    },
    ControlDocEntry {
        id: "badge",
        title: "Badge",
        description: "Look-specific inline status element with variant colors, shared metric scaling, and optional start/end Lucide icons. Compose at render time via look.badge — no entity lifecycle.",
        category: ControlCategory::Command,
        snippet: "look.badge(\"Verified\")\n    .variant(BadgeVariant::Default)\n    .start_icon(LucideIcon::BadgeCheck);",
        section_order: 135,
    },
    ControlDocEntry {
        id: "progress",
        title: "Progress",
        description: "Read-only progress indicator with circular ring and linear bar templates. Configure direction, optional thumb, and update value programmatically via set_value.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Progress::linear(\"upload\").look(look.as_ref())\n    .range(0..100)\n    .value(41)\n    .direction(ProgressDirection::LeftToRight)\n    .show_thumb(true)\n    .spawn(cx);",
        section_order: 136,
    },
    ControlDocEntry {
        id: "stepper",
        title: "Stepper",
        description: "Discrete multi-step process indicator with complete, in-progress, and incomplete step badges plus connector tracks. Configure step count, current step, optional labels, and layout direction.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Stepper::new(\"checkout\", 4).look(look.as_ref())\n    .current_step(1)\n    .labels(vec![\"Account\", \"Shipping\", \"Payment\", \"Review\"])\n    .direction(ProgressDirection::LeftToRight)\n    .spawn(cx);",
        section_order: 137,
    },
    ControlDocEntry {
        id: "accordion",
        title: "Accordion",
        description: "Collapsible sections with single or multiple expansion modes. Triggers support icons, disabled items, and custom content closures for interactive panels.",
        category: ControlCategory::Choice,
        snippet: "shadcn::Accordion::new(\"settings\").look(look.as_ref())\n    .single()\n    .item(AccordionItem::new(\"general\", trigger, content))\n    .spawn(cx);",
        section_order: 138,
    },
    ControlDocEntry {
        id: "listbox",
        title: "ListBox",
        description: "Keyed collection state with host-composed vertical rows and horizontal cards. Independent multi-select examples, keyboard navigation, and scoped composition inspectors.",
        category: ControlCategory::Selection,
        snippet: "let state = ListBoxState::try_new(items, |item| item.id, SelectionMode::Multiple)?;\n// Compose state.visible_items() with vstack! or hstack!.\n// Attach ListBoxBinding for focus, selection, and keyboard input.",
        section_order: 305,
    },
    ControlDocEntry {
        id: "scrolling-table",
        title: "Scrolling Table",
        description: "Virtualized task grid with fixed viewport, column templates, row snap scrolling, and keyboard selection. Ideal for long datasets that scroll in place.",
        category: ControlCategory::Choice,
        snippet: "scrolling_table! {\n    table_theme = look.table_theme();\n    id = \"tasks\";\n    items = rows;\n    visible_rows = 25;\n    scroll_snap = true;\n    grid_view = { /* columns */ };\n}.spawn(cx);",
        section_order: 140,
    },
    ControlDocEntry {
        id: "paging-table",
        title: "Paging Table",
        description: "Grid table with embedded pager toolbar. Page size and pager style stay wired to Table page commands for prev/next navigation.",
        category: ControlCategory::Choice,
        snippet: "paging_table! {\n    table_theme = look.table_theme();\n    id = \"tasks\";\n    items = rows;\n    page_size = 10;\n    pager = shadcn::Pager::new(\"pager\").look(look.as_ref()).style(PagerStyle::MinimalEdge).into_sdk_builder(cx);\n    grid_view = { /* columns */ };\n}.spawn(cx);",
        section_order: 140,
    },
    ControlDocEntry {
        id: "textfield",
        title: "Text Field",
        description: "Single-line text entry for labels, search, and form values. Supports placeholder text, full-width layout, compact density, and look overrides for monospace or token-style fields. Focus, selection, and clipboard behavior live in the SDK editing engine; the parent owns the string value and updates the field programmatically when model data changes.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::TextField::new(\"email\").look(look.as_ref())\n    .placeholder(\"Email\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 200,
    },
    ControlDocEntry {
        id: "textarea",
        title: "Text Area",
        description: "Multiline text input with hard-line editing, selection, escape-clear policy, optional validation, and fixed row height. Gallery-aligned demo includes option toggles and variant state previews.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::TextArea::new(\"notes\").look(look.as_ref())\n    .placeholder(\"Write a message…\")\n    .rows(6)\n    .full_width(true)\n    .spawn(cx);",
        section_order: 201,
    },
    ControlDocEntry {
        id: "slider",
        title: "Slider",
        description: "Unified linear and angular slider engine — fill tracks, vertical and reversed orientation, blocked intervals, multi-stop thumbs, dials, and size variants. Subscribe to SliderEvent::Change for live preview and SliderEvent::Release to commit.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Slider::new(\"volume\").look(look.as_ref())\n    .range(0..100)\n    .step(1)\n    .value(50)\n    .spawn(cx);",
        section_order: 202,
    },
    ControlDocEntry {
        id: "scrollbar",
        title: "Scrollbar",
        description: "Standalone scroll thumb and track control for custom scroll surfaces. Supports horizontal and vertical orientation, step/page step, and thumb fraction sizing. Pair with a clipped content viewport for classic scroll UI.",
        category: ControlCategory::Inputs,
        snippet: "shadcn::Scrollbar::new(\"list-scroll\").look(look.as_ref())\n    .vertical()\n    .range(0..500)\n    .step(20)\n    .value(0)\n    .spawn(cx);",
        section_order: 203,
    },
    ControlDocEntry {
        id: "context-menu",
        title: "Context Menu",
        description: "Right-click targets that open anchored floating menus. Compose MenuItem trees with icons and nested submenus; subscribe to ContextMenuEvent::Select for the chosen action. Shadcn look binds themed target chrome and floating menu tokens.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "shadcn::ContextMenu::new(\"ctx\").look(look.as_ref())\n    .label(\"Right-click me\")\n    .items(menu_items)\n    .spawn(cx);",
        section_order: 220,
    },
    ControlDocEntry {
        id: "floating-menu",
        title: "Floating Menu",
        description: "Template primitive for menu item lists — shared by popup menus, context menus, and navigation submenus. render_floating_menu accepts item hover/click handler slots; FloatingMenuState drives keyboard navigation and submenu paths.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "render_floating_menu(&id, &items, open_submenu, active_path, look, hovers, clicks);",
        section_order: 221,
    },
    ControlDocEntry {
        id: "popup-menu",
        title: "Popup Menu",
        description: "Labeled triggers that open anchored popup menus with smart or fixed placement. Supports ghost triggers, submenu nesting, and disabled items. Subscribe to PopupMenuEvent::Select for activation.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "shadcn::PopupMenu::new(\"menu\").look(look.as_ref())\n    .label(\"Actions\")\n    .items(menu_items)\n    .placement(PopupMenuPlacement::Smart)\n    .spawn(cx);",
        section_order: 222,
    },
    ControlDocEntry {
        id: "autocomplete-textfield",
        title: "Autocomplete TextBox",
        description: "Single-line text field with typeahead popup filtering. Emits Change while typing and Select or Complete when an item is committed. Ideal for token fields, address lookup, and constrained free text.",
        category: ControlCategory::Selection,
        snippet: "shadcn::Autocomplete::new(\"city\", items).look(look.as_ref())\n    .placeholder(\"Start typing…\")\n    .full_width(true)\n    .spawn(cx);",
        section_order: 230,
    },
    ControlDocEntry {
        id: "combobox",
        title: "ComboBox",
        description: "Editable text field with anchored item popup — strict or permissive typing policies, down-arrow and clear affordances, and second-tier list/panel templates for custom row chrome.",
        category: ControlCategory::Selection,
        snippet: "shadcn::ComboBox::new(\"state\", items).look(look.as_ref())\n    .typing_policy(TypingPolicy::Strict)\n    .show_down_arrow(true)\n    .spawn(cx);",
        section_order: 231,
    },
    ControlDocEntry {
        id: "search-selector",
        title: "SearchSelector",
        description: "Read-only trigger with a popup search field and scrollable item list. Separates trigger label from in-popup filtering — useful when the committed value should stay compact while search is expansive.",
        category: ControlCategory::Selection,
        snippet: "shadcn::SearchSelector::new(\"state\", items).look(look.as_ref())\n    .placeholder(\"Choose…\")\n    .search_placeholder(\"Search\")\n    .spawn(cx);",
        section_order: 232,
    },
    ControlDocEntry {
        id: "popup-selector",
        title: "Selector",
        description: "Labeled triggers with anchored selector popups — placement strategies include smart flip, below/above start, and overlay-on-trigger. Custom item templates support swatches, icons, and two-line rows.",
        category: ControlCategory::Selection,
        snippet: "shadcn::Selector::new(\"status\").look(look.as_ref())\n    .label(\"Select status\")\n    .items(selector_items)\n    .placement(SelectorPlacement::Smart)\n    .spawn(cx);",
        section_order: 233,
    },
    ControlDocEntry {
        id: "selection-panel",
        title: "Selection Panel",
        description: "Scrollable selectable item panels with keyboard navigation, row hover/active states, and template hooks for custom item and panel chrome. Emits ActivateRow, ActiveIndexChanged, and HoverChanged for list-driven workflows.",
        category: ControlCategory::Selection,
        snippet: "shadcn::SelectionPanel::new(\"actions\").look(look.as_ref())\n    .items(items)\n    .scrolling(true)\n    .spawn(cx);",
        section_order: 234,
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
        id: "color-field",
        title: "Color Field",
        description: "2D color selection surfaces for saturation/value, hue/value, triangle, and wheel domains. Supports vector rendering and prewarmed raster image fields.",
        category: ControlCategory::Color,
        snippet: "cx.new(|_| {\n    ColorFieldState::saturation_value_rect(\"sv\", hsv, thumb_size)\n        .vector()\n        .rounded(px(12.0))\n});",
        section_order: 211,
    },
    ControlDocEntry {
        id: "color-ring",
        title: "Color Ring",
        description: "Circular hue, saturation, and lightness sliders built on the unified slider engine with ring-specific delegates. Vector and raster rendering paths available.",
        category: ControlCategory::Color,
        snippet: "ColorRingBuilder::hue(\"ring\", h, s, l)\n    .size(Size::Medium)\n    .allow_inner_target(true)\n    .spawn(cx);",
        section_order: 212,
    },
    ControlDocEntry {
        id: "color-arc",
        title: "Color Arc",
        description: "Partial-ring color sliders with configurable sweep angles and track thickness. Hue, saturation, and lightness delegates share arc geometry helpers.",
        category: ControlCategory::Color,
        snippet: "ColorArcBuilder::hue(\"arc\", h, s, l)\n    .size(Size::Medium)\n    .start_degrees(-45.0)\n    .sweep_degrees(270.0)\n    .spawn(cx);",
        section_order: 213,
    },
    ControlDocEntry {
        id: "color-picker",
        title: "Color Picker",
        description: "Photoshop-style composed picker — saturation/value field with linked hue and alpha sliders. ColorCompositionSync keeps controls in sync without feedback loops. CompositionSize scales control width and swatch height.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorFieldState::saturation_value(\"sv\", hsv, thumb_size)\n    .vector()\n    .rounded(px(3.0));\n\nColorSliderBuilder::hue(\"hue\", hsv.h).spawn(cx);\nColorSliderBuilder::alpha(\"alpha\", hsv.a, hsv).spawn(cx);",
        section_order: 214,
    },
    ControlDocEntry {
        id: "color-hsv-plane",
        title: "HSV Plane",
        description: "Photoshop-style HSV plane with linked H, S, and V channel sliders. Raster hue-saturation-value field synced through ColorCompositionSync. CompositionSize scales the plane dimensions.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorFieldState::hue_saturation_value(\"plane\", hsv, thumb_size)\n    .raster_image();\n\nColorSliderBuilder::hue(\"h\", hsv.h).spawn(cx);",
        section_order: 215,
    },
    ControlDocEntry {
        id: "color-hsv-wheel",
        title: "HSV Wheel",
        description: "Hue ring with a centered saturation/value square. ColorRingBuilder and ColorFieldState compose through ColorCompositionSync. CompositionSize scales ring outer size and inner plane.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorRingBuilder::hue(\"ring\", h, 1.0, 0.5).spawn(cx);\nColorFieldState::saturation_value(\"sv\", hsv, thumb).edge_to_edge();",
        section_order: 216,
    },
    ControlDocEntry {
        id: "color-sv-triangle",
        title: "SV Triangle",
        description: "Hue ring with a centered Photoshop-style saturation/value triangle. TriangleDomain and a custom ColorFieldModel2D map barycentric UV to HSV channels.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorRingBuilder::hue(\"ring\", h, 1.0, 0.5).spawn(cx);\nColorFieldState::new(\"tri\", hsv, TriangleDomain { .. }, model);",
        section_order: 217,
    },
    ControlDocEntry {
        id: "color-split-ring",
        title: "Split Ring",
        description: "Offset saturation and lightness arcs framing an inner hue ring. ColorArcBuilder and ColorRingBuilder delegates stay linked via ColorCompositionSync.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorArcBuilder::saturation_with_renderer(\"sat\", s, h, v, Raster).spawn(cx);\nColorRingBuilder::hue(\"hue\", h, s, l).spawn(cx);",
        section_order: 218,
    },
    ControlDocEntry {
        id: "color-multi-mixer",
        title: "Multi Mixer",
        description: "Per-color-space channel mixers — HueAlpha, RGBA, HSLA, HSVA, Lab variants, and OKLCH. Each card binds ColorSliderBuilder channel delegates to a ColorSpecification.",
        category: ControlCategory::Color,
        snippet: "ColorSpaceMixerState::new(\"hsva\", None, true, look, Hsv::from_hsla(color), cx);",
        section_order: 219,
    },
    ControlDocEntry {
        id: "color-harmonies",
        title: "Color Harmonies",
        description: "Hue wheel and lightness ring with harmony selector and palette readout. Monochromatic through hexadic palettes derived from the base color.",
        category: ControlCategory::ColorCompositions,
        snippet: "ColorFieldState::new(\"wheel\", hsv, CircleDomain, HslWheelModel);\nColorRingBuilder::lightness(\"l\", l, h, s).spawn(cx);",
        section_order: 220,
    },
    ControlDocEntry {
        id: "color-slider-revealed",
        title: "Color Slider Revealed",
        description: "Styling survey for the color-slider primitive — scale, corner radius, thumb shapes, edge-to-edge layout, interpolation modes, and delegate families.",
        category: ControlCategory::Color,
        snippet: "ColorSliderBuilder::hue(\"hue\", 180.0)\n    .size(ControlSize::Lg)\n    .thumb_square()\n    .edge_to_edge()\n    .spawn(cx);",
        section_order: 221,
    },
    ControlDocEntry {
        id: "modal-overlay",
        title: "Modal Overlay",
        description: "Modal overlay windows block the workspace for confirm/cancel or short-form tasks. They trap focus, dim the background, and restore focus to the opener on dismiss. Compose header, body, and footer in the content closure; register persistent child controls with theme_children so token changes refresh without app-level notify plumbing.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"confirm-archive\")\n    .mode(OverlayWindowMode::Modal)\n    .content(|_, _, _| { /* body */ })\n    .theme_children([cancel.clone(), confirm.clone()])\n    .spawn(cx);",
        section_order: 300,
    },
    ControlDocEntry {
        id: "modeless-overlay",
        title: "Modeless Overlay",
        description: "Non-blocking overlay surfaces leave the workspace interactive. Use modeless overlays for notes, palettes, and inspectors that should stay open while the user keeps working in the background.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"notes\")\n    .mode(OverlayWindowMode::Modeless)\n    .position(OverlayWindowPosition::TopRight)\n    .content(|_, _, _| { /* body */ })\n    .spawn(cx);",
        section_order: 301,
    },
    ControlDocEntry {
        id: "overlay-positioning",
        title: "Overlay Positioning",
        description: "Reposition the same modeless overlay at center, corner, or absolute coordinates. Call set_position before open to anchor the surface where the workflow needs it.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "overlay.set_position(OverlayWindowPosition::TopRight, cx);\noverlay.open_from(Some(opener_focus), cx);",
        section_order: 302,
    },
    ControlDocEntry {
        id: "draggable-overlay",
        title: "Draggable Overlay",
        description: "Modeless overlays with header drag repositioning. Enable draggable on the builder or entity, then drag the top strip to move tool palettes and floating panels.",
        category: ControlCategory::OverlaysDialogs,
        snippet: "look\n    .overlay_window(\"palette\")\n    .mode(OverlayWindowMode::Modeless)\n    .absolute_position(240.0, 180.0)\n    .draggable(true)\n    .spawn(cx);",
        section_order: 303,
    },
    ControlDocEntry {
        id: "dock-panel",
        title: "Dock Panel",
        description: "Ordered docking layout macro — left, top, right, bottom bands plus a fill child. Pair dock boundaries with DockSplitter entities for draggable resize handles.",
        category: ControlCategory::NavigationPanels,
        snippet: "dock_panel! {\n    left: sidebar,\n    left: left_splitter,\n    fill: editor_content,\n}",
        section_order: 310,
    },
    ControlDocEntry {
        id: "resizable-panels",
        title: "Resizable Panels",
        description: "Weighted panel splits with draggable handles, min/max constraints, hover visibility, and collapse gestures. Horizontal or vertical orientation via builder or resizable_panels! macro.",
        category: ControlCategory::NavigationPanels,
        snippet: "look.resizable_panels(\"workspace\")\n    .orientation(Horizontal)\n    .panels([sidebar_spec, content_spec])\n    .spawn(cx);",
        section_order: 311,
    },
    ControlDocEntry {
        id: "sidebar",
        title: "Sidebar",
        description: "Hierarchical sidebar composition via SidebarControl — header, grouped menus, footer, and optional icon rail. Emits SidebarEvent for selection, collapse, and hover.",
        category: ControlCategory::NavigationPanels,
        snippet: r#"fn spawn_sidebar_control(
    look: &Arc<ShadcnLook>,
    cx: &mut Context<SidebarControlExposition>,
) -> Entity<SidebarControl> {
    let mut pinned_menu = shadcn::Sidebar::menu("pinned_menu");

    for leaf in PINNED_PROPERTIES {
        pinned_menu = pinned_menu.item(property_leaf_menu_item(look, leaf));
    }

    let mut properties_menu = shadcn::Sidebar::menu("properties_menu");

    for group in PROPERTY_GROUPS {
        let mut sub = shadcn::Sidebar::menu_sub();

        for leaf in group.leaves {
            sub = sub.item(property_leaf_menu_item(look, leaf));
        }

        properties_menu = properties_menu.item(
            shadcn::Sidebar::menu_item(group.id, group.label)
                .icon(group.icon)
                .expanded(group.expanded)
                .sub(sub),
        );
    }

    shadcn::Sidebar::new("controls-doc-sidebar-control").look(look.as_ref())
        .default_open(true)
        .collapsible(SidebarCollapsible::Icon)
        .auto_hide_scrollbar(true)
        .auto_hide_scrollbar_activate(ScrollbarAutoHideActivate::Move)
        .sidebar(
            shadcn::Sidebar::panel("workbench_sidebar")
                .header(
                    shadcn::Sidebar::header()
                        .title("Properties")
                        .subtitle("Rectangle / Prominent card"),
                )
                .content(
                    shadcn::Sidebar::content()
                        .group(shadcn::Sidebar::group().label("Pinned").menu(pinned_menu))
                        .group(
                            shadcn::Sidebar::group()
                                .label("Properties")
                                .menu(properties_menu),
                        ),
                )
                .rail(shadcn::Sidebar::rail()),
        )
        .overlay_scrollbar(true)
        .spawn(cx)
}"#,
        section_order: 312,
    },
    ControlDocEntry {
        id: "tabs-navigation",
        title: "Tabs Navigation",
        description: "Horizontal tab strip with intrinsic or uniform width modes, keyboard roving focus, and template overrides. Emits Activate and Change for tab selection binding.",
        category: ControlCategory::NavigationPanels,
        snippet: "shadcn::Tabs::new(\"project\").look(look.as_ref())\n    .items(tabs)\n    .active(\"overview\")\n    .spawn(cx);",
        section_order: 313,
    },
    ControlDocEntry {
        id: "slide-panel",
        title: "Slide Panel",
        description: "Gallery drawer prototype — deferred window overlay with edge-anchored panels, open/close animation, focus trap and restore, Escape and backdrop dismissal, and draggable resize handles.",
        category: ControlCategory::NavigationPanels,
        snippet: "let mut state = SlidePanelState::new(SlidePanelTopAnchor::BelowTopBar, SlidePanelSizeConfig::new(360.0, 280.0, 720.0));\nstate.open(SlidePanelEdge::Right, opener_focus);\nrender_slide_panel_overlay(slide_panel_background(&look), &slide_panel_panels_look(&look), &state, viewport, content, handlers, resize_handlers);",
        section_order: 314,
    },
];

pub fn catalog_entry(id: &str) -> Option<&'static ControlDocEntry> {
    CONTROL_CATALOG.iter().find(|entry| entry.id == id)
}
