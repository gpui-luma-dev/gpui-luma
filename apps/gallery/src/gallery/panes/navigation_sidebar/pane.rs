use std::sync::Arc;
use std::{cell::Cell, rc::Rc};

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px};
use gpui_luma::controls::navigation_sidebar::{NavNode, NavigationSidebar, NavigationSidebarEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_navigation_sidebar_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct NavigationSidebarPane {
    sidebar: Entity<NavigationSidebar>,
    collapsed: Rc<Cell<bool>>,
    inspector: Entity<ColorInspectorShell>,
}

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

impl NavigationSidebarPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "navigation-sidebar-inspector-tree",
            look.clone(),
            build_navigation_sidebar_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "navigation-sidebar-inspector",
                "navigation-sidebar-inspector-split",
                "navigation-sidebar-inspector-detail",
                build_navigation_sidebar_inspect_tree,
                cx,
            )
        });

        let sidebar = look
            .navigation_sidebar("properties-navigation-sidebar")
            .title("Properties")
            .subtitle("Rectangle / Prominent card")
            .collapsible(true)
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .items(property_nodes())
            .footer_nodes(FOOTER_PROPERTIES.iter().map(property_leaf_node))
            .spawn(cx);

        Self { sidebar, collapsed: Rc::new(Cell::new(false)), inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        let collapsed = self.collapsed.clone();
        subscriptions.push(cx.subscribe(&self.sidebar, move |_, _, event: &NavigationSidebarEvent, cx| {
            if let NavigationSidebarEvent::CollapsedChanged { collapsed: next_collapsed } = event {
                collapsed.set(*next_collapsed);
                cx.notify();
            }
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let width = if self.collapsed.get() { px(56.0) } else { px(300.0) };

        gallery_pane_with_inspector(
            "Navigation Sidebar",
            div()
                .flex_none()
                .w(width)
                .h(px(500.0))
                .overflow_hidden()
                .rounded(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .child(self.sidebar.clone())
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.sidebar, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
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
