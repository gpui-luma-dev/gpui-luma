use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, FocusHandle, IntoElement, SharedString, div, prelude::*};

use super::{NavigationSidebar, NavigationSidebarTemplate, default_navigation_sidebar_template};
use crate::controls::content_presenter::{ContentPresenter, HostedContent, IntoContentPresenter};

pub type NavHostedContent = HostedContent;
pub type NavContentPresenter = ContentPresenter<NavNodeState>;

#[derive(Clone, Debug)]
pub struct NavNodeState {
    pub id: SharedString,
    pub depth: usize,
    pub sidebar_collapsed: bool,
    pub index: usize,
    pub sibling_count: usize,
    pub expanded: bool,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct NavNode {
    pub(crate) id: SharedString,
    pub(crate) content_presenter: NavContentPresenter,
    pub(crate) children: Vec<NavNode>,
    pub(crate) expanded: bool,
    pub(crate) enabled: bool,
    pub(crate) visible: bool,
}

impl NavNode {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content_presenter: ContentPresenter::new(|_, _, _| NavHostedContent {
                element: div().into_any_element(),
                focus_handle: None,
            }),
            children: Vec::new(),
            expanded: false,
            enabled: true,
            visible: true,
        }
    }

    pub fn content_presenter(mut self, presenter: impl IntoContentPresenter<NavNodeState>) -> Self {
        self.content_presenter = presenter.into_content_presenter();
        self
    }

    pub fn child(mut self, child: NavNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = NavNode>) -> Self {
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

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
}

#[derive(Clone)]
pub struct NavigationSidebarModel {
    pub(crate) id: SharedString,
    pub(crate) header_nodes: Vec<NavNode>,
    pub(crate) nodes: Vec<NavNode>,
    pub(crate) footer_nodes: Vec<NavNode>,
    pub(crate) collapsed: bool,
    pub(crate) template: Arc<dyn NavigationSidebarTemplate>,
}

pub struct NavigationSidebarRenderModel {
    pub id: SharedString,
    pub header_nodes: Vec<RenderedNavNode>,
    pub nodes: Vec<RenderedNavNode>,
    pub footer_nodes: Vec<RenderedNavNode>,
    pub collapsed: bool,
}

pub struct RenderedNavNode {
    pub id: SharedString,
    pub state: NavNodeState,
    pub element: AnyElement,
    pub children: Vec<RenderedNavNode>,
}

pub struct NavigationSidebarBuilder {
    pub(crate) model: NavigationSidebarModel,
}

impl NavigationSidebarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: NavigationSidebarModel {
                id: id.into(),
                header_nodes: Vec::new(),
                nodes: Vec::new(),
                footer_nodes: Vec::new(),
                collapsed: false,
                template: default_navigation_sidebar_template(),
            },
        }
    }

    pub fn header_node(mut self, node: NavNode) -> Self {
        self.model.header_nodes.push(node);
        self
    }

    pub fn header_nodes(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.header_nodes = nodes.into_iter().collect();
        self
    }

    pub fn item(mut self, node: NavNode) -> Self {
        self.model.nodes.push(node);
        self
    }

    pub fn items(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.nodes = nodes.into_iter().collect();
        self
    }

    pub fn footer_node(mut self, node: NavNode) -> Self {
        self.model.footer_nodes.push(node);
        self
    }

    pub fn footer_nodes(mut self, nodes: impl IntoIterator<Item = NavNode>) -> Self {
        self.model.footer_nodes = nodes.into_iter().collect();
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.model.collapsed = collapsed;
        self
    }

    pub fn template(mut self, template: Arc<dyn NavigationSidebarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<NavigationSidebar> {
        cx.new(|cx| NavigationSidebar::from_builder(self, cx))
    }
}

pub fn hosted_entity_presenter<T>(entity: Entity<T>, focus_handle: FocusHandle) -> NavContentPresenter
where
    Entity<T>: IntoElement + Clone + 'static,
{
    ContentPresenter::new(move |_, _, _| NavHostedContent {
        element: div().w_full().child(entity.clone()).into_any_element(),
        focus_handle: Some(focus_handle.clone()),
    })
}
