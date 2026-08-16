use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui::{
    Bounds, ClickEvent, Context, EventEmitter, FocusHandle, FocusOutEvent, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Render, SharedString, Size, Subscription, Window, div, prelude::*,
};

use super::{
    NavNode, NavNodeKind, NavNodeState, SidebarPanelEngineBuilder, SidebarPanelEngineModel,
    SidebarPanelEngineRenderModel, SidebarPanelTemplateHandlers, RenderedNavNode, RenderedRailSubmenu,
};
use crate::controls::menu_item::MenuItem;
use crate::controls::scroll_container::ScrollContainer;
use crate::controls::scrollbar::ScrollbarEvent;
use crate::controls::state::MenuPath;
use crate::animation::{DEFAULT_TRANSITION_DURATION, VisualTransition};
use crate::controls::overlay_presence::OverlayPresence;
use crate::keyhandling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SidebarPanelEngineEvent {
    Activate { node_id: SharedString, label: SharedString },
    BranchExpandedChanged { node_id: SharedString, expanded: bool },
    CollapsedChanged { collapsed: bool },
    FocusChanged { focused: bool },
    ItemFocused { node_id: SharedString, label: SharedString },
    ItemHoverChanged { node_id: SharedString, hovered: bool },
    RailSubmenuOpenChanged { node_id: Option<SharedString> },
    EnabledChanged { enabled: bool },
}

pub struct SidebarPanelEngine {
    model: SidebarPanelEngineModel,
    main_scroll: ScrollContainer,
    hovered_node: Option<SharedString>,
    pressed_node: Option<SharedString>,
    row_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_focus_handles: HashMap<SharedString, FocusHandle>,
    rail_node_bounds: HashMap<SharedString, Bounds<Pixels>>,
    open_rail_submenu: Option<SharedString>,
    rail_submenu_open_submenu: Option<usize>,
    rail_submenu_active_path: Option<MenuPath>,
    rail_submenu_presence: OverlayPresence,
    rail_submenu_content: Option<RenderedRailSubmenu>,
    rail_submenu_content_size: Option<Size<Pixels>>,
    visible_focus_nodes: Vec<(NavigationFocusTarget, FocusHandle)>,
    focus_subscriptions: Vec<Subscription>,
    focus_subscription_keys: Vec<(NavigationFocusTarget, FocusHandle)>,
    emitted_focused: bool,
    _subscriptions: Vec<Subscription>,
    branch_transitions: HashMap<SharedString, VisualTransition>,
    branch_heights_px: HashMap<SharedString, f32>,
}

impl EventEmitter<SidebarPanelEngineEvent> for SidebarPanelEngine {}

impl SidebarPanelEngine {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SidebarPanelEngineBuilder {
        SidebarPanelEngineBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SidebarPanelEngineBuilder, cx: &mut Context<Self>) -> Self {
        let animated = builder.model.animated;
        let main_scroll = ScrollContainer::new(
            format!("{}-main-scroll", builder.model.id),
            builder.model.scrollbar_template.clone(),
            cx,
        )
        .placement(builder.model.scrollbar_placement)
        .visibility(builder.model.scrollbar_visibility)
        .auto_hide_activate(builder.model.scrollbar_auto_hide_activate);
        let scrollbar = main_scroll.scrollbar();
        let subscriptions = vec![
            cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
                if let ScrollbarEvent::Change { value } = event {
                    this.main_scroll.set_vertical_offset(*value, cx);
                }
            }),
            observe_theme_revision(cx, |_, cx| cx.notify()),
        ];

        let mut engine = Self {
            model: builder.model,
            main_scroll,
            hovered_node: None,
            pressed_node: None,
            row_focus_handles: HashMap::new(),
            rail_focus_handles: HashMap::new(),
            rail_node_bounds: HashMap::new(),
            open_rail_submenu: None,
            rail_submenu_open_submenu: None,
            rail_submenu_active_path: None,
            rail_submenu_presence: OverlayPresence::new(false, animated),
            rail_submenu_content: None,
            rail_submenu_content_size: None,
            visible_focus_nodes: Vec::new(),
            focus_subscriptions: Vec::new(),
            focus_subscription_keys: Vec::new(),
            emitted_focused: false,
            _subscriptions: subscriptions,
            branch_transitions: HashMap::new(),
            branch_heights_px: HashMap::new(),
        };
        engine.sync_branch_state();
        engine
    }

    pub fn set_header_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.header_nodes = nodes.into_iter().collect();
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_items(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.nodes = nodes.into_iter().collect();
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_footer_nodes(&mut self, nodes: impl IntoIterator<Item = NavNode>, cx: &mut Context<Self>) {
        self.model.footer_nodes = nodes.into_iter().collect();
        self.sync_branch_state();
        self.close_rail_submenu(cx);
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::SidebarPanelTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_scrollbar_template(
        &mut self,
        template: std::sync::Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.scrollbar_template = template.clone();
        self.main_scroll.scrollbar().update(cx, |scrollbar, cx| scrollbar.set_template(template, cx));
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

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_subscriptions.clear();
        self.focus_subscription_keys.clear();
        if !enabled {
            self.clear_hovered_node(cx);
            self.pressed_node = None;
            self.close_rail_submenu(cx);
            self.emit_focus_changed(false, cx);
        }
        cx.emit(SidebarPanelEngineEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.model.collapsed != collapsed {
            self.model.collapsed = collapsed;
            self.clear_hovered_node(cx);
            self.pressed_node = None;
            self.close_rail_submenu(cx);
            cx.emit(SidebarPanelEngineEvent::CollapsedChanged { collapsed });
            cx.notify();
        }
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        self.set_collapsed(!self.model.collapsed, cx);
    }

    pub fn set_node_expanded(&mut self, node_id: impl Into<SharedString>, expanded: bool, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if set_expanded_in_nodes(&mut self.model.nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.header_nodes, &node_id, expanded)
            || set_expanded_in_nodes(&mut self.model.footer_nodes, &node_id, expanded)
        {
            self.set_branch_target(&node_id, expanded);
            cx.emit(SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded });
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

        self.set_branch_target(&node_id, expanded);
        cx.emit(SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded });
        cx.notify();
    }

    fn transition_duration(&self) -> Duration {
        if self.model.animated {
            DEFAULT_TRANSITION_DURATION
        } else {
            Duration::ZERO
        }
    }

    fn sync_branch_state(&mut self) {
        let mut branch_ids = HashSet::new();
        fn collect(nodes: &[NavNode], ids: &mut HashSet<SharedString>) {
            for node in nodes {
                if !node.children.is_empty() {
                    ids.insert(node.id.clone());
                }
                collect(&node.children, ids);
            }
        }
        collect(&self.model.header_nodes, &mut branch_ids);
        collect(&self.model.nodes, &mut branch_ids);
        collect(&self.model.footer_nodes, &mut branch_ids);
        self.branch_transitions.retain(|id, _| branch_ids.contains(id));
        self.branch_heights_px.retain(|id, _| branch_ids.contains(id));
        let duration = self.transition_duration();
        for id in branch_ids {
            let expanded = self.find_node(&id).is_some_and(|node| node.expanded);
            self.branch_transitions
                .entry(id)
                .or_insert_with(|| VisualTransition::new(if expanded { 1.0 } else { 0.0 }, duration));
        }
    }

    fn set_branch_target(&mut self, node_id: &SharedString, expanded: bool) {
        let duration = self.transition_duration();
        let transition = self
            .branch_transitions
            .entry(node_id.clone())
            .or_insert_with(|| VisualTransition::new(if expanded { 0.0 } else { 1.0 }, duration));
        if self.model.animated {
            transition.set_target(if expanded { 1.0 } else { 0.0 });
        } else {
            transition.snap_to(if expanded { 1.0 } else { 0.0 });
        }
    }

    fn sync_branch_transitions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut animating = false;
        for transition in self.branch_transitions.values_mut() {
            transition.sync();
            animating |= transition.is_animating();
        }
        if animating {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
    }

    fn set_branch_height(&mut self, node_id: SharedString, height: f32, cx: &mut Context<Self>) {
        let height = height.max(0.0);
        let settled_open = self
            .branch_transitions
            .get(&node_id)
            .is_some_and(|transition| !transition.is_animating() && transition.progress() >= 1.0 - f32::EPSILON);
        let cached = self.branch_heights_px.entry(node_id).or_insert(0.0);
        if (settled_open || height > *cached + 0.5) && (height - *cached).abs() > 0.5 {
            *cached = height;
            cx.notify();
        }
    }

    fn render_model(&mut self, window: &mut Window, cx: &mut Context<Self>) -> SidebarPanelEngineRenderModel {
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
            content.presence = self.rail_submenu_presence;
            content.content_size = self.rail_submenu_content_size;
        }
        let rail_submenu =
            self.rail_submenu_presence.should_paint().then(|| self.rail_submenu_content.clone()).flatten();

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

    fn template_handlers(
        &self,
        model: &SidebarPanelEngineRenderModel,
        cx: &mut Context<Self>,
    ) -> SidebarPanelTemplateHandlers {
        let mut handlers = SidebarPanelTemplateHandlers::default();

        if model.collapsed {
            push_template_handlers(&model.rail_nodes, &mut handlers, cx, SidebarNodeInteraction::Rail);
            push_template_handlers(&model.rail_footer_nodes, &mut handlers, cx, SidebarNodeInteraction::Rail);

            if let Some(rail_submenu) = &model.rail_submenu {
                handlers.rail_submenu_bounds =
                    Some(Box::new(cx.listener(|this, bounds: &Bounds<Pixels>, _window, cx| {
                        if this.rail_submenu_content_size != Some(bounds.size) {
                            this.rail_submenu_content_size = Some(bounds.size);
                            cx.notify();
                        }
                    })));
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
            push_children_height_handlers(&model.header_nodes, &mut handlers, cx);
            push_children_height_handlers(&model.nodes, &mut handlers, cx);
            push_children_height_handlers(&model.footer_nodes, &mut handlers, cx);
            push_template_handlers(&model.header_nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
            push_template_handlers(&model.nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
            push_template_handlers(&model.footer_nodes, &mut handlers, cx, SidebarNodeInteraction::Default);
        }

        handlers
    }

    fn can_interact_with_default_node(&self, node_id: &SharedString) -> bool {
        self.model.enabled
            && self
                .find_node(node_id)
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
        self.model.enabled
            && self
                .find_top_level_main_node(node_id)
                .is_some_and(|node| collapsed_rail_node_visible(node) && node.enabled)
    }

    fn close_rail_submenu(&mut self, cx: &mut Context<Self>) -> bool {
        if self.open_rail_submenu.is_none()
            && self.rail_submenu_open_submenu.is_none()
            && self.rail_submenu_active_path.is_none()
            && !self.rail_submenu_presence.is_animating()
        {
            return false;
        }

        self.open_rail_submenu = None;
        self.rail_submenu_open_submenu = None;
        self.rail_submenu_active_path = None;
        self.rail_submenu_presence.set_open(false);
        cx.emit(SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id: None });
        true
    }

    fn set_open_rail_submenu(&mut self, node_id: SharedString, cx: &mut Context<Self>) {
        if self.open_rail_submenu.as_ref() == Some(&node_id) {
            self.close_rail_submenu(cx);
        } else {
            self.open_rail_submenu = Some(node_id.clone());
            self.rail_submenu_open_submenu = None;
            self.rail_submenu_active_path = None;
            self.rail_submenu_presence.set_open(true);
            cx.emit(SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id: Some(node_id) });
        }
    }

    fn rendered_rail_submenu(&self) -> Option<RenderedRailSubmenu> {
        let parent_node_id = self.open_rail_submenu.as_ref()?;
        let parent = self.find_top_level_main_node(parent_node_id)?;
        if parent.children.is_empty() || !collapsed_rail_node_visible(parent) {
            return None;
        }

        let parent_bounds = *self.rail_node_bounds.get(parent_node_id)?;
        let items = parent.children.iter().filter(|node| node.visible).map(nav_node_to_menu_item).collect();

        Some(RenderedRailSubmenu {
            id: format!("{}-rail-submenu-{}", self.model.id, parent_node_id).into(),
            parent_node_id: parent_node_id.clone(),
            parent_bounds,
            items,
            open_submenu: self.rail_submenu_open_submenu,
            active_path: self.rail_submenu_active_path,
            presence: self.rail_submenu_presence,
            content_size: self.rail_submenu_content_size,
        })
    }

    fn rail_submenu_click_node_ids(&self, parent_node_id: &SharedString) -> Vec<SharedString> {
        let Some(parent) = self.find_top_level_main_node(parent_node_id) else {
            return Vec::new();
        };

        nav_menu_click_node_ids(&parent.children, self.rail_submenu_open_submenu)
    }

    fn handle_node_hover(&mut self, node_id: SharedString, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_interact_with_default_node(&node_id) {
            return;
        }

        if hovered {
            if self.hovered_node.as_ref() != Some(&node_id) {
                self.set_hovered_node(Some(node_id), cx);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.set_hovered_node(None, cx);
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
                self.set_hovered_node(Some(node_id), cx);
                cx.notify();
            }
        } else if self.hovered_node.as_ref() == Some(&node_id) {
            self.set_hovered_node(None, cx);
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
            self.close_rail_submenu(cx);
            self.activate_node(node_id, cx);
        } else {
            self.set_open_rail_submenu(node_id, cx);
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
        self.rail_node_bounds.insert(node_id, *bounds);
    }

    fn handle_rail_submenu_mouse_down_out(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_rail_submenu.is_some() {
            self.close_rail_submenu(cx);
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

        self.close_rail_submenu(cx);
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

        cx.emit(SidebarPanelEngineEvent::Activate { node_id, label });
    }

    fn set_hovered_node(&mut self, hovered_node: Option<SharedString>, cx: &mut Context<Self>) {
        if self.hovered_node == hovered_node {
            return;
        }

        if let Some(node_id) = self.hovered_node.take() {
            cx.emit(SidebarPanelEngineEvent::ItemHoverChanged { node_id, hovered: false });
        }

        if let Some(node_id) = hovered_node {
            cx.emit(SidebarPanelEngineEvent::ItemHoverChanged { node_id: node_id.clone(), hovered: true });
            self.hovered_node = Some(node_id);
        }
    }

    fn clear_hovered_node(&mut self, cx: &mut Context<Self>) {
        self.set_hovered_node(None, cx);
    }

    fn node_label(&self, node_id: &SharedString) -> SharedString {
        self.find_node(node_id).and_then(|node| node.label.clone()).unwrap_or_else(|| node_id.clone())
    }

    fn focused_target(&self, window: &Window) -> Option<NavigationFocusTarget> {
        self.visible_focus_nodes
            .iter()
            .find_map(|(target, focus_handle)| focus_handle.is_focused(window).then(|| target.clone()))
    }

    fn move_focus(&mut self, window: &mut Window, cx: &mut Context<Self>, direction: FocusDirection) {
        let Some((target, next_focus_handle)) = next_focus_target(&self.visible_focus_nodes, window, direction) else {
            return;
        };

        next_focus_handle.focus(window, cx);
        self.activate_node(target, cx);
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
        let Some(node_id) = self.focused_target(window) else {
            return;
        };

        if self.model.collapsed {
            return;
        };

        self.set_node_expanded(node_id, true, cx);
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.model.collapsed {
            if self.open_rail_submenu.is_some() {
                self.close_rail_submenu(cx);
                cx.notify();
            }
            return;
        }

        let Some(node_id) = self.focused_target(window) else {
            return;
        };

        self.set_node_expanded(node_id, false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let Some(node_id) = self.focused_target(window) else {
            return;
        };

        if self.model.collapsed {
            if self.find_top_level_main_node(&node_id).is_some_and(|node| node.children.is_empty()) {
                self.close_rail_submenu(cx);
                self.activate_node(node_id, cx);
            } else {
                self.set_open_rail_submenu(node_id, cx);
                cx.notify();
            }
        } else if self.find_node(&node_id).is_some_and(|node| !node.children.is_empty()) {
            self.toggle_node_expanded(node_id, cx);
        } else {
            self.activate_node(node_id, cx);
        }
    }

    fn sync_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.visible_focus_nodes == self.focus_subscription_keys {
            return;
        }

        self.focus_subscriptions.clear();
        self.focus_subscription_keys = self.visible_focus_nodes.clone();

        for (target, focus_handle) in self.focus_subscription_keys.clone() {
            self.focus_subscriptions.push(cx.on_focus_in(&focus_handle, window, {
                let node_id = target.clone();
                move |this, _window, cx| {
                    let focus_changed = this.emit_focus_changed(true, cx);
                    cx.emit(SidebarPanelEngineEvent::ItemFocused {
                        node_id: node_id.clone(),
                        label: this.node_label(&node_id),
                    });
                    if focus_changed {
                        cx.notify();
                    }
                }
            }));
            self.focus_subscriptions.push(cx.on_focus_out(
                &focus_handle,
                window,
                move |this, _: FocusOutEvent, window, cx| {
                    if !this.any_focus_target_focused(window) && this.emit_focus_changed(false, cx) {
                        cx.notify();
                    }
                },
            ));
        }
    }

    fn any_focus_target_focused(&self, window: &Window) -> bool {
        self.focus_subscription_keys.iter().any(|(_, focus_handle)| focus_handle.is_focused(window))
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(SidebarPanelEngineEvent::FocusChanged { focused });
        true
    }
}

type NavigationFocusTarget = SharedString;

impl Render for SidebarPanelEngine {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_rail_submenu_animating = self.rail_submenu_presence.is_animating();
        let is_rail_submenu_animating = self.rail_submenu_presence.sync();
        self.rail_submenu_presence.schedule_frame(window, cx);
        if was_rail_submenu_animating || is_rail_submenu_animating {
            cx.notify();
        }
        if !self.rail_submenu_presence.should_paint() {
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

fn branch_progress_for(
    transitions: &HashMap<SharedString, VisualTransition>,
    node_id: &SharedString,
    expanded: bool,
) -> f32 {
    transitions.get(node_id).map(VisualTransition::progress).unwrap_or(if expanded { 1.0 } else { 0.0 })
}

fn branch_visible_for(
    transitions: &HashMap<SharedString, VisualTransition>,
    node_id: &SharedString,
    expanded: bool,
) -> bool {
    branch_progress_for(transitions, node_id, expanded) > f32::EPSILON
        || transitions.get(node_id).is_some_and(VisualTransition::is_animating)
}

#[allow(clippy::too_many_arguments)]
fn render_nodes(
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
fn render_collapsed_rail_nodes(
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
    handlers: &mut SidebarPanelTemplateHandlers,
    cx: &mut Context<SidebarPanelEngine>,
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

        handlers.row_mouse_ups.push(Box::new(cx.listener(SidebarPanelEngine::handle_node_mouse_up)));
        handlers.row_mouse_up_outs.push(Box::new(cx.listener(SidebarPanelEngine::handle_node_mouse_up)));

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

fn push_children_height_handlers(
    nodes: &[RenderedNavNode],
    handlers: &mut SidebarPanelTemplateHandlers,
    cx: &mut Context<SidebarPanelEngine>,
) {
    for node in nodes {
        if node.has_children {
            let node_id = node.id.clone();
            handlers.children_height_reports.insert(
                node_id.clone(),
                Box::new(cx.listener(move |this, height, _window, cx| {
                    this.set_branch_height(node_id.clone(), *height, cx);
                })),
            );
        }
        push_children_height_handlers(&node.children, handlers, cx);
    }
}

#[derive(Clone, Copy)]
enum FocusDirection {
    Previous,
    Next,
    First,
    Last,
}

fn next_focus_target(
    focus_nodes: &[(NavigationFocusTarget, FocusHandle)],
    window: &Window,
    direction: FocusDirection,
) -> Option<(NavigationFocusTarget, FocusHandle)> {
    if focus_nodes.is_empty() {
        return None;
    }

    match direction {
        FocusDirection::First => focus_nodes.first().cloned(),
        FocusDirection::Last => focus_nodes.last().cloned(),
        FocusDirection::Previous | FocusDirection::Next => {
            let current_index = focus_nodes.iter().position(|(_, focus_handle)| focus_handle.is_focused(window));
            let next_index = match (current_index, direction) {
                (Some(index), FocusDirection::Previous) => index.checked_sub(1).unwrap_or(focus_nodes.len() - 1),
                (Some(index), FocusDirection::Next) => (index + 1) % focus_nodes.len(),
                (None, FocusDirection::Previous) => focus_nodes.len() - 1,
                (None, FocusDirection::Next) => 0,
                _ => unreachable!(),
            };

            focus_nodes.get(next_index).cloned()
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
    use std::collections::HashMap;
    use std::time::Duration;

    use gpui::SharedString;
    use lucide_icons::Icon as LucideIcon;

    use crate::animation::VisualTransition;

    use super::{
        branch_progress_for, branch_visible_for, collapsed_rail_node_count, set_expanded_in_nodes,
        toggle_expanded_in_nodes,
    };
    use super::NavNode;

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

    #[test]
    fn closing_branch_remains_visible_while_transition_is_active() {
        let branch_id: SharedString = "inputs".into();
        let mut transitions =
            HashMap::from([(branch_id.clone(), VisualTransition::new(1.0, Duration::from_millis(200)))]);
        transitions.get_mut(&branch_id).unwrap().set_target(0.0);

        assert!(branch_visible_for(&transitions, &branch_id, false));
        assert!(branch_progress_for(&transitions, &branch_id, false) > 0.0);
    }
}
