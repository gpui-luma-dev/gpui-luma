use std::collections::HashMap;

use gpui::{
    Bounds, ClickEvent, Context, EventEmitter, FocusHandle, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    SharedString, Subscription, Window, div, prelude::*,
};

use super::{
    NavNode, NavNodeKind, NavNodeState, NavigationSidebarBuilder, NavigationSidebarModel, NavigationSidebarRenderModel,
    NavigationSidebarTemplateHandlers, RenderedCollapseTrigger, RenderedNavNode, RenderedRailSubmenu,
};
use crate::controls::menu_item::MenuItem;
use crate::controls::scroll_container::ScrollContainer;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::state::MenuPath;
use crate::keyhandling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum NavigationSidebarEvent {
    Activate { node_id: SharedString, label: SharedString },
    BranchExpandedChanged { node_id: SharedString, expanded: bool },
    CollapsedChanged { collapsed: bool },
}

pub struct NavigationSidebar {
    model: NavigationSidebarModel,
    main_scroll: ScrollContainer,
    hovered_node: Option<SharedString>,
    pressed_node: Option<SharedString>,
    collapse_trigger_hovered: bool,
    collapse_trigger_pressed: bool,
    collapse_trigger_focus_handle: FocusHandle,
    row_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_node_bounds: HashMap<SharedString, Bounds<Pixels>>,
    open_rail_submenu: Option<SharedString>,
    rail_submenu_open_submenu: Option<usize>,
    rail_submenu_active_path: Option<MenuPath>,
    visible_focus_nodes: Vec<(NavigationFocusTarget, FocusHandle)>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<NavigationSidebarEvent> for NavigationSidebar {}

impl NavigationSidebar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> NavigationSidebarBuilder {
        NavigationSidebarBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: NavigationSidebarBuilder, cx: &mut Context<Self>) -> Self {
        let main_scroll = ScrollContainer::new(
            format!("{}-main-scroll", builder.model.id),
            builder.model.scrollbar_template.clone(),
            cx,
        );
        let scrollbar = main_scroll.scrollbar();
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| match event {
            ScrollbarEvent::Change { value } => {
                this.main_scroll.set_vertical_offset(*value, cx);
            }
        })];

        Self {
            model: builder.model,
            main_scroll,
            hovered_node: None,
            pressed_node: None,
            collapse_trigger_hovered: false,
            collapse_trigger_pressed: false,
            collapse_trigger_focus_handle: cx.focus_handle().tab_stop(true),
            row_focus_handles: HashMap::new(),
            rail_focus_handles: HashMap::new(),
            rail_node_bounds: HashMap::new(),
            open_rail_submenu: None,
            rail_submenu_open_submenu: None,
            rail_submenu_active_path: None,
            visible_focus_nodes: Vec::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn set_header_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.header_nodes = nodes.into_iter().collect();
        self.close_rail_submenu();
        cx.notify();
    }

    pub fn set_items(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.nodes = nodes.into_iter().collect();
        self.close_rail_submenu();
        cx.notify();
    }

    pub fn set_footer_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.footer_nodes = nodes.into_iter().collect();
        self.close_rail_submenu();
        cx.notify();
    }

    pub fn set_selected_id(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_id = selected_id.into();
        if self.model.selected_id.as_ref() != Some(&selected_id) {
            self.model.selected_id = Some(selected_id);
            cx.notify();
        }
    }

    pub fn clear_selected_id(&mut self, cx: &mut Context<Self>) {
        if self.model.selected_id.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.title = Some(title.into());
        cx.notify();
    }

    pub fn clear_title(&mut self, cx: &mut Context<Self>) {
        self.model.title = None;
        cx.notify();
    }

    pub fn set_subtitle(&mut self, subtitle: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.subtitle = Some(subtitle.into());
        cx.notify();
    }

    pub fn clear_subtitle(&mut self, cx: &mut Context<Self>) {
        self.model.subtitle = None;
        cx.notify();
    }

    pub fn collapsed(&self) -> bool {
        self.model.collapsed
    }

    pub fn set_collapsible(&mut self, collapsible: bool, cx: &mut Context<Self>) {
        if self.model.collapsible == collapsible {
            return;
        }

        self.model.collapsible = collapsible;
        if !collapsible && self.model.collapsed {
            self.model.collapsed = false;
            cx.emit(NavigationSidebarEvent::CollapsedChanged { collapsed: false });
        }
        cx.notify();
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.model.collapsed != collapsed {
            self.model.collapsed = collapsed;
            self.hovered_node = None;
            self.pressed_node = None;
            self.collapse_trigger_hovered = false;
            self.collapse_trigger_pressed = false;
            self.close_rail_submenu();
            cx.emit(NavigationSidebarEvent::CollapsedChanged { collapsed });
            cx.notify();
        }
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.set_collapsed(!self.model.collapsed, cx);
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
        let mut visible_focus_nodes = Vec::new();
        let collapse_trigger = self.model.collapsible.then(|| {
            let focus_handle = self.collapse_trigger_focus_handle.clone();
            visible_focus_nodes.push((NavigationFocusTarget::CollapseTrigger, focus_handle.clone()));

            RenderedCollapseTrigger {
                id: format!("{}-collapse-trigger", self.model.id).into(),
                collapsed: self.model.collapsed,
                hovered: self.collapse_trigger_hovered,
                pressed: self.collapse_trigger_pressed,
                focused: focus_handle.is_focused(window),
                focus_handle,
            }
        });

        let rail_nodes = if self.model.collapsed {
            render_collapsed_rail_nodes(
                &self.model.nodes,
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
        let rail_submenu = self.rendered_rail_submenu();

        if self.model.collapsed {
            self.visible_focus_nodes = visible_focus_nodes;

            return NavigationSidebarRenderModel {
                id: self.model.id.clone(),
                title: self.model.title.clone(),
                subtitle: self.model.subtitle.clone(),
                header_nodes: Vec::new(),
                nodes: Vec::new(),
                footer_nodes: Vec::new(),
                rail_nodes,
                rail_footer_nodes,
                rail_submenu,
                collapse_trigger,
                selected_id: self.model.selected_id.clone(),
                collapsible: self.model.collapsible,
                collapsed: self.model.collapsed,
            };
        }

        let header_nodes = render_nodes(
            &self.model.header_nodes,
            0,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
        );
        let nodes = render_nodes(
            &self.model.nodes,
            0,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
        );
        let footer_nodes = render_nodes(
            &self.model.footer_nodes,
            0,
            self.model.collapsed,
            self.model.selected_id.as_ref(),
            self.hovered_node.as_ref(),
            self.pressed_node.as_ref(),
            &mut self.row_focus_handles,
            &mut visible_focus_nodes,
            window,
            cx,
        );

        self.visible_focus_nodes = visible_focus_nodes;

        NavigationSidebarRenderModel {
            id: self.model.id.clone(),
            title: self.model.title.clone(),
            subtitle: self.model.subtitle.clone(),
            header_nodes,
            nodes,
            footer_nodes,
            rail_nodes,
            rail_footer_nodes,
            rail_submenu,
            collapse_trigger,
            selected_id: self.model.selected_id.clone(),
            collapsible: self.model.collapsible,
            collapsed: self.model.collapsed,
        }
    }

    fn template_handlers(
        &self,
        model: &NavigationSidebarRenderModel,
        cx: &mut Context<Self>,
    ) -> NavigationSidebarTemplateHandlers {
        let mut handlers = NavigationSidebarTemplateHandlers::default();

        if model.collapsible {
            handlers.collapse_hover = Some(Box::new(cx.listener(Self::handle_collapse_trigger_hover)));
            handlers.collapse_mouse_down = Some(Box::new(cx.listener(Self::handle_collapse_trigger_mouse_down)));
            handlers.collapse_mouse_up = Some(Box::new(cx.listener(Self::handle_collapse_trigger_mouse_up)));
            handlers.collapse_mouse_up_out = Some(Box::new(cx.listener(Self::handle_collapse_trigger_mouse_up)));
            handlers.collapse_click = Some(Box::new(cx.listener(Self::handle_collapse_trigger_click)));
        }

        if model.collapsed {
            push_template_handlers(&model.rail_nodes, &mut handlers, cx, SidebarNodeInteraction::Rail);
            push_template_handlers(&model.rail_footer_nodes, &mut handlers, cx, SidebarNodeInteraction::Rail);

            if let Some(rail_submenu) = &model.rail_submenu {
                let parent_node_id = rail_submenu.parent_node_id.clone();
                handlers.rail_submenu_mouse_down_out =
                    Some(Box::new(cx.listener(Self::handle_rail_submenu_mouse_down_out)));
                handlers.rail_submenu_item_hovers = (0..rail_submenu.items.len())
                    .map(|index| {
                        let parent_node_id = parent_node_id.clone();
                        Box::new(cx.listener(move |this, hovered, _window, cx| {
                            this.handle_rail_submenu_item_hover(parent_node_id.clone(), index, *hovered, cx);
                        })) as _
                    })
                    .collect();
                handlers.rail_submenu_item_clicks = self
                    .rail_submenu_click_node_ids(&parent_node_id)
                    .into_iter()
                    .map(|node_id| {
                        Box::new(cx.listener(move |this, event, _window, cx| {
                            this.handle_rail_submenu_item_click(node_id.clone(), event, cx);
                        })) as _
                    })
                    .collect();
            }
        } else {
            push_template_handlers(&model.header_nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
            push_template_handlers(&model.nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
            push_template_handlers(&model.footer_nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
        }

        handlers
    }

    fn can_interact_with_default_node(&self, node_id: &SharedString) -> bool {
        self.find_node(node_id)
            .is_some_and(|node| node.enabled && node.kind == NavNodeKind::Item && node.presenter.is_none())
    }

    fn find_node(&self, node_id: &SharedString) -> Option<&NavNode> {
        find_node_in_nodes(&self.model.nodes, node_id)
            .or_else(|| find_node_in_nodes(&self.model.header_nodes, node_id))
            .or_else(|| find_node_in_nodes(&self.model.footer_nodes, node_id))
    }

    fn find_top_level_main_node(&self, node_id: &SharedString) -> Option<&NavNode> {
        self.model.nodes.iter().find(|node| node.id == *node_id)
    }

    fn can_interact_with_rail_node(&self, node_id: &SharedString) -> bool {
        self.find_top_level_main_node(node_id)
            .is_some_and(|node| collapsed_rail_node_visible(node) && node.enabled)
    }

    fn close_rail_submenu(&mut self) {
        self.open_rail_submenu = None;
        self.rail_submenu_open_submenu = None;
        self.rail_submenu_active_path = None;
    }

    fn set_open_rail_submenu(&mut self, node_id: SharedString) {
        if self.open_rail_submenu.as_ref() == Some(&node_id) {
            self.close_rail_submenu();
        } else {
            self.open_rail_submenu = Some(node_id);
            self.rail_submenu_open_submenu = None;
            self.rail_submenu_active_path = None;
        }
    }

    fn rendered_rail_submenu(&self) -> Option<RenderedRailSubmenu> {
        let parent_node_id = self.open_rail_submenu.as_ref()?;
        let parent = self.find_top_level_main_node(parent_node_id)?;
        if parent.children.is_empty() || !collapsed_rail_node_visible(parent) {
            return None;
        }

        let parent_bounds = self.rail_node_bounds.get(parent_node_id)?.clone();
        let items = parent.children.iter().filter(|node| node.visible).map(nav_node_to_menu_item).collect();

        Some(RenderedRailSubmenu {
            id: format!("{}-rail-submenu-{}", self.model.id, parent_node_id).into(),
            parent_node_id: parent_node_id.clone(),
            parent_bounds,
            items,
            open_submenu: self.rail_submenu_open_submenu,
            active_path: self.rail_submenu_active_path,
        })
    }

    fn rail_submenu_click_node_ids(&self, parent_node_id: &SharedString) -> Vec<SharedString> {
        let Some(parent) = self.find_top_level_main_node(parent_node_id) else {
            return Vec::new();
        };

        nav_menu_click_node_ids(&parent.children, self.rail_submenu_open_submenu)
    }

    fn handle_collapse_trigger_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.collapse_trigger_hovered != *hovered {
            self.collapse_trigger_hovered = *hovered;
            if !hovered {
                self.collapse_trigger_pressed = false;
            }
            cx.notify();
        }
    }

    fn handle_collapse_trigger_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.collapse_trigger_focus_handle.focus(window, cx);
        self.collapse_trigger_pressed = true;
        cx.notify();
    }

    fn handle_collapse_trigger_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.collapse_trigger_pressed {
            self.collapse_trigger_pressed = false;
            cx.notify();
        }
    }

    fn handle_collapse_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.toggle_collapsed(cx);
    }

    fn handle_node_hover(&mut self, node_id: SharedString, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_interact_with_default_node(&node_id) {
            return;
        }

        if hovered {
            if self.hovered_node.as_ref() != Some(&node_id) {
                self.hovered_node = Some(node_id);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.hovered_node = None;
            if self.pressed_node.as_ref() == Some(&node_id) {
                self.pressed_node = None;
            }
            cx.notify();
        }
    }

    fn handle_node_mouse_down(
        &mut self,
        node_id: SharedString,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_interact_with_default_node(&node_id) {
            if let Some(focus_handle) = self.row_focus_handles.get(&node_id) {
                focus_handle.focus(window, cx);
            }
            self.pressed_node = Some(node_id);
            cx.notify();
        }
    }

    fn handle_node_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_node.is_some() {
            self.pressed_node = None;
            cx.notify();
        }
    }

    fn handle_node_click(
        &mut self,
        node_id: SharedString,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() {
            return;
        }

        if !self.can_interact_with_default_node(&node_id) {
            return;
        }

        if self.find_node(&node_id).is_some_and(|node| !node.children.is_empty()) {
            self.toggle_node_expanded(node_id, cx);
        } else {
            self.activate_node(node_id, cx);
        }
    }

    fn handle_rail_node_hover(&mut self, node_id: SharedString, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_interact_with_rail_node(&node_id) {
            return;
        }

        if hovered {
            if self.hovered_node.as_ref() != Some(&node_id) {
                self.hovered_node = Some(node_id);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.hovered_node = None;
            if self.pressed_node.as_ref() == Some(&node_id) {
                self.pressed_node = None;
            }
            cx.notify();
        }
    }

    fn handle_rail_node_mouse_down(
        &mut self,
        node_id: SharedString,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_interact_with_rail_node(&node_id) {
            if let Some(focus_handle) = self.rail_focus_handles.get(&node_id) {
                focus_handle.focus(window, cx);
            }
            self.pressed_node = Some(node_id);
            cx.notify();
        }
    }

    fn handle_rail_node_click(
        &mut self,
        node_id: SharedString,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.is_keyboard() || !self.can_interact_with_rail_node(&node_id) {
            return;
        }

        if self.find_top_level_main_node(&node_id).is_some_and(|node| node.children.is_empty()) {
            self.close_rail_submenu();
            self.activate_node(node_id, cx);
        } else {
            self.set_open_rail_submenu(node_id);
            cx.notify();
        }
    }

    fn handle_rail_node_bounds(
        &mut self,
        node_id: SharedString,
        bounds: &Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.rail_node_bounds.insert(node_id, bounds.clone());
    }

    fn handle_rail_submenu_mouse_down_out(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_rail_submenu.is_some() {
            self.close_rail_submenu();
            cx.notify();
        }
    }

    fn handle_rail_submenu_item_hover(
        &mut self,
        parent_node_id: SharedString,
        index: usize,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if !hovered || self.open_rail_submenu.as_ref() != Some(&parent_node_id) {
            return;
        }

        let open_submenu = self
            .find_top_level_main_node(&parent_node_id)
            .and_then(|parent| parent.children.iter().filter(|node| node.visible).nth(index))
            .is_some_and(|node| node.enabled && !node.children.is_empty())
            .then_some(index);
        let active_path = Some(MenuPath::Root(index));

        if self.rail_submenu_open_submenu != open_submenu || self.rail_submenu_active_path != active_path {
            self.rail_submenu_open_submenu = open_submenu;
            self.rail_submenu_active_path = active_path;
            cx.notify();
        }
    }

    fn handle_rail_submenu_item_click(&mut self, node_id: SharedString, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.close_rail_submenu();
        self.activate_node(node_id, cx);
        cx.notify();
    }

    fn activate_node(&mut self, node_id: SharedString, cx: &mut Context<Self>) {
        let Some(node) = self.find_node(&node_id) else {
            return;
        };
        if !node.enabled || node.kind != NavNodeKind::Item || !node.children.is_empty() {
            return;
        }

        let label = node.label.clone().unwrap_or_else(|| node_id.clone());
        if self.model.selected_id.as_ref() != Some(&node_id) {
            self.model.selected_id = Some(node_id.clone());
            cx.notify();
        }

        cx.emit(NavigationSidebarEvent::Activate { node_id, label });
    }

    fn focused_target(&self, window: &Window) -> Option<NavigationFocusTarget> {
        self.visible_focus_nodes
            .iter()
            .find_map(|(target, focus_handle)| focus_handle.is_focused(window).then(|| target.clone()))
    }

    fn move_focus(&self, window: &mut Window, cx: &mut Context<Self>, direction: FocusDirection) {
        let Some(next_focus_handle) = next_focus_handle(&self.visible_focus_nodes, window, direction) else {
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

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, window: &mut Window, cx: &mut Context<Self>) {
        let Some(NavigationFocusTarget::Node(node_id)) = self.focused_target(window) else {
            return;
        };

        if self.model.collapsed {
            return;
        };

        self.set_node_expanded(node_id, true, cx);
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, window: &mut Window, cx: &mut Context<Self>) {
        let Some(NavigationFocusTarget::Node(node_id)) = self.focused_target(window) else {
            return;
        };

        if self.model.collapsed {
            return;
        };

        self.set_node_expanded(node_id, false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.focused_target(window) else {
            return;
        };

        match target {
            NavigationFocusTarget::CollapseTrigger => self.toggle_collapsed(cx),
            NavigationFocusTarget::Node(node_id) if self.model.collapsed => {
                if self.find_top_level_main_node(&node_id).is_some_and(|node| node.children.is_empty()) {
                    self.close_rail_submenu();
                    self.activate_node(node_id, cx);
                } else {
                    self.set_open_rail_submenu(node_id);
                    cx.notify();
                }
            }
            NavigationFocusTarget::Node(node_id)
                if self.find_node(&node_id).is_some_and(|node| !node.children.is_empty()) =>
            {
                self.toggle_node_expanded(node_id, cx);
            }
            NavigationFocusTarget::Node(node_id) => self.activate_node(node_id, cx),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NavigationFocusTarget {
    CollapseTrigger,
    Node(SharedString),
}

impl Render for NavigationSidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window, cx);
        let handlers = self.template_handlers(&model, cx);
        self.main_scroll.sync_scrollbar(cx);

        div()
            .size_full()
            .key_context(ControlKeyProfile::Navigation.context())
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

fn render_nodes(
    nodes: &[NavNode],
    depth: usize,
    sidebar_collapsed: bool,
    selected_id: Option<&SharedString>,
    hovered_node: Option<&SharedString>,
    pressed_node: Option<&SharedString>,
    row_focus_handles: &mut HashMap<SharedString, FocusHandle>,
    visible_focus_nodes: &mut Vec<(NavigationFocusTarget, FocusHandle)>,
    window: &mut Window,
    cx: &mut Context<NavigationSidebar>,
) -> Vec<RenderedNavNode> {
    let sibling_count = nodes.iter().filter(|node| node.visible).count();
    let mut visible_index = 0;
    let mut rendered_nodes = Vec::new();

    for node in nodes {
        if !node.visible {
            continue;
        }

        let has_default_interaction = node.kind == NavNodeKind::Item && node.enabled && node.presenter.is_none();
        let focus_handle = has_default_interaction.then(|| {
            let focus_handle = row_focus_handles.entry(node.id.clone()).or_insert_with(|| cx.focus_handle()).clone();
            let focus_handle = focus_handle.tab_stop(true);
            row_focus_handles.insert(node.id.clone(), focus_handle.clone());
            visible_focus_nodes.push((NavigationFocusTarget::Node(node.id.clone()), focus_handle.clone()));
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
            enabled: node.enabled,
        };
        visible_index += 1;

        let custom_content = node.presenter.as_ref().map(|presenter| presenter.present(&state, window, cx));
        if node.enabled
            && let Some(focus_handle) = custom_content.as_ref().and_then(|content| content.focus_handle.as_ref())
        {
            visible_focus_nodes.push((NavigationFocusTarget::Node(node.id.clone()), focus_handle.clone()));
        }

        let has_children = !node.children.is_empty();
        let children = if node.expanded {
            render_nodes(
                &node.children,
                depth + 1,
                sidebar_collapsed,
                selected_id,
                hovered_node,
                pressed_node,
                row_focus_handles,
                visible_focus_nodes,
                window,
                cx,
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
            children,
        });
    }

    rendered_nodes
}

fn render_collapsed_rail_nodes(
    nodes: &[NavNode],
    selected_id: Option<&SharedString>,
    hovered_node: Option<&SharedString>,
    pressed_node: Option<&SharedString>,
    rail_focus_handles: &mut HashMap<SharedString, FocusHandle>,
    visible_focus_nodes: &mut Vec<(NavigationFocusTarget, FocusHandle)>,
    window: &mut Window,
    cx: &mut Context<NavigationSidebar>,
) -> Vec<RenderedNavNode> {
    let sibling_count = collapsed_rail_node_count(nodes);
    let mut visible_index = 0;
    let mut rendered_nodes = Vec::new();

    for node in nodes.iter().filter(|node| collapsed_rail_node_visible(node)) {
        let focus_handle = node.enabled.then(|| {
            let focus_handle = rail_focus_handles.entry(node.id.clone()).or_insert_with(|| cx.focus_handle()).clone();
            let focus_handle = focus_handle.tab_stop(true);
            rail_focus_handles.insert(node.id.clone(), focus_handle.clone());
            visible_focus_nodes.push((NavigationFocusTarget::Node(node.id.clone()), focus_handle.clone()));
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
            enabled: node.enabled,
        };
        visible_index += 1;

        rendered_nodes.push(RenderedNavNode {
            id: node.id.clone(),
            kind: node.kind,
            label: node.label.clone(),
            icon: node.icon,
            state,
            custom_element: None,
            focus_handle,
            has_children: !node.children.is_empty(),
            children: Vec::new(),
        });
    }

    rendered_nodes
}

fn collapsed_rail_node_count(nodes: &[NavNode]) -> usize {
    nodes.iter().filter(|node| collapsed_rail_node_visible(node)).count()
}

fn collapsed_rail_node_visible(node: &NavNode) -> bool {
    node.visible && node.kind == NavNodeKind::Item && node.icon.is_some()
}

fn nav_node_to_menu_item(node: &NavNode) -> MenuItem {
    let label = node.label.clone().unwrap_or_else(|| node.id.clone());
    let mut item = MenuItem::new(node.id.clone()).label(label).enabled(node.enabled);

    if let Some(icon) = node.icon {
        item = item.icon(icon);
    }

    let submenu = node.children.iter().filter(|child| child.visible).map(nav_node_to_menu_item).collect::<Vec<_>>();
    if !submenu.is_empty() {
        item = item.submenu(submenu);
    }

    item
}

fn nav_menu_click_node_ids(nodes: &[NavNode], open_submenu: Option<usize>) -> Vec<SharedString> {
    let visible_nodes = nodes.iter().filter(|node| node.visible);
    let mut ids = Vec::new();

    for (index, node) in visible_nodes.enumerate() {
        if !node.enabled {
            continue;
        }

        if node.children.is_empty() {
            ids.push(node.id.clone());
        } else if open_submenu == Some(index) {
            ids.extend(
                node.children
                    .iter()
                    .filter(|child| child.visible && child.enabled && child.children.is_empty())
                    .map(|child| child.id.clone()),
            );
        }
    }

    ids
}

#[derive(Clone, Copy)]
enum SidebarNodeInteraction {
    Default,
    Rail,
}

fn push_template_handlers(
    nodes: &[RenderedNavNode],
    handlers: &mut NavigationSidebarTemplateHandlers,
    cx: &mut Context<NavigationSidebar>,
    interaction: SidebarNodeInteraction,
) {
    for node in nodes {
        let hover_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                handlers.row_hovers.push(Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_node_hover(hover_id.clone(), *hovered, cx);
                })));
            }
            SidebarNodeInteraction::Rail => {
                handlers.row_hovers.push(Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_rail_node_hover(hover_id.clone(), *hovered, cx);
                })));
            }
        }

        if matches!(interaction, SidebarNodeInteraction::Rail) {
            let bounds_id = node.id.clone();
            handlers.row_bounds.push(Box::new(cx.listener(move |this, bounds, window, cx| {
                this.handle_rail_node_bounds(bounds_id.clone(), bounds, window, cx);
            })));
        }

        let mouse_down_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                handlers.row_mouse_downs.push(Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_node_mouse_down(mouse_down_id.clone(), event, window, cx);
                })));
            }
            SidebarNodeInteraction::Rail => {
                handlers.row_mouse_downs.push(Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_rail_node_mouse_down(mouse_down_id.clone(), event, window, cx);
                })));
            }
        }

        handlers.row_mouse_ups.push(Box::new(cx.listener(NavigationSidebar::handle_node_mouse_up)));
        handlers.row_mouse_up_outs.push(Box::new(cx.listener(NavigationSidebar::handle_node_mouse_up)));

        let click_id = node.id.clone();
        match interaction {
            SidebarNodeInteraction::Default => {
                handlers.row_clicks.push(Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_node_click(click_id.clone(), event, window, cx);
                })));
            }
            SidebarNodeInteraction::Rail => {
                handlers.row_clicks.push(Box::new(cx.listener(move |this, event, window, cx| {
                    this.handle_rail_node_click(click_id.clone(), event, window, cx);
                })));
            }
        }

        push_template_handlers(&node.children, handlers, cx, interaction);
    }
}

#[derive(Clone, Copy)]
enum FocusDirection {
    Previous,
    Next,
    First,
    Last,
}

fn next_focus_handle(
    focus_nodes: &[(NavigationFocusTarget, FocusHandle)],
    window: &Window,
    direction: FocusDirection,
) -> Option<FocusHandle> {
    if focus_nodes.is_empty() {
        return None;
    }

    match direction {
        FocusDirection::First => focus_nodes.first().map(|(_, focus_handle)| focus_handle.clone()),
        FocusDirection::Last => focus_nodes.last().map(|(_, focus_handle)| focus_handle.clone()),
        FocusDirection::Previous | FocusDirection::Next => {
            let current_index = focus_nodes.iter().position(|(_, focus_handle)| focus_handle.is_focused(window));
            let next_index = match (current_index, direction) {
                (Some(index), FocusDirection::Previous) => index.checked_sub(1).unwrap_or(focus_nodes.len() - 1),
                (Some(index), FocusDirection::Next) => (index + 1) % focus_nodes.len(),
                (None, FocusDirection::Previous) => focus_nodes.len() - 1,
                (None, FocusDirection::Next) => 0,
                _ => unreachable!(),
            };

            focus_nodes.get(next_index).map(|(_, focus_handle)| focus_handle.clone())
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

fn find_node_in_nodes<'a>(nodes: &'a [NavNode], node_id: &SharedString) -> Option<&'a NavNode> {
    for node in nodes {
        if node.id == *node_id {
            return Some(node);
        }

        if let Some(node) = find_node_in_nodes(&node.children, node_id) {
            return Some(node);
        }
    }

    None
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
    use lucide_icons::Icon as LucideIcon;

    use super::{collapsed_rail_node_count, set_expanded_in_nodes, toggle_expanded_in_nodes};
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

    #[test]
    fn collapsed_rail_only_counts_visible_top_level_item_nodes_with_icons() {
        let nodes = vec![
            NavNode::section("section", "Section").icon(LucideIcon::List),
            NavNode::new("icon-leaf").icon(LucideIcon::House),
            NavNode::new("icon-branch")
                .icon(LucideIcon::Settings)
                .child(NavNode::new("child").icon(LucideIcon::Circle)),
            NavNode::new("label-only"),
            NavNode::new("hidden").icon(LucideIcon::Search).visible(false),
        ];

        assert_eq!(collapsed_rail_node_count(&nodes), 2);
    }
}
