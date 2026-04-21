use gpui::{App, Context, EventEmitter, FocusHandle, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::{
    NavNode, NavNodeState, NavigationSidebarBuilder, NavigationSidebarModel, NavigationSidebarRenderModel,
    RenderedNavNode,
};
use crate::keyhandling::{
    CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum NavigationSidebarEvent {
    BranchExpandedChanged { node_id: SharedString, expanded: bool },
}

pub struct NavigationSidebar {
    model: NavigationSidebarModel,
    visible_focus_handles: Vec<FocusHandle>,
}

impl EventEmitter<NavigationSidebarEvent> for NavigationSidebar {}

impl NavigationSidebar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> NavigationSidebarBuilder {
        NavigationSidebarBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: NavigationSidebarBuilder, _cx: &mut Context<Self>) -> Self {
        Self { model: builder.model, visible_focus_handles: Vec::new() }
    }

    pub fn set_header_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.header_nodes = nodes.into_iter().collect();
        cx.notify();
    }

    pub fn set_items(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.nodes = nodes.into_iter().collect();
        cx.notify();
    }

    pub fn set_footer_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.footer_nodes = nodes.into_iter().collect();
        cx.notify();
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.model.collapsed != collapsed {
            self.model.collapsed = collapsed;
            cx.notify();
        }
    }

    pub fn set_node_expanded(&mut self, node_id: impl Into<SharedString>, expanded: bool, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if set_expanded_in_nodes(&mut self.model.nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.header_nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.footer_nodes, &node_id, expanded)
        {
            cx.emit(NavigationSidebarEvent::BranchExpandedChanged { node_id, expanded });
            cx.notify();
        }
    }

    pub fn toggle_node_expanded(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        let Some(expanded) = toggle_expanded_in_nodes(&mut self.model.nodes, &node_id)
            .or_else(|| toggle_expanded_in_nodes(&mut self.model.header_nodes, &node_id))
            .or_else(|| toggle_expanded_in_nodes(&mut self.model.footer_nodes, &node_id))
        else {
            return;
        };

        cx.emit(NavigationSidebarEvent::BranchExpandedChanged { node_id, expanded });
        cx.notify();
    }

    fn render_model(&mut self, window: &mut Window, cx: &mut Context<Self>) -> NavigationSidebarRenderModel {
        let mut visible_focus_handles = Vec::new();

        let header_nodes =
            render_nodes(&self.model.header_nodes, 0, self.model.collapsed, &mut visible_focus_handles, window, cx);
        let nodes = render_nodes(&self.model.nodes, 0, self.model.collapsed, &mut visible_focus_handles, window, cx);
        let footer_nodes =
            render_nodes(&self.model.footer_nodes, 0, self.model.collapsed, &mut visible_focus_handles, window, cx);

        self.visible_focus_handles = visible_focus_handles;

        NavigationSidebarRenderModel {
            id: self.model.id.clone(),
            header_nodes,
            nodes,
            footer_nodes,
            collapsed: self.model.collapsed,
        }
    }

    fn move_focus(&self, window: &mut Window, cx: &mut Context<Self>, direction: FocusDirection) {
        let Some(next_focus_handle) = next_focus_handle(&self.visible_focus_handles, window, direction) else {
            return;
        };

        next_focus_handle.focus(window, cx);
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::Previous);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::Next);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::First);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(window, cx, FocusDirection::Last);
    }

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, _window: &mut Window, _cx: &mut Context<Self>) {}
}

impl Render for NavigationSidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window, cx);

        div()
            .size_full()
            .key_context(ControlKeyProfile::Navigation.context())
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_open_submenu))
            .on_action(cx.listener(Self::handle_close_submenu))
            .child(self.model.template.render(model, window, cx))
            .into_any_element()
    }
}

fn render_nodes(
    nodes: &[NavNode],
    depth: usize,
    sidebar_collapsed: bool,
    visible_focus_handles: &mut Vec<FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<RenderedNavNode> {
    let sibling_count = nodes.iter().filter(|node| node.visible).count();
    let mut visible_index = 0;
    let mut rendered_nodes = Vec::new();

    for node in nodes {
        if !node.visible {
            continue;
        }

        let state = NavNodeState {
            id: node.id.clone(),
            depth,
            sidebar_collapsed,
            index: visible_index,
            sibling_count,
            expanded: node.expanded,
            enabled: node.enabled,
        };
        visible_index += 1;

        let hosted_content = node.content_presenter.present(&state, window, cx);
        if node.enabled
            && let Some(focus_handle) = hosted_content.focus_handle.as_ref()
        {
            visible_focus_handles.push(focus_handle.clone());
        }

        let children = if node.expanded {
            render_nodes(&node.children, depth + 1, sidebar_collapsed, visible_focus_handles, window, cx)
        } else {
            Vec::new()
        };

        rendered_nodes.push(RenderedNavNode { id: node.id.clone(), state, element: hosted_content.element, children });
    }

    rendered_nodes
}

#[derive(Clone, Copy)]
enum FocusDirection {
    Previous,
    Next,
    First,
    Last,
}

fn next_focus_handle(focus_handles: &[FocusHandle], window: &Window, direction: FocusDirection) -> Option<FocusHandle> {
    if focus_handles.is_empty() {
        return None;
    }

    match direction {
        FocusDirection::First => focus_handles.first().cloned(),
        FocusDirection::Last => focus_handles.last().cloned(),
        FocusDirection::Previous | FocusDirection::Next => {
            let current_index = focus_handles.iter().position(|focus_handle| focus_handle.is_focused(window));
            let next_index = match (current_index, direction) {
                (Some(index), FocusDirection::Previous) => index.checked_sub(1).unwrap_or(focus_handles.len() - 1),
                (Some(index), FocusDirection::Next) => (index + 1) % focus_handles.len(),
                (None, FocusDirection::Previous) => focus_handles.len() - 1,
                (None, FocusDirection::Next) => 0,
                _ => unreachable!(),
            };

            focus_handles.get(next_index).cloned()
        }
    }
}

fn set_expanded_in_nodes(nodes: &mut [NavNode], node_id: &SharedString, expanded: bool) -> bool {
    for node in nodes {
        if node.id == *node_id && !node.children.is_empty() && node.enabled && node.expanded != expanded {
            node.expanded = expanded;
            return true;
        }

        if set_expanded_in_nodes(&mut node.children, node_id, expanded) {
            return true;
        }
    }

    false
}

fn toggle_expanded_in_nodes(nodes: &mut [NavNode], node_id: &SharedString) -> Option<bool> {
    for node in nodes {
        if node.id == *node_id && !node.children.is_empty() && node.enabled {
            node.expanded = !node.expanded;
            return Some(node.expanded);
        }

        if let Some(expanded) = toggle_expanded_in_nodes(&mut node.children, node_id) {
            return Some(expanded);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{set_expanded_in_nodes, toggle_expanded_in_nodes};
    use crate::controls::navigation_sidebar::NavNode;

    #[test]
    fn toggles_enabled_branch() {
        let branch_id: SharedString = "inputs".into();
        let mut nodes = vec![NavNode::new("inputs").child(NavNode::new("button"))];

        assert_eq!(toggle_expanded_in_nodes(&mut nodes, &branch_id), Some(true));
        assert!(nodes[0].is_expanded());
    }

    #[test]
    fn ignores_disabled_branch() {
        let branch_id: SharedString = "inputs".into();
        let mut nodes = vec![NavNode::new("inputs").child(NavNode::new("button")).enabled(false)];

        assert_eq!(toggle_expanded_in_nodes(&mut nodes, &branch_id), None);
        assert!(!set_expanded_in_nodes(&mut nodes, &branch_id, true));
    }
}
