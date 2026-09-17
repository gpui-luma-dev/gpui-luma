use std::collections::HashMap;

use gpui::{Context, FocusHandle, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::super::{NavNode, NavNodeKind, NavNodeState, SidebarPanelEngineRenderModel, RenderedNavNode};
use super::nodes::{branch_progress_for, branch_visible_for, collapsed_rail_node_count, collapsed_rail_node_visible};
use super::{NavigationFocusTarget, SidebarPanelEngine};
use crate::key_handling::ControlKeyProfile;
use crate::motion::VisualTransition;

impl SidebarPanelEngine {
    pub(super) fn render_model(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SidebarPanelEngineRenderModel {
        let mut visible_focus_nodes = Vec::new();

        let rail_nodes = if self.model.collapsed {
            render_collapsed_rail_nodes(
                &self.model.nodes,
                self.model.enabled,
                self.model.selected_id.as_ref(),
                self.hovered_node.as_ref(),
                self.pressed_node.as_ref(),
                &mut self.rail_focus_handles,
                &mut visible_focus_nodes,
                window,
                cx,
            )
        } else {
            Vec::new()
        };
        let rail_footer_nodes = if self.model.collapsed {
            render_collapsed_rail_nodes(
                &self.model.footer_nodes,
                self.model.enabled,
                self.model.selected_id.as_ref(),
                self.hovered_node.as_ref(),
                self.pressed_node.as_ref(),
                &mut self.rail_focus_handles,
                &mut visible_focus_nodes,
                window,
                cx,
            )
        } else {
            Vec::new()
        };
        if let Some(current) = self.rendered_rail_submenu() {
            self.rail_submenu_content = Some(current);
        }
        if let Some(content) = &mut self.rail_submenu_content {
            content.presence = self.rail_submenu_lifecycle.presence();
            content.content_size = self.rail_submenu_content_size;
        }
        let rail_submenu = self
            .rail_submenu_lifecycle
            .presence()
            .should_paint()
            .then(|| self.rail_submenu_content.clone())
            .flatten();

        if self.model.collapsed {
            self.visible_focus_nodes = visible_focus_nodes;

            return SidebarPanelEngineRenderModel {
                id: self.model.id.clone(),
                title: self.model.title.clone(),
                subtitle: self.model.subtitle.clone(),
                header_nodes: Vec::new(),
                nodes: Vec::new(),
                footer_nodes: Vec::new(),
                rail_nodes,
                rail_footer_nodes,
                rail_submenu,
                selected_id: self.model.selected_id.clone(),
                collapsed: self.model.collapsed,
                disclosure_icons: self.model.disclosure_icons.clone(),
            };
        }

        let header_nodes = render_nodes(
            &self.model.header_nodes,
            0,
            self.model.enabled,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
            &self.branch_transitions,
            &self.branch_heights_px,
        );
        let nodes = render_nodes(
            &self.model.nodes,
            0,
            self.model.enabled,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
            &self.branch_transitions,
            &self.branch_heights_px,
        );
        let footer_nodes = render_nodes(
            &self.model.footer_nodes,
            0,
            self.model.enabled,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
            &self.branch_transitions,
            &self.branch_heights_px,
        );

        self.visible_focus_nodes = visible_focus_nodes;

        SidebarPanelEngineRenderModel {
            id: self.model.id.clone(),
            title: self.model.title.clone(),
            subtitle: self.model.subtitle.clone(),
            header_nodes,
            nodes,
            footer_nodes,
            rail_nodes,
            rail_footer_nodes,
            rail_submenu,
            selected_id: self.model.selected_id.clone(),
            collapsed: self.model.collapsed,
            disclosure_icons: self.model.disclosure_icons.clone(),
        }
    }
}

impl Render for SidebarPanelEngine {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_rail_submenu_animating = self.rail_submenu_lifecycle.presence().is_animating();
        let is_rail_submenu_animating = self.rail_submenu_lifecycle.sync();
        self.rail_submenu_lifecycle.schedule_frame(window, cx);
        if was_rail_submenu_animating || is_rail_submenu_animating {
            cx.notify();
        }
        if !self.rail_submenu_lifecycle.presence().should_paint() {
            self.rail_submenu_content = None;
        }
        self.sync_branch_transitions(window, cx);
        let model = self.render_model(window, cx);
        self.sync_focus_subscriptions(window, cx);
        let handlers = self.template_handlers(&model, cx);
        self.main_scroll.sync_scrollbar(cx);

        div()
            .size_full()
            .key_context(ControlKeyProfile::Navigation.context())
            .on_action(cx.listener(Self::handle_escape_focus))
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_open_submenu))
            .on_action(cx.listener(Self::handle_close_submenu))
            .on_action(cx.listener(Self::handle_activate_control))
            .child(self.model.template.render(model, &self.main_scroll, handlers, window, cx))
            .into_any_element()
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn render_nodes(
    nodes: &[NavNode],
    depth: usize,
    sidebar_enabled: bool,
    sidebar_collapsed: bool,
    selected_id: Option<&SharedString>,
    hovered_node: Option<&SharedString>,
    pressed_node: Option<&SharedString>,
    row_focus_handles: &mut HashMap<SharedString, FocusHandle>,
    visible_focus_nodes: &mut Vec<(NavigationFocusTarget, FocusHandle)>,
    window: &mut Window,
    cx: &mut Context<SidebarPanelEngine>,
    branch_transitions: &HashMap<SharedString, VisualTransition>,
    branch_heights_px: &HashMap<SharedString, f32>,
) -> Vec<RenderedNavNode> {
    let sibling_count = nodes.iter().filter(|node| node.visible).count();
    let mut visible_index = 0;
    let mut rendered_nodes = Vec::new();

    for node in nodes {
        if !node.visible {
            continue;
        }

        let has_default_interaction =
            sidebar_enabled && node.kind == NavNodeKind::Item && node.enabled && node.presenter.is_none();
        let focus_handle = has_default_interaction.then(|| {
            let focus_handle = row_focus_handles.entry(node.id.clone()).or_insert_with(|| cx.focus_handle()).clone();
            let focus_handle = focus_handle.tab_stop(true);
            row_focus_handles.insert(node.id.clone(), focus_handle.clone());
            visible_focus_nodes.push((node.id.clone(), focus_handle.clone()));
            focus_handle
        });

        let state = NavNodeState {
            id: node.id.clone(),
            depth,
            sidebar_collapsed,
            index: visible_index,
            sibling_count,
            selected: selected_id == Some(&node.id),
            hovered: hovered_node == Some(&node.id),
            pressed: pressed_node == Some(&node.id),
            focused: focus_handle.as_ref().is_some_and(|focus_handle| focus_handle.is_focused(window)),
            expanded: node.expanded,
            enabled: sidebar_enabled && node.enabled,
        };
        visible_index += 1;

        let custom_content = node.presenter.as_ref().map(|presenter| presenter.present(&state, window, cx));
        if sidebar_enabled
            && node.enabled
            && let Some(focus_handle) = custom_content.as_ref().and_then(|content| content.focus_handle.as_ref())
        {
            visible_focus_nodes.push((node.id.clone(), focus_handle.clone()));
        }

        let has_children = !node.children.is_empty();
        let children = if branch_visible_for(branch_transitions, &node.id, node.expanded) {
            render_nodes(
                &node.children,
                depth + 1,
                sidebar_enabled,
                sidebar_collapsed,
                selected_id,
                hovered_node,
                pressed_node,
                row_focus_handles,
                visible_focus_nodes,
                window,
                cx,
                branch_transitions,
                branch_heights_px,
            )
        } else {
            Vec::new()
        };

        rendered_nodes.push(RenderedNavNode {
            id: node.id.clone(),
            kind: node.kind,
            label: node.label.clone(),
            icon: node.icon,
            state,
            custom_element: custom_content.map(|content| content.element),
            focus_handle,
            has_children,
            expansion_progress: branch_progress_for(branch_transitions, &node.id, node.expanded),
            children_height_px: branch_heights_px.get(&node.id).copied().unwrap_or(0.0),
            children,
        });
    }

    rendered_nodes
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_collapsed_rail_nodes(
    nodes: &[NavNode],
    sidebar_enabled: bool,
    selected_id: Option<&SharedString>,
    hovered_node: Option<&SharedString>,
    pressed_node: Option<&SharedString>,
    rail_focus_handles: &mut HashMap<SharedString, FocusHandle>,
    visible_focus_nodes: &mut Vec<(NavigationFocusTarget, FocusHandle)>,
    window: &mut Window,
    cx: &mut Context<SidebarPanelEngine>,
) -> Vec<RenderedNavNode> {
    let sibling_count = collapsed_rail_node_count(nodes);
    let mut rendered_nodes = Vec::new();

    for (visible_index, node) in nodes.iter().filter(|node| collapsed_rail_node_visible(node)).enumerate() {
        let focus_handle = (sidebar_enabled && node.enabled).then(|| {
            let focus_handle = rail_focus_handles.entry(node.id.clone()).or_insert_with(|| cx.focus_handle()).clone();
            let focus_handle = focus_handle.tab_stop(true);
            rail_focus_handles.insert(node.id.clone(), focus_handle.clone());
            visible_focus_nodes.push((node.id.clone(), focus_handle.clone()));
            focus_handle
        });

        let state = NavNodeState {
            id: node.id.clone(),
            depth: 0,
            sidebar_collapsed: true,
            index: visible_index,
            sibling_count,
            selected: selected_id == Some(&node.id),
            hovered: hovered_node == Some(&node.id),
            pressed: pressed_node == Some(&node.id),
            focused: focus_handle.as_ref().is_some_and(|focus_handle| focus_handle.is_focused(window)),
            expanded: node.expanded,
            enabled: sidebar_enabled && node.enabled,
        };

        rendered_nodes.push(RenderedNavNode {
            id: node.id.clone(),
            kind: node.kind,
            label: node.label.clone(),
            icon: node.icon,
            state,
            custom_element: None,
            focus_handle,
            has_children: !node.children.is_empty(),
            expansion_progress: 0.0,
            children_height_px: 0.0,
            children: Vec::new(),
        });
    }

    rendered_nodes
}
