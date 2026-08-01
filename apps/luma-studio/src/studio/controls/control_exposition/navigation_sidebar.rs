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
use super::collection_theme_inspectors::NavigationSidebarThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::navigation_sidebar_inspector_adapter::{NavigationSidebarInspectorAdapter, NAVIGATION_SIDEBAR_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;

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
    left_pane: Entity<NavigationSidebarExpositionLeftPane>,
    theme_inspector: Entity<NavigationSidebarThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct NavigationSidebarExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    sidebar: Entity<NavigationSidebar>,
    collapsed: Rc<Cell<bool>>,
    event_stream: Entity<ControlEventStream>,
}

impl NavigationSidebarExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.sidebar.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for NavigationSidebarExpositionLeftPane {
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

            div()
                .id("controls-doc-navigation-sidebar-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
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
        let left_pane = cx.new(|_| NavigationSidebarExpositionLeftPane {
            look: look.clone(),
            entry,
            sidebar: sidebar.clone(),
            collapsed: collapsed.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-navigation-sidebar-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &NAVIGATION_SIDEBAR_INSPECTOR_SPEC,
            NavigationSidebarInspectorAdapter::shared(),
        );

        let collapsed_cell = collapsed.clone();
        let subscription = cx.subscribe(&sidebar, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.clone();
            move |_, _, event: &NavigationSidebarEvent, cx| {
                if let NavigationSidebarEvent::CollapsedChanged { collapsed } = event {
                    collapsed_cell.set(*collapsed);
                    left_pane.update(cx, |_, cx| cx.notify());
                }
                if let Some(line) = format_navigation_sidebar_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: vec![subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for NavigationSidebarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-navigation-sidebar-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
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
