use gpui_luma::controls::navigation_sidebar::NavNode;
use lucide_icons::Icon as LucideIcon;

pub(crate) const INITIAL_PROPERTY_SELECTION_ID: &str = "dimensions";

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

pub(crate) fn property_navigation_footer_nodes() -> Vec<NavNode> {
    FOOTER_PROPERTIES.iter().map(property_leaf_node).collect()
}

pub(crate) fn property_navigation_nodes() -> Vec<NavNode> {
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
