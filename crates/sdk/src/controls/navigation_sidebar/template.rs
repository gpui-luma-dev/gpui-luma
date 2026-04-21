use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, Stateful, Window, div, prelude::*, px};

use super::{NavigationSidebarRenderModel, RenderedNavNode};

pub trait NavigationSidebarTemplate: Send + Sync {
    fn render(&self, model: NavigationSidebarRenderModel, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedNavigationSidebarTemplate;

impl ThemedNavigationSidebarTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemedNavigationSidebarTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_navigation_sidebar_template() -> Arc<dyn NavigationSidebarTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn NavigationSidebarTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedNavigationSidebarTemplate)).clone()
}

impl NavigationSidebarTemplate for ThemedNavigationSidebarTemplate {
    fn render(&self, model: NavigationSidebarRenderModel, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let mut root = div().id(model.id).size_full().flex().flex_col().gap(px(8.0)).p(px(8.0));

        if !model.header_nodes.is_empty() {
            root = root.child(render_region(model.header_nodes).pb(px(8.0)));
        }

        root = root.child(render_region(model.nodes).flex_1().min_h(px(0.0)));

        if !model.footer_nodes.is_empty() {
            root = root.child(render_region(model.footer_nodes).pt(px(8.0)));
        }

        root
    }
}

fn render_region(nodes: Vec<RenderedNavNode>) -> Div {
    let mut region = div().flex().flex_col().gap(px(4.0));

    for node in nodes {
        region = region.child(render_node(node));
    }

    region
}

fn render_node(node: RenderedNavNode) -> AnyElement {
    let mut root = div().id(node.id).flex().flex_col().gap(px(4.0)).child(render_row(node.state.depth, node.element));

    if node.state.expanded {
        for child in node.children {
            root = root.child(render_node(child));
        }
    }

    root.into_any_element()
}

fn render_row(_depth: usize, element: gpui::AnyElement) -> Div {
    div().w_full().child(element)
}
