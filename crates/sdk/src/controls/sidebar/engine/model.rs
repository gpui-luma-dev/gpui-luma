use std::sync::Arc;

use gpui::{AnyElement, AppContext, Bounds, Entity, FocusHandle, IntoElement, Pixels, SharedString, div, prelude::*};
use lucide_icons::Icon as LucideIcon;

use super::{SidebarPanelEngine, SidebarPanelTemplate, default_sidebar_panel_template};
use super::template::modified_sidebar_panel_template;
use crate::controls::presenter::{Presenter, HostedContent, IntoPresenter};
use crate::controls::menu_item::MenuItem;
use crate::controls::state::MenuPath;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};

pub type NavHostedContent = HostedContent;
pub type NavPresenter = Presenter<NavNodeState>;

#[derive(Clone, Debug)]
pub struct NavNodeState {
    pub id: SharedString,
    pub depth: usize,
    pub sidebar_collapsed: bool,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub expanded: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavNodeKind {
    Item,
    Section,
}

#[derive(Clone)]
pub struct NavNode {
    pub(crate) id: SharedString,
    pub(crate) kind: NavNodeKind,
    pub(crate) label: Option<SharedString>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) presenter: Option<NavPresenter>,
    pub(crate) children: Vec<NavNode>,
    pub(crate) expanded: bool,
    pub(crate) enabled: bool,
    pub(crate) visible: bool,
}

impl NavNode {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind: NavNodeKind::Item,
            label: None,
            icon: None,
            presenter: None,
            children: Vec::new(),
            expanded: false,
            enabled: true,
            visible: true,
        }
    }

    pub fn section(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id).label(label).kind(NavNodeKind::Section).enabled(false)
    }

    pub fn kind(mut self, kind: NavNodeKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn presenter(mut self, presenter: impl IntoPresenter<NavNodeState>) -> Self {
        self.presenter = Some(presenter.into_presenter());
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
pub struct SidebarPanelEngineModel {
    pub(crate) id: SharedString,
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
    pub(crate) header_nodes: Vec<NavNode>,
    pub(crate) nodes: Vec<NavNode>,
    pub(crate) footer_nodes: Vec<NavNode>,
    pub(crate) selected_id: Option<SharedString>,
    pub(crate) enabled: bool,
    pub(crate) collapsible: bool,
    pub(crate) collapsed: bool,
    pub(crate) template: Arc<dyn SidebarPanelTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
}

pub struct RenderedCollapseTrigger {
    pub id: SharedString,
    pub collapsed: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
    pub focus_handle: FocusHandle,
}

pub struct RenderedRailSubmenu {
    pub id: SharedString,
    pub parent_node_id: SharedString,
    pub parent_bounds: Bounds<Pixels>,
    pub items: Vec<MenuItem>,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
}

pub struct SidebarPanelEngineRenderModel {
    pub id: SharedString,
    pub title: Option<SharedString>,
    pub subtitle: Option<SharedString>,
    pub header_nodes: Vec<RenderedNavNode>,
    pub nodes: Vec<RenderedNavNode>,
    pub footer_nodes: Vec<RenderedNavNode>,
    pub rail_nodes: Vec<RenderedNavNode>,
    pub rail_footer_nodes: Vec<RenderedNavNode>,
    pub rail_submenu: Option<RenderedRailSubmenu>,
    pub collapse_trigger: Option<RenderedCollapseTrigger>,
    pub selected_id: Option<SharedString>,
    pub collapsible: bool,
    pub collapsed: bool,
}

pub struct RenderedNavNode {
    pub id: SharedString,
    pub kind: NavNodeKind,
    pub label: Option<SharedString>,
    pub icon: Option<LucideIcon>,
    pub state: NavNodeState,
    pub custom_element: Option<AnyElement>,
    pub focus_handle: Option<FocusHandle>,
    pub has_children: bool,
    pub children: Vec<RenderedNavNode>,
}

pub struct SidebarPanelEngineBuilder {
    pub(crate) model: SidebarPanelEngineModel,
}

impl SidebarPanelEngineBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SidebarPanelEngineModel {
                id: id.into(),
                title: None,
                subtitle: None,
                header_nodes: Vec::new(),
                nodes: Vec::new(),
                footer_nodes: Vec::new(),
                selected_id: None,
                enabled: true,
                collapsible: false,
                collapsed: false,
                template: default_sidebar_panel_template(),
                scrollbar_template: default_scrollbar_template(),
            },
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = Some(title.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.model.subtitle = Some(subtitle.into());
        self
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

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.model.collapsed = collapsed;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.model.collapsible = collapsible;
        self
    }

    pub fn template(mut self, template: Arc<dyn SidebarPanelTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_sidebar_panel_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SidebarPanelEngine> {
        cx.new(|cx| SidebarPanelEngine::from_builder(self, cx))
    }
}

pub fn entity_presenter<T>(entity: Entity<T>, focus_handle: FocusHandle) -> NavPresenter
where
    Entity<T>: IntoElement + Clone + 'static,
{
    Presenter::new(move |_, _, _| NavHostedContent {
        element: div().w_full().child(entity.clone()).into_any_element(),
        focus_handle: Some(focus_handle.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_sidebar_panel_template();
        let builder = SidebarPanelEngineBuilder::new("nav-test")
            .template(template.clone())
            .with_template_modifier(|element| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
