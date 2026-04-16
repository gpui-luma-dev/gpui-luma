use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*,
};

use super::{
    DropdownMenuBuilder, DropdownMenuItem, DropdownMenuRenderModel, DropdownMenuTemplateHandlers,
};
use crate::controls::dropdown_menu::model::DropdownMenuModel;
use crate::controls::interaction::ControlInteraction;

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
        self.open = false;
        self.open_submenu = None;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.open = false;
            self.open_submenu = None;
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
            enabled: self.model.enabled,
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

    fn handle_trigger_click(
        &mut self,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.enabled {
            self.open = !self.open;
            if !self.open {
                self.open_submenu = None;
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
        let Some(item) = self.item_at_path(path) else {
            return;
        };

        if self.model.enabled && item.enabled && item.submenu_items.is_empty() {
            let item_id = item.id.clone();
            let label = item.label.clone();
            self.open = false;
            self.open_submenu = None;
            cx.emit(DropdownMenuEvent::Select { item_id, label });
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
        if self.open {
            self.open = false;
            self.open_submenu = None;
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
                    .track_focus(self.interaction.focus_handle()),
            )
            .into_any_element()
    }
}
