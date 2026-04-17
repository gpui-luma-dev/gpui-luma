use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Pixels, Point, Render, SharedString, Window, div, prelude::*,
};

use super::{ContextMenuBuilder, ContextMenuRenderModel, ContextMenuTemplateHandlers};
use crate::controls::context_menu::model::ContextMenuModel;
use crate::controls::dropdown_menu::DropdownMenuItem;
use crate::controls::interaction::ControlInteraction;

#[derive(Clone, Debug)]
pub enum ContextMenuEvent {
    Select {
        item_id: SharedString,
        label: SharedString,
    },
}

pub struct ContextMenu {
    model: ContextMenuModel,
    menu_position: Option<Point<Pixels>>,
    open_submenu: Option<usize>,
    interaction: ControlInteraction,
}

impl EventEmitter<ContextMenuEvent> for ContextMenu {}

impl ContextMenu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ContextMenuBuilder {
        ContextMenuBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ContextMenuBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            menu_position: None,
            open_submenu: None,
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

    fn render_model<'a>(&'a self, window: &Window) -> ContextMenuRenderModel<'a> {
        ContextMenuRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            items: &self.model.items,
            menu_position: self.menu_position,
            open_submenu: self.open_submenu,
            enabled: self.model.enabled,
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ContextMenuTemplateHandlers {
        let item_paths = self.item_click_paths();

        ContextMenuTemplateHandlers {
            target_aux_click: Box::new(cx.listener(Self::handle_target_aux_click)),
            target_hover: Box::new(cx.listener(Self::handle_hover)),
            target_mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            target_mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            target_mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
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

    fn handle_target_aux_click(
        &mut self,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.enabled
            && event.is_right_click()
            && let Some(position) = event.mouse_position()
        {
            self.menu_position = Some(position);
            self.open_submenu = None;
            cx.stop_propagation();
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
        let Some(item) = self.item_at_path(path) else {
            return;
        };

        if self.model.enabled && item.enabled && item.submenu_items.is_empty() {
            let item_id = item.id.clone();
            let label = item.label.clone();
            self.close_menu();
            cx.emit(ContextMenuEvent::Select { item_id, label });
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
        if !hovered || self.menu_position.is_none() {
            return;
        }

        let next_submenu = self
            .model
            .items
            .get(index)
            .is_some_and(|item| item.enabled && !item.submenu_items.is_empty())
            .then_some(index);

        if self.open_submenu != next_submenu {
            self.open_submenu = next_submenu;
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
        if self.menu_position.is_some() {
            self.close_menu();
            cx.notify();
        }
    }

    fn close_menu(&mut self) {
        self.menu_position = None;
        self.open_submenu = None;
    }
}

impl Focusable for ContextMenu {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for ContextMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle()),
            )
            .into_any_element()
    }
}
