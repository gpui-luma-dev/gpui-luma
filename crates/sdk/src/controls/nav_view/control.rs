use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render,
    SharedString, Window, div, prelude::*,
};

use super::{
    NavButton, NavButtonRenderModel, NavButtonTemplateHandlers, NavItem, NavItemState, NavNodeItemRenderModel,
    NavNodeItemTemplateHandlers, NavNodeRenderModel, NavNodeTemplateHandlers, NavRenderItem,
    NavRenderItemTemplateHandlers, NavViewBuilder, NavViewRenderModel, NavViewTemplateHandlers,
};
use crate::controls::nav_view::model::{NavLabelRenderModel, NavViewModel};
use crate::controls::state::ControlFocusState;
use crate::keyhandling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum NavViewEvent {
    Activate { item_id: SharedString },
    ToggleNode { node_id: SharedString, expanded: bool },
}

pub struct NavView {
    model: NavViewModel,
    focus_handle: gpui::FocusHandle,
    hovered_path: Option<NavPath>,
    pressed_path: Option<NavPath>,
    active_path: Option<NavPath>,
}

impl EventEmitter<NavViewEvent> for NavView {}

impl NavView {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> NavViewBuilder {
        NavViewBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: NavViewBuilder, cx: &mut Context<Self>) -> Self {
        normalize_selected_item_id(&mut builder.model);
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            hovered_path: None,
            pressed_path: None,
            active_path: None,
        }
    }

    pub fn selected_item_id(&self) -> Option<&SharedString> {
        self.model.selected_item_id.as_ref()
    }

    pub fn set_selected(&mut self, selected_item_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_item_id = selected_item_id.into();
        if self.can_select_id(&selected_item_id) {
            self.model.selected_item_id = Some(selected_item_id);
            cx.notify();
        }
    }

    pub fn clear_selected(&mut self, cx: &mut Context<Self>) {
        if self.model.selected_item_id.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = impl Into<NavItem>>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().map(Into::into).collect();
        self.hovered_path = None;
        self.pressed_path = None;
        self.active_path = self.active_path.filter(|path| self.path_is_visible_and_enabled(*path));
        normalize_selected_item_id(&mut self.model);
        cx.notify();
    }

    pub fn set_bottom_items(&mut self, items: impl IntoIterator<Item = NavButton>, cx: &mut Context<Self>) {
        self.model.bottom_items = items.into_iter().collect();
        self.hovered_path = None;
        self.pressed_path = None;
        self.active_path = self.active_path.filter(|path| self.path_is_visible_and_enabled(*path));
        normalize_selected_item_id(&mut self.model);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);

        if !enabled {
            self.hovered_path = None;
            self.pressed_path = None;
            self.active_path = None;
        }

        cx.notify();
    }

    pub fn set_node_expanded(&mut self, node_id: impl Into<SharedString>, expanded: bool, cx: &mut Context<Self>) {
        let node_id = node_id.into();

        for item in &mut self.model.items {
            let NavItem::Node(node) = item else {
                continue;
            };

            if node.id == node_id && node.expanded != expanded {
                node.expanded = expanded;
                self.active_path = self.active_path.filter(|path| self.path_is_visible_and_enabled(*path));
                cx.notify();
                return;
            }
        }
    }

    fn render_model<'a>(&'a self, window: &Window) -> NavViewRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let active_path = focus.focused.then(|| self.effective_active_path()).flatten();
        let mut items = Vec::new();

        for (index, item) in self.model.items.iter().enumerate() {
            match item {
                NavItem::Button(button) => {
                    let path = NavPath::Top(index);
                    items.push(NavRenderItem::Button(self.button_render_model(button, path, 0, focus, active_path)));
                }
                NavItem::Label(label) => {
                    items.push(NavRenderItem::Label(NavLabelRenderModel { label: &label.label, depth: 0 }));
                }
                NavItem::Node(node) => {
                    let path = NavPath::Top(index);
                    let enabled = self.model.enabled && node.enabled;
                    let active = enabled && active_path == Some(path);
                    let mut children = Vec::new();

                    if node.expanded {
                        for (child_index, child) in node.children.iter().enumerate() {
                            let child_path = NavPath::Child { parent: index, child: child_index };
                            let child_enabled = self.model.enabled && child.enabled;
                            let selected = self
                                .model
                                .selected_item_id
                                .as_ref()
                                .is_some_and(|selected_id| selected_id == &child.id);
                            let child_active = child_enabled && active_path == Some(child_path);

                            children.push(NavNodeItemRenderModel {
                                id: &child.id,
                                label: &child.label,
                                selected,
                                active: child_active,
                                enabled: child_enabled,
                                depth: 1,
                                state: self.item_state(child_path, child_enabled, selected, child_active, focus),
                            });
                        }
                    }

                    items.push(NavRenderItem::Node(NavNodeRenderModel {
                        id: &node.id,
                        label: &node.label,
                        expanded: node.expanded,
                        active,
                        enabled,
                        depth: 0,
                        state: self.item_state(path, enabled, false, active, focus),
                        children,
                    }));
                }
            }
        }

        let bottom_items = self
            .model
            .bottom_items
            .iter()
            .enumerate()
            .map(|(index, button)| self.button_render_model(button, NavPath::Bottom(index), 0, focus, active_path))
            .collect();

        NavViewRenderModel {
            id: &self.model.id,
            items,
            bottom_items,
            selected_item_id: self.model.selected_item_id.as_ref(),
            active_item_id: active_path.and_then(|path| self.id_for_path(path)),
            enabled: self.model.enabled,
            focus,
            item_template: self.model.item_template.as_ref(),
        }
    }

    fn button_render_model<'a>(
        &'a self,
        button: &'a NavButton,
        path: NavPath,
        depth: usize,
        focus: ControlFocusState,
        active_path: Option<NavPath>,
    ) -> NavButtonRenderModel<'a> {
        let enabled = self.model.enabled && button.enabled;
        let selected = self.model.selected_item_id.as_ref().is_some_and(|selected_id| selected_id == &button.id);
        let active = enabled && active_path == Some(path);

        NavButtonRenderModel {
            id: &button.id,
            label: &button.label,
            selected,
            active,
            enabled,
            depth,
            state: self.item_state(path, enabled, selected, active, focus),
        }
    }

    fn item_state(
        &self,
        path: NavPath,
        enabled: bool,
        selected: bool,
        active: bool,
        focus: ControlFocusState,
    ) -> NavItemState {
        NavItemState {
            disabled: !enabled,
            hovered: enabled && self.hovered_path == Some(path),
            pressed: enabled && self.pressed_path == Some(path),
            selected,
            active,
            focus_visible: enabled && focus.focused && active,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> NavViewTemplateHandlers {
        let mut items = Vec::new();
        let mut bottom_buttons = Vec::new();

        for (index, item) in self.model.items.iter().enumerate() {
            match item {
                NavItem::Button(_) => {
                    items.push(NavRenderItemTemplateHandlers::Button(self.button_handlers(NavPath::Top(index), cx)));
                }
                NavItem::Label(_) => {
                    items.push(NavRenderItemTemplateHandlers::Label);
                }
                NavItem::Node(node) => {
                    let children = if node.expanded {
                        node.children
                            .iter()
                            .enumerate()
                            .map(|(child, _)| self.node_item_handlers(NavPath::Child { parent: index, child }, cx))
                            .collect()
                    } else {
                        Vec::new()
                    };

                    items.push(NavRenderItemTemplateHandlers::Node {
                        node: self.node_handlers(NavPath::Top(index), cx),
                        children,
                    });
                }
            }
        }

        bottom_buttons.extend(
            self.model
                .bottom_items
                .iter()
                .enumerate()
                .map(|(index, _)| self.button_handlers(NavPath::Bottom(index), cx)),
        );

        NavViewTemplateHandlers { items, bottom_buttons }
    }

    fn button_handlers(&self, path: NavPath, cx: &mut Context<Self>) -> NavButtonTemplateHandlers {
        NavButtonTemplateHandlers {
            hover: Box::new(cx.listener(move |this, hovered, _window, cx| {
                this.handle_item_hover(path, *hovered, cx);
            })),
            mouse_down: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_item_mouse_down(path, event, window, cx);
            })),
            mouse_up: Box::new(cx.listener(Self::handle_item_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_item_mouse_up)),
            click: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_activate_click(path, event, window, cx);
            })),
        }
    }

    fn node_handlers(&self, path: NavPath, cx: &mut Context<Self>) -> NavNodeTemplateHandlers {
        NavNodeTemplateHandlers {
            hover: Box::new(cx.listener(move |this, hovered, _window, cx| {
                this.handle_item_hover(path, *hovered, cx);
            })),
            mouse_down: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_item_mouse_down(path, event, window, cx);
            })),
            mouse_up: Box::new(cx.listener(Self::handle_item_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_item_mouse_up)),
            click: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_toggle_click(path, event, window, cx);
            })),
        }
    }

    fn node_item_handlers(&self, path: NavPath, cx: &mut Context<Self>) -> NavNodeItemTemplateHandlers {
        NavNodeItemTemplateHandlers {
            hover: Box::new(cx.listener(move |this, hovered, _window, cx| {
                this.handle_item_hover(path, *hovered, cx);
            })),
            mouse_down: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_item_mouse_down(path, event, window, cx);
            })),
            mouse_up: Box::new(cx.listener(Self::handle_item_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_item_mouse_up)),
            click: Box::new(cx.listener(move |this, event, window, cx| {
                this.handle_activate_click(path, event, window, cx);
            })),
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn handle_item_hover(&mut self, path: NavPath, hovered: bool, cx: &mut Context<Self>) {
        if hovered {
            if self.hovered_path != Some(path) {
                self.hovered_path = Some(path);
                cx.notify();
            }
        } else if self.hovered_path == Some(path) {
            self.hovered_path = None;
            if self.pressed_path == Some(path) {
                self.pressed_path = None;
            }
            cx.notify();
        }
    }

    fn handle_item_mouse_down(
        &mut self,
        path: NavPath,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.path_is_visible_and_enabled(path) {
            return;
        }

        self.pressed_path = Some(path);
        self.active_path = Some(path);
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_path.is_some() {
            self.pressed_path = None;
            cx.notify();
        }
    }

    fn handle_activate_click(
        &mut self,
        path: NavPath,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.activate_path(path, cx);
    }

    fn handle_toggle_click(
        &mut self,
        path: NavPath,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_node_at_path(path, cx);
    }

    fn activate_path(&mut self, path: NavPath, cx: &mut Context<Self>) -> bool {
        if !self.path_is_activatable(path) {
            return false;
        }

        let Some(item_id) = self.id_for_path(path).cloned() else {
            return false;
        };

        self.model.selected_item_id = Some(item_id.clone());
        self.active_path = Some(path);
        cx.emit(NavViewEvent::Activate { item_id });
        cx.notify();
        true
    }

    fn toggle_node_at_path(&mut self, path: NavPath, cx: &mut Context<Self>) -> bool {
        let NavPath::Top(index) = path else {
            return false;
        };

        let Some(NavItem::Node(node)) = self.model.items.get_mut(index) else {
            return false;
        };

        if !self.model.enabled || !node.enabled {
            return false;
        }

        node.expanded = !node.expanded;
        let node_id = node.id.clone();
        let expanded = node.expanded;
        self.active_path = Some(path);
        cx.emit(NavViewEvent::ToggleNode { node_id, expanded });
        cx.notify();
        true
    }

    fn step_active_path(&mut self, direction: NavDirection, cx: &mut Context<Self>) {
        if let Some(next_path) = next_active_path(&self.model, self.active_path, direction)
            && self.active_path != Some(next_path)
        {
            self.active_path = Some(next_path);
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let visible_paths = self.visible_enabled_paths();
        let next_path = if first {
            visible_paths.first().copied()
        } else {
            visible_paths.last().copied()
        };

        if let Some(next_path) = next_path
            && self.active_path != Some(next_path)
        {
            self.active_path = Some(next_path);
            cx.notify();
        }
    }

    fn open_active_node(&mut self, cx: &mut Context<Self>) {
        let Some(NavPath::Top(index)) = self.effective_active_path() else {
            return;
        };

        let Some(NavItem::Node(node)) = self.model.items.get_mut(index) else {
            return;
        };

        if self.model.enabled && node.enabled && !node.expanded {
            node.expanded = true;
            let node_id = node.id.clone();
            cx.emit(NavViewEvent::ToggleNode { node_id, expanded: true });
            cx.notify();
        }
    }

    fn close_active_node(&mut self, cx: &mut Context<Self>) {
        let Some(active_path) = self.effective_active_path() else {
            return;
        };

        let index = match active_path {
            NavPath::Top(index) => index,
            NavPath::Child { parent, .. } => {
                self.active_path = Some(NavPath::Top(parent));
                cx.notify();
                return;
            }
            NavPath::Bottom(_) => return,
        };

        let Some(NavItem::Node(node)) = self.model.items.get_mut(index) else {
            return;
        };

        if self.model.enabled && node.enabled && node.expanded {
            node.expanded = false;
            let node_id = node.id.clone();
            cx.emit(NavViewEvent::ToggleNode { node_id, expanded: false });
            cx.notify();
        }
    }

    fn activate_active_path(&mut self, cx: &mut Context<Self>) {
        let Some(active_path) = self.effective_active_path() else {
            return;
        };

        if self.path_is_node(active_path) {
            self.toggle_node_at_path(active_path, cx);
        } else {
            self.activate_path(active_path, cx);
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.step_active_path(NavDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.step_active_path(NavDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, cx);
    }

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        self.open_active_node(cx);
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        self.close_active_node(cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_active_path(cx);
    }

    fn visible_enabled_paths(&self) -> Vec<NavPath> {
        visible_enabled_paths_for_model(&self.model)
    }

    fn path_is_visible_and_enabled(&self, path: NavPath) -> bool {
        self.visible_enabled_paths().contains(&path)
    }

    fn effective_active_path(&self) -> Option<NavPath> {
        effective_active_path_for_model(&self.model, self.active_path)
    }

    fn path_is_activatable(&self, path: NavPath) -> bool {
        if !self.model.enabled {
            return false;
        }

        match path {
            NavPath::Top(index) => {
                matches!(self.model.items.get(index), Some(NavItem::Button(button)) if button.enabled)
            }
            NavPath::Child { parent, child } => {
                matches!(self.model.items.get(parent), Some(NavItem::Node(node)) if node.expanded && node.children.get(child).is_some_and(|child| child.enabled))
            }
            NavPath::Bottom(index) => self.model.bottom_items.get(index).is_some_and(|button| button.enabled),
        }
    }

    fn path_is_node(&self, path: NavPath) -> bool {
        matches!(path, NavPath::Top(index) if matches!(self.model.items.get(index), Some(NavItem::Node(_))))
    }

    fn can_select_id(&self, selected_item_id: &SharedString) -> bool {
        self.model.items.iter().any(|item| match item {
            NavItem::Button(button) => button.enabled && &button.id == selected_item_id,
            NavItem::Label(_) => false,
            NavItem::Node(node) => node.children.iter().any(|child| child.enabled && &child.id == selected_item_id),
        }) || self.model.bottom_items.iter().any(|button| button.enabled && &button.id == selected_item_id)
    }

    fn id_for_path(&self, path: NavPath) -> Option<&SharedString> {
        match path {
            NavPath::Top(index) => match self.model.items.get(index)? {
                NavItem::Button(button) => Some(&button.id),
                NavItem::Node(node) => Some(&node.id),
                NavItem::Label(_) => None,
            },
            NavPath::Child { parent, child } => match self.model.items.get(parent)? {
                NavItem::Node(node) => Some(&node.children.get(child)?.id),
                _ => None,
            },
            NavPath::Bottom(index) => Some(&self.model.bottom_items.get(index)?.id),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NavPath {
    Top(usize),
    Child { parent: usize, child: usize },
    Bottom(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NavDirection {
    Previous,
    Next,
}

impl Focusable for NavView {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NavView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .size_full()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(&self.focus_handle)
                    .key_context(ControlKeyProfile::Navigation.context())
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                    .on_action(cx.listener(Self::handle_select_previous_item))
                    .on_action(cx.listener(Self::handle_select_next_item))
                    .on_action(cx.listener(Self::handle_select_first_item))
                    .on_action(cx.listener(Self::handle_select_last_item))
                    .on_action(cx.listener(Self::handle_open_submenu))
                    .on_action(cx.listener(Self::handle_close_submenu))
                    .on_action(cx.listener(Self::handle_activate_control)),
            )
            .into_any_element()
    }
}

fn normalize_selected_item_id(model: &mut NavViewModel) {
    let Some(selected_item_id) = model.selected_item_id.as_ref() else {
        return;
    };

    let selected_exists =
        model.items.iter().any(|item| match item {
            NavItem::Button(button) => button.enabled && &button.id == selected_item_id,
            NavItem::Label(_) => false,
            NavItem::Node(node) => node.children.iter().any(|child| child.enabled && &child.id == selected_item_id),
        }) || model.bottom_items.iter().any(|button| button.enabled && &button.id == selected_item_id);

    if !selected_exists {
        model.selected_item_id = None;
    }
}

fn visible_enabled_paths_for_model(model: &NavViewModel) -> Vec<NavPath> {
    if !model.enabled {
        return Vec::new();
    }

    let mut paths = Vec::new();

    for (index, item) in model.items.iter().enumerate() {
        match item {
            NavItem::Button(button) if button.enabled => paths.push(NavPath::Top(index)),
            NavItem::Button(_) | NavItem::Label(_) => {}
            NavItem::Node(node) if node.enabled => {
                paths.push(NavPath::Top(index));

                if node.expanded {
                    paths.extend(
                        node.children
                            .iter()
                            .enumerate()
                            .filter(|(_, child)| child.enabled)
                            .map(|(child, _)| NavPath::Child { parent: index, child }),
                    );
                }
            }
            NavItem::Node(_) => {}
        }
    }

    paths.extend(
        model
            .bottom_items
            .iter()
            .enumerate()
            .filter(|(_, button)| button.enabled)
            .map(|(index, _)| NavPath::Bottom(index)),
    );

    paths
}

fn selected_path_for_model(model: &NavViewModel) -> Option<NavPath> {
    let selected_id = model.selected_item_id.as_ref()?;

    for (index, item) in model.items.iter().enumerate() {
        match item {
            NavItem::Button(button) if button.id == *selected_id && button.enabled => {
                return Some(NavPath::Top(index));
            }
            NavItem::Node(node) => {
                if let Some((child, _)) =
                    node.children.iter().enumerate().find(|(_, child)| child.id == *selected_id && child.enabled)
                {
                    return Some(NavPath::Child { parent: index, child });
                }
            }
            _ => {}
        }
    }

    model
        .bottom_items
        .iter()
        .enumerate()
        .find(|(_, button)| button.id == *selected_id && button.enabled)
        .map(|(index, _)| NavPath::Bottom(index))
}

fn effective_active_path_for_model(model: &NavViewModel, active_path: Option<NavPath>) -> Option<NavPath> {
    if !model.enabled {
        return None;
    }

    let visible_paths = visible_enabled_paths_for_model(model);

    active_path
        .filter(|active| visible_paths.contains(active))
        .or_else(|| selected_path_for_model(model).filter(|selected| visible_paths.contains(selected)))
        .or_else(|| visible_paths.first().copied())
}

fn next_active_path(model: &NavViewModel, active_path: Option<NavPath>, direction: NavDirection) -> Option<NavPath> {
    if !model.enabled {
        return None;
    }

    let visible_paths = visible_enabled_paths_for_model(model);
    if visible_paths.is_empty() {
        return None;
    }

    let current_path = active_path
        .filter(|active| visible_paths.contains(active))
        .or_else(|| selected_path_for_model(model).filter(|selected| visible_paths.contains(selected)));
    let next_index = match current_path.and_then(|active| visible_paths.iter().position(|path| *path == active)) {
        Some(index) => match direction {
            NavDirection::Previous => index.checked_sub(1).unwrap_or(visible_paths.len() - 1),
            NavDirection::Next => (index + 1) % visible_paths.len(),
        },
        None if direction == NavDirection::Previous => visible_paths.len() - 1,
        None => 0,
    };

    Some(visible_paths[next_index])
}

#[cfg(test)]
mod tests {
    use super::{
        NavDirection, NavItem, NavPath, NavViewModel, effective_active_path_for_model, next_active_path,
        normalize_selected_item_id, visible_enabled_paths_for_model,
    };
    use crate::controls::nav_view::{NavButton, NavNodeItem, default_nav_item_template, default_nav_view_template};

    #[test]
    fn normalize_keeps_enabled_button_selection() {
        let mut model = model_with_selected(Some("all-controls"));

        normalize_selected_item_id(&mut model);

        assert_eq!(model.selected_item_id.as_ref().map(|id| id.to_string()), Some("all-controls".to_string()));
    }

    #[test]
    fn normalize_keeps_enabled_child_selection() {
        let mut model = model_with_selected(Some("button"));

        normalize_selected_item_id(&mut model);

        assert_eq!(model.selected_item_id.as_ref().map(|id| id.to_string()), Some("button".to_string()));
    }

    #[test]
    fn normalize_clears_missing_selection() {
        let mut model = model_with_selected(Some("missing"));

        normalize_selected_item_id(&mut model);

        assert_eq!(model.selected_item_id, None);
    }

    #[test]
    fn effective_active_path_uses_selected_visible_row() {
        let model = model_with_selected(Some("button"));

        assert_eq!(effective_active_path_for_model(&model, None), Some(NavPath::Child { parent: 2, child: 0 }));
    }

    #[test]
    fn effective_active_path_falls_back_to_first_enabled_row() {
        let model = model_with_selected(None);

        assert_eq!(effective_active_path_for_model(&model, None), Some(NavPath::Top(0)));
    }

    #[test]
    fn effective_active_path_is_empty_when_disabled() {
        let mut model = model_with_selected(Some("button"));
        model.enabled = false;

        assert_eq!(visible_enabled_paths_for_model(&model), Vec::new());
        assert_eq!(effective_active_path_for_model(&model, None), None);
        assert_eq!(next_active_path(&model, None, NavDirection::Next), None);
    }

    #[test]
    fn visible_enabled_paths_skip_labels_and_disabled_rows() {
        let model = model_with_selected(None);

        assert_eq!(
            visible_enabled_paths_for_model(&model),
            vec![NavPath::Top(0), NavPath::Top(2), NavPath::Child { parent: 2, child: 0 }, NavPath::Bottom(0)]
        );
    }

    #[test]
    fn arrow_movement_starts_from_effective_selected_row() {
        let model = model_with_selected(Some("button"));

        assert_eq!(next_active_path(&model, None, NavDirection::Next), Some(NavPath::Bottom(0)));
    }

    fn model_with_selected(selected_item_id: Option<&str>) -> NavViewModel {
        NavViewModel {
            id: "gallery-nav".into(),
            items: model_items(),
            bottom_items: vec![NavButton::new("settings").label("Settings")],
            selected_item_id: selected_item_id.map(|id| id.to_string().into()),
            enabled: true,
            template: default_nav_view_template(),
            item_template: default_nav_item_template(),
        }
    }

    fn model_items() -> Vec<NavItem> {
        vec![
            NavItem::button("all-controls").label("All Controls").into(),
            NavItem::label("Controls"),
            NavItem::node("command")
                .label("Command")
                .expanded(true)
                .children([
                    NavNodeItem::new("button").label("Button"),
                    NavNodeItem::new("icon-button").label("Icon Button").enabled(false),
                ])
                .into(),
        ]
    }
}
