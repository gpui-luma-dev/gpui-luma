use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{ControlFocusState, NavItemState, NavItemTemplate, NavView, NavViewTemplate};
use super::{default_nav_item_template, default_nav_view_template};

#[derive(Clone, Debug)]
pub enum NavItem {
    Button(NavButton),
    Label(NavLabel),
    Node(NavNode),
}

impl NavItem {
    pub fn button(id: impl Into<SharedString>) -> NavButton {
        NavButton::new(id)
    }

    pub fn label(label: impl Into<SharedString>) -> Self {
        Self::Label(NavLabel::new(label))
    }

    pub fn node(id: impl Into<SharedString>) -> NavNode {
        NavNode::new(id)
    }
}

impl From<NavButton> for NavItem {
    fn from(button: NavButton) -> Self {
        Self::Button(button)
    }
}

impl From<NavLabel> for NavItem {
    fn from(label: NavLabel) -> Self {
        Self::Label(label)
    }
}

impl From<NavNode> for NavItem {
    fn from(node: NavNode) -> Self {
        Self::Node(node)
    }
}

#[derive(Clone, Debug)]
pub struct NavButton {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl NavButton {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Debug)]
pub struct NavLabel {
    pub(crate) label: SharedString,
}

impl NavLabel {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into() }
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }
}

#[derive(Clone, Debug)]
pub struct NavNode {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) children: Vec<NavNodeItem>,
    pub(crate) expanded: bool,
    pub(crate) enabled: bool,
}

impl NavNode {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, children: Vec::new(), expanded: false, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn child(mut self, child: NavNodeItem) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = NavNodeItem>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn child_items(&self) -> &[NavNodeItem] {
        &self.children
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Debug)]
pub struct NavNodeItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl NavNodeItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct NavViewModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<NavItem>,
    pub(crate) bottom_items: Vec<NavButton>,
    pub(crate) selected_item_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn NavViewTemplate>,
    pub(crate) item_template: Arc<dyn NavItemTemplate>,
}

pub struct NavViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<NavRenderItem<'a>>,
    pub bottom_items: Vec<NavButtonRenderModel<'a>>,
    pub selected_item_id: Option<&'a SharedString>,
    pub active_item_id: Option<&'a SharedString>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: &'a dyn NavItemTemplate,
}

pub enum NavRenderItem<'a> {
    Button(NavButtonRenderModel<'a>),
    Label(NavLabelRenderModel<'a>),
    Node(NavNodeRenderModel<'a>),
}

pub struct NavButtonRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected: bool,
    pub active: bool,
    pub enabled: bool,
    pub depth: usize,
    pub state: NavItemState,
}

pub struct NavLabelRenderModel<'a> {
    pub label: &'a SharedString,
    pub depth: usize,
}

pub struct NavNodeRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub expanded: bool,
    pub active: bool,
    pub enabled: bool,
    pub depth: usize,
    pub state: NavItemState,
    pub children: Vec<NavNodeItemRenderModel<'a>>,
}

pub struct NavNodeItemRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected: bool,
    pub active: bool,
    pub enabled: bool,
    pub depth: usize,
    pub state: NavItemState,
}

pub struct NavViewBuilder {
    pub(crate) model: NavViewModel,
}

impl NavViewBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: NavViewModel {
                id: id.into(),
                items: Vec::new(),
                bottom_items: Vec::new(),
                selected_item_id: None,
                enabled: true,
                template: default_nav_view_template(),
                item_template: default_nav_item_template(),
            },
        }
    }

    pub fn item(mut self, item: impl Into<NavItem>) -> Self {
        self.model.items.push(item.into());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = impl Into<NavItem>>) -> Self {
        self.model.items = items.into_iter().map(Into::into).collect();
        self
    }

    pub fn bottom_item(mut self, item: NavButton) -> Self {
        self.model.bottom_items.push(item);
        self
    }

    pub fn bottom_items(mut self, items: impl IntoIterator<Item = NavButton>) -> Self {
        self.model.bottom_items = items.into_iter().collect();
        self
    }

    pub fn selected(mut self, selected_item_id: impl Into<SharedString>) -> Self {
        self.model.selected_item_id = Some(selected_item_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn NavViewTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn item_template(mut self, item_template: Arc<dyn NavItemTemplate>) -> Self {
        self.model.item_template = item_template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<NavView> {
        cx.new(|cx| NavView::from_builder(self, cx))
    }
}
