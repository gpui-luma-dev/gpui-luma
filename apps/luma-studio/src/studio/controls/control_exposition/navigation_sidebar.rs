//! Navigation sidebar control exposition — gallery-aligned properties tree.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::navigation_sidebar::{NavNode, NavigationSidebar, NavigationSidebarEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "NavigationSidebarEvent::Activate { node_id, label }",
        trigger: "Pointer or keyboard activate on an enabled row",
        notes: "Primary selection signal for navigation sidebars.",
    },
    EventReferenceSpec {
        event: "NavigationSidebarEvent::BranchExpandedChanged { node_id, expanded }",
        trigger: "Expand/collapse branch rows",
        notes: "Emitted when branch open state changes.",
    },
    EventReferenceSpec {
        event: "NavigationSidebarEvent::CollapsedChanged { collapsed }",
        trigger: "Collapse rail toggle",
        notes: "When collapsible(true), sidebar width switches to icon rail.",
    },
    EventReferenceSpec {
        event: "NavigationSidebarEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the sidebar",
        notes: "Useful for form-level focus coordination.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "NavigationSidebar",
        surface: "Type",
        notes: "Entity<NavigationSidebar> — hierarchical nav with optional collapse rail.",
    },
    PublicInterfaceSpec {
        symbol: "look.navigation_sidebar(id)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory with title, items, footer_nodes.",
    },
    PublicInterfaceSpec {
        symbol: "NavNode::new / section / children",
        surface: "Model",
        notes: "Branch, leaf, and section header node construction.",
    },
];

#[derive(Clone, Copy)]
struct PropertyLeaf {
    id: &'static str,
    label: &'static str,
    icon: Option<LucideIcon>,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct PropertyGroup {
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    expanded: bool,
    leaves: &'static [PropertyLeaf],
}

const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";

const PINNED_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "summary", label: "Summary", icon: Some(LucideIcon::Info), enabled: true },
    PropertyLeaf { id: "tokens", label: "Design Tokens", icon: Some(LucideIcon::Tags), enabled: true },
];

const LAYOUT_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "position", label: "Position", icon: None, enabled: true },
    PropertyLeaf { id: INITIAL_PROPERTY_SELECTION_ID, label: "Dimensions", icon: None, enabled: true },
    PropertyLeaf { id: "constraints", label: "Constraints", icon: None, enabled: true },
    PropertyLeaf { id: "grid", label: "Grid", icon: None, enabled: true },
];

const LOOK_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "fill", label: "Fill", icon: None, enabled: true },
    PropertyLeaf { id: "stroke", label: "Stroke", icon: None, enabled: true },
    PropertyLeaf { id: "typography", label: "Typography", icon: None, enabled: true },
    PropertyLeaf { id: "effects", label: "Effects", icon: None, enabled: true },
];

const BEHAVIOR_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "interactions", label: "Interactions", icon: None, enabled: true },
    PropertyLeaf { id: "conditions", label: "Conditions", icon: None, enabled: true },
    PropertyLeaf { id: "validation", label: "Validation", icon: None, enabled: true },
    PropertyLeaf { id: "data-binding", label: "Data Binding", icon: None, enabled: false },
];

const PROPERTY_GROUPS: &[PropertyGroup] = &[
    PropertyGroup { id: "layout", label: "Layout", icon: LucideIcon::Ruler, expanded: true, leaves: LAYOUT_PROPERTIES },
    PropertyGroup { id: "look", label: "Look", icon: LucideIcon::Palette, expanded: true, leaves: LOOK_PROPERTIES },
    PropertyGroup {
        id: "behavior",
        label: "Behavior",
        icon: LucideIcon::MousePointer2,
        expanded: false,
        leaves: BEHAVIOR_PROPERTIES,
    },
];

const FOOTER_PROPERTIES: &[PropertyLeaf] = &[
    PropertyLeaf { id: "audit-log", label: "Audit Log", icon: Some(LucideIcon::FileText), enabled: true },
    PropertyLeaf { id: "reset-overrides", label: "Reset Overrides", icon: Some(LucideIcon::RotateCcw), enabled: false },
];

pub struct NavigationSidebarControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    sidebar: Entity<NavigationSidebar>,
    collapsed: Rc<Cell<bool>>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl NavigationSidebarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("navigation-sidebar").expect("navigation-sidebar catalog entry");

        let sidebar = look
            .navigation_sidebar("controls-doc-navigation-sidebar")
            .title("Properties")
            .subtitle("Rectangle / Prominent card")
            .collapsible(true)
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .items(property_nodes())
            .footer_nodes(FOOTER_PROPERTIES.iter().map(property_leaf_node))
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-navigation-sidebar-event-log",
                "Expand branches, select rows, and toggle collapse; events appear below.",
            )
        });

        let collapsed = Rc::new(Cell::new(false));
        let collapsed_cell = collapsed.clone();
        let subscriptions = vec![cx.subscribe(&sidebar, {
            let event_stream = event_stream.clone();
            move |_, _, event: &NavigationSidebarEvent, cx| {
                if let NavigationSidebarEvent::CollapsedChanged { collapsed } = event {
                    collapsed_cell.set(*collapsed);
                    cx.notify();
                }
                if let Some(line) = format_navigation_sidebar_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        })];

        Self { look, entry, sidebar, collapsed, event_stream, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.sidebar.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for NavigationSidebarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let width = if self.collapsed.get() { px(56.0) } else { px(300.0) };

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .flex_none()
                        .w(width)
                        .h(px(500.0))
                        .overflow_hidden()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(chrome.border)
                        .child(self.sidebar.clone()),
                )
                .child(self.event_stream.clone());

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn format_navigation_sidebar_event(event: &NavigationSidebarEvent) -> Option<String> {
    match event {
        NavigationSidebarEvent::Activate { node_id, label } => {
            Some(format!("NavigationSidebarEvent::Activate {{ node_id: \"{node_id}\", label: \"{label}\" }}"))
        }
        NavigationSidebarEvent::BranchExpandedChanged { node_id, expanded } => Some(format!(
            "NavigationSidebarEvent::BranchExpandedChanged {{ node_id: \"{node_id}\", expanded: {expanded} }}"
        )),
        NavigationSidebarEvent::CollapsedChanged { collapsed } => {
            Some(format!("NavigationSidebarEvent::CollapsedChanged {{ collapsed: {collapsed} }}"))
        }
        NavigationSidebarEvent::FocusChanged { focused } => {
            Some(format!("NavigationSidebarEvent::FocusChanged {{ focused: {focused} }}"))
        }
        NavigationSidebarEvent::ItemFocused { node_id, label } => {
            Some(format!("NavigationSidebarEvent::ItemFocused {{ node_id: \"{node_id}\", label: \"{label}\" }}"))
        }
        NavigationSidebarEvent::ItemHoverChanged { node_id, hovered } => {
            Some(format!("NavigationSidebarEvent::ItemHoverChanged {{ node_id: \"{node_id}\", hovered: {hovered} }}"))
        }
        NavigationSidebarEvent::RailSubmenuOpenChanged { node_id } => Some(match node_id {
            Some(id) => format!("NavigationSidebarEvent::RailSubmenuOpenChanged {{ node_id: Some(\"{id}\") }}"),
            None => "NavigationSidebarEvent::RailSubmenuOpenChanged { node_id: None }".to_string(),
        }),
        NavigationSidebarEvent::EnabledChanged { enabled } => {
            Some(format!("NavigationSidebarEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}

fn property_nodes() -> Vec<NavNode> {
    let mut nodes = Vec::new();

    nodes.push(NavNode::section("pinned-label", "Pinned"));
    nodes.extend(PINNED_PROPERTIES.iter().map(property_leaf_node));

    nodes.push(NavNode::section("properties-label", "Properties"));
    nodes.extend(PROPERTY_GROUPS.iter().map(|group| {
        NavNode::new(group.id)
            .label(group.label)
            .icon(group.icon)
            .expanded(group.expanded)
            .children(group.leaves.iter().map(property_leaf_node))
    }));

    nodes
}

fn property_leaf_node(leaf: &PropertyLeaf) -> NavNode {
    let mut node = NavNode::new(leaf.id).label(leaf.label).enabled(leaf.enabled);

    if let Some(icon) = leaf.icon {
        node = node.icon(icon);
    }

    node
}
