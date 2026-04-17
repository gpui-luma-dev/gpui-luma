use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, KeyDownEvent, MouseDownEvent,
    MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::{
    DropdownMenuBuilder, DropdownMenuItem, DropdownMenuRenderModel, DropdownMenuTemplateHandlers,
    MenuPath,
};
use crate::controls::dropdown_menu::model::DropdownMenuModel;
use crate::controls::focus::blur_on_escape;
use crate::controls::interaction::ControlInteraction;
use crate::controls::menu_navigation::{MenuDirection, MenuKey, MenuNavigator};
use crate::controls::state::ControlFocusState;

#[derive(Clone, Debug)]
pub enum DropdownMenuEvent {
    Select {
        item_id: SharedString,
        label: SharedString,
    },
}

pub struct DropdownMenu {
    model: DropdownMenuModel,
    open: bool,
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    interaction: ControlInteraction,
}

impl EventEmitter<DropdownMenuEvent> for DropdownMenu {}

impl DropdownMenu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> DropdownMenuBuilder {
        DropdownMenuBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: DropdownMenuBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            open: false,
            open_submenu: None,
            active_path: None,
            interaction: ControlInteraction::new(enabled, cx),
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = DropdownMenuItem>,
        cx: &mut Context<Self>,
    ) {
        self.model.items = items.into_iter().collect();
        self.close_menu();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.close_menu();
        }
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> DropdownMenuRenderModel<'a> {
        DropdownMenuRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            items: &self.model.items,
            open: self.open,
            open_submenu: self.open_submenu,
            active_path: self.active_path,
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(
                self.model.enabled,
                self.interaction.focus_handle(),
                window,
            ),
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> DropdownMenuTemplateHandlers {
        let item_paths = self.item_click_paths();

        DropdownMenuTemplateHandlers {
            trigger_click: Box::new(cx.listener(Self::handle_trigger_click)),
            trigger_hover: Box::new(cx.listener(Self::handle_hover)),
            trigger_mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            trigger_mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            trigger_mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            root_mouse_down_out: Box::new(cx.listener(Self::handle_mouse_down_out)),
            item_hovers: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_item_hover(index, *hovered, cx);
                    })) as _
                })
                .collect(),
            item_clicks: item_paths
                .into_iter()
                .map(|path| {
                    Box::new(cx.listener(move |this, event, window, cx| {
                        this.handle_item_click(&path, event, window, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn item_click_paths(&self) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();

        for (index, item) in self.model.items.iter().enumerate() {
            if item.enabled && item.submenu_items.is_empty() {
                paths.push(vec![index]);
            } else if self.open_submenu == Some(index) {
                paths.extend(
                    item.submenu_items
                        .iter()
                        .enumerate()
                        .filter(|(_, submenu_item)| {
                            submenu_item.enabled && submenu_item.submenu_items.is_empty()
                        })
                        .map(|(submenu_index, _)| vec![index, submenu_index]),
                );
            }
        }

        paths
    }

    fn close_menu(&mut self) {
        self.open = false;
        self.open_submenu = None;
        self.active_path = None;
    }

    fn open_menu_with(&mut self, active_path: Option<MenuPath>) -> bool {
        let open_submenu = match active_path {
            Some(MenuPath::Submenu { parent, .. }) => Some(parent),
            _ => None,
        };

        let changed =
            !self.open || self.open_submenu != open_submenu || self.active_path != active_path;

        self.open = true;
        self.open_submenu = open_submenu;
        self.active_path = active_path;

        changed
    }

    fn select_item_at_path(&mut self, path: &[usize], cx: &mut Context<Self>) -> bool {
        let Some(item) = self.item_at_path(path) else {
            return false;
        };

        if !self.model.enabled || !item.enabled || !item.submenu_items.is_empty() {
            return false;
        }

        let item_id = item.id.clone();
        let label = item.label.clone();
        self.close_menu();
        cx.emit(DropdownMenuEvent::Select { item_id, label });
        true
    }

    fn handle_trigger_click(
        &mut self,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.enabled {
            if self.open {
                self.close_menu();
            } else {
                self.open_menu_with(None);
            }
            cx.notify();
        }
    }

    fn handle_item_click(
        &mut self,
        path: &[usize],
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.select_item_at_path(path, cx) {
            cx.notify();
        }
    }

    fn item_at_path(&self, path: &[usize]) -> Option<&DropdownMenuItem> {
        match path {
            [index] => self.model.items.get(*index),
            [index, submenu_index] => self
                .model
                .items
                .get(*index)?
                .submenu_items
                .get(*submenu_index),
            _ => None,
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !hovered || !self.open {
            return;
        }

        let next_submenu = self
            .model
            .items
            .get(index)
            .is_some_and(|item| item.enabled && !item.submenu_items.is_empty())
            .then_some(index);

        let next_active_path = Some(MenuPath::Root(index));
        if self.open_submenu != next_submenu || self.active_path != next_active_path {
            self.open_submenu = next_submenu;
            self.active_path = next_active_path;
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .interaction
            .handle_mouse_down(self.model.enabled, window, cx)
        {
            cx.notify();
        }
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_mouse_down_out(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open {
            self.close_menu();
            cx.notify();
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        if !self.open {
            if blur_on_escape(event, window, cx) {
                return;
            }

            let Some(key) = MenuKey::opening_key(event) else {
                return;
            };
            let navigator = MenuNavigator::new(&self.model.items);
            let active_path = match key {
                MenuKey::OpenLast => navigator.last_root().map(MenuPath::Root),
                _ => navigator.first_root().map(MenuPath::Root),
            };

            cx.stop_propagation();
            if self.open_menu_with(active_path) {
                cx.notify();
            }
            return;
        }

        let Some(key) = MenuKey::from_key_down(event) else {
            return;
        };

        cx.stop_propagation();

        let navigator = MenuNavigator::new(&self.model.items);
        let mut notify = false;

        match key {
            MenuKey::Close => {
                self.close_menu();
                notify = true;
            }
            MenuKey::Previous | MenuKey::Next => {
                let direction = if key == MenuKey::Previous {
                    MenuDirection::Previous
                } else {
                    MenuDirection::Next
                };

                let next_path = match self.active_path {
                    Some(MenuPath::Submenu { parent, child }) => navigator
                        .step_submenu(parent, Some(child), direction)
                        .map(|child| MenuPath::Submenu { parent, child }),
                    _ => navigator
                        .step_root(navigator.active_root(self.active_path), direction)
                        .map(MenuPath::Root),
                };

                if let Some(next_path) = next_path
                    && self.active_path != Some(next_path)
                {
                    self.active_path = Some(next_path);
                    if matches!(next_path, MenuPath::Root(_)) {
                        self.open_submenu = None;
                    }
                    notify = true;
                }
            }
            MenuKey::First | MenuKey::Last => {
                let next_path = match self.active_path {
                    Some(MenuPath::Submenu { parent, .. }) => {
                        let child = if key == MenuKey::First {
                            navigator.first_submenu(parent)
                        } else {
                            navigator.last_submenu(parent)
                        };

                        child.map(|child| MenuPath::Submenu { parent, child })
                    }
                    _ => {
                        let root = if key == MenuKey::First {
                            navigator.first_root()
                        } else {
                            navigator.last_root()
                        };

                        root.map(MenuPath::Root)
                    }
                };

                if let Some(next_path) = next_path
                    && self.active_path != Some(next_path)
                {
                    self.active_path = Some(next_path);
                    if matches!(next_path, MenuPath::Root(_)) {
                        self.open_submenu = None;
                    }
                    notify = true;
                }
            }
            MenuKey::OpenSubmenu => {
                if let Some(parent) = navigator.active_root(self.active_path)
                    && let Some(child) = navigator.first_submenu(parent)
                {
                    let next_path = Some(MenuPath::Submenu { parent, child });
                    if self.open_submenu != Some(parent) || self.active_path != next_path {
                        self.open_submenu = Some(parent);
                        self.active_path = next_path;
                        notify = true;
                    }
                }
            }
            MenuKey::CloseSubmenu => {
                if let Some(MenuPath::Submenu { parent, .. }) = self.active_path {
                    self.open_submenu = None;
                    self.active_path = Some(MenuPath::Root(parent));
                    notify = true;
                }
            }
            MenuKey::Select => {
                let Some(active_path) = self.active_path else {
                    return;
                };

                if let Some(item) = navigator.active_item(Some(active_path)) {
                    if !item.submenu_items().is_empty() {
                        if let MenuPath::Root(parent) = active_path
                            && let Some(child) = navigator.first_submenu(parent)
                        {
                            self.open_submenu = Some(parent);
                            self.active_path = Some(MenuPath::Submenu { parent, child });
                            notify = true;
                        }
                    } else {
                        let path = match active_path {
                            MenuPath::Root(index) => vec![index],
                            MenuPath::Submenu { parent, child } => vec![parent, child],
                        };
                        notify = self.select_item_at_path(&path, cx);
                    }
                }
            }
            MenuKey::OpenFirst | MenuKey::OpenLast => {}
        }

        if notify {
            cx.notify();
        }
    }
}

impl Focusable for DropdownMenu {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for DropdownMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .on_key_down(cx.listener(Self::handle_key_down)),
            )
            .into_any_element()
    }
}
