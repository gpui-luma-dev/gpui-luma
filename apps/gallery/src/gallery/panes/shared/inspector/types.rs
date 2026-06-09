use gpui::SharedString;
use gpui_luma::controls::tree_view::TreeNode;
use gpui_luma::theme::ControlSize;

#[derive(Clone)]
pub(in crate::gallery) enum ColorInspectTreeData {
    Branch,
    ColorField(InspectColorFieldData),
    LayoutSize(InspectLayoutSizeData),
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectColorFieldData {
    pub css_key: SharedString,
    pub provenance: Option<SharedString>,
    pub swatch: gpui::Hsla,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectMetricPropertyData {
    pub name: SharedString,
    pub value: SharedString,
    pub source: SharedString,
    pub provenance: Option<SharedString>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectLayoutSizeData {
    pub size: ControlSize,
    pub properties: Vec<InspectMetricPropertyData>,
}

#[derive(Clone)]
pub(in crate::gallery) enum InspectFieldKind {
    Color(InspectColorFieldData),
    Layout(InspectLayoutSizeData),
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectFieldSelection {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: InspectFieldKind,
}

pub(in crate::gallery) fn find_inspect_field_selection(
    items: &[TreeNode<ColorInspectTreeData>],
    id: &SharedString,
) -> Option<InspectFieldSelection> {
    for node in items {
        if node.id == *id {
            return match &node.data {
                ColorInspectTreeData::ColorField(field) => Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Color(field.clone()),
                }),
                ColorInspectTreeData::LayoutSize(field) => Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Layout(field.clone()),
                }),
                ColorInspectTreeData::Branch => None,
            };
        }
        if let Some(selection) = find_inspect_field_selection(&node.children, id) {
            return Some(selection);
        }
    }
    None
}

pub(in crate::gallery) fn first_inspect_field_id(items: &[TreeNode<ColorInspectTreeData>]) -> Option<SharedString> {
    fn walk(nodes: &[TreeNode<ColorInspectTreeData>]) -> Option<SharedString> {
        for node in nodes {
            if matches!(node.data, ColorInspectTreeData::ColorField(_) | ColorInspectTreeData::LayoutSize(_)) {
                return Some(node.id.clone());
            }
            if let Some(id) = walk(&node.children) {
                return Some(id);
            }
        }
        None
    }
    walk(items)
}

pub(in crate::gallery) fn inspect_slug(label: &str) -> String {
    label.to_ascii_lowercase()
}
