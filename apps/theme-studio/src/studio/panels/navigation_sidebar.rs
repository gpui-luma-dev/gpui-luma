use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::navigation_sidebar::{NavNode, NavigationSidebar, NavigationSidebarEvent};
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";
const SIDEBAR_HEIGHT_PX: f32 = 500.0;
const SIDEBAR_WIDTH_EXPANDED_PX: f32 = 300.0;
const SIDEBAR_WIDTH_COLLAPSED_PX: f32 = 56.0;

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

const APPEARANCE_PROPERTIES: &[PropertyLeaf] = &[
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
    PropertyGroup {
        id: "appearance",
        label: "Appearance",
        icon: LucideIcon::Palette,
        expanded: true,
        leaves: APPEARANCE_PROPERTIES,
    },
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

pub struct NavigationSidebarPanel {
    radix_theme: Arc<RadixTheme>,
    sidebar: Entity<NavigationSidebar>,
    collapsed: Rc<Cell<bool>>,
    _subscriptions: Vec<Subscription>,
}

impl NavigationSidebarPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>) -> Self {
        let sidebar = radix_theme
            .navigation_sidebar("properties-navigation-sidebar")
            .title("Properties")
            .subtitle("Rectangle / Prominent card")
            .collapsible(true)
            .selected_id(INITIAL_PROPERTY_SELECTION_ID)
            .items(property_nodes())
            .footer_nodes(FOOTER_PROPERTIES.iter().map(property_leaf_node))
            .spawn(cx);

        let collapsed = Rc::new(Cell::new(false));
        let collapsed_for_sub = collapsed.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&sidebar, move |_, _, event: &NavigationSidebarEvent, cx| {
            if let NavigationSidebarEvent::CollapsedChanged { collapsed: next_collapsed } = event {
                collapsed_for_sub.set(*next_collapsed);
                cx.notify();
            }
        }));

        Self { radix_theme, sidebar, collapsed, _subscriptions: subscriptions }
    }
}

impl Render for NavigationSidebarPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let sidebar_width = if self.collapsed.get() {
            SIDEBAR_WIDTH_COLLAPSED_PX
        } else {
            SIDEBAR_WIDTH_EXPANDED_PX
        };
        div()
            .flex_none()
            .w(px(sidebar_width))
            .h(px(SIDEBAR_HEIGHT_PX))
            .overflow_hidden()
            .rounded_b(px(8.0))
            .border_1()
            .border_t_0()
            .border_color(chrome.border)
            .child(self.sidebar.clone())
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
