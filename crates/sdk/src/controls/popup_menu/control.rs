use gpui::{
    App, Bounds, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Pixels,
    Render, SharedString, Window, div, prelude::*,
};

use super::{PopupMenuBuilder, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplateHandlers, MenuPath};
use crate::controls::popup_menu::model::PopupMenuModel;
use crate::controls::floating_menu::{FloatingMenuActivateResult, FloatingMenuState, FloatingMenuStepDirection};
use crate::controls::interaction::ControlInteraction;
use crate::controls::menu_navigation::MenuNavigator;
use crate::controls::state::ControlFocusState;
use crate::focus::EscapeFocus;
use crate::keyhandling::{
    ActivateControl, CloseSubmenu, ControlKeyProfile, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum PopupMenuEvent {
    Select { item_id: SharedString, label: SharedString },
}

pub struct PopupMenu {
    model: PopupMenuModel,
    open: bool,
    trigger_bounds: Option<Bounds<Pixels>>,
    menu_state: FloatingMenuState,
    interaction: ControlInteraction,
}

impl EventEmitter<PopupMenuEvent> for PopupMenu {}

impl PopupMenu {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopupMenuBuilder {
        PopupMenuBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: PopupMenuBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            open: false,
            trigger_bounds: None,
            menu_state: FloatingMenuState::default(),
            interaction: ControlInteraction::new(enabled, cx),
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = crate::controls::menu_item::MenuItem>,
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

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::PopupMenuTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_placement(&mut self, placement: PopupMenuPlacement, cx: &mut Context<Self>) {
        self.model.placement = placement;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> PopupMenuRenderModel<'a> {
        PopupMenuRenderModel {
            id: &self.model.id,
            label: &self.model.label,
            items: &self.model.items,
            open: self.open,
            trigger_bounds: self.trigger_bounds,
            placement: self.model.placement,
            open_submenu: self.menu_state.open_submenu(),
            active_path: self.menu_state.active_path(),
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, self.interaction.focus_handle(), window),
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> PopupMenuTemplateHandlers {
        let item_paths = self.menu_state.item_click_paths(&self.model.items);

        PopupMenuTemplateHandlers {
            trigger_bounds: Box::new(cx.listener(Self::handle_trigger_bounds)),
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

    fn close_menu(&mut self) {
        self.open = false;
        self.menu_state.clear();
    }

    fn open_menu_with(&mut self, active_path: Option<MenuPath>) -> bool {
        let changed = !self.open || self.menu_state.open_with(active_path);
        self.open = true;
        changed
    }

    fn select_item_at_path(&mut self, path: &[usize], cx: &mut Context<Self>) -> bool {
        if !self.model.enabled {
            return false;
        }

        if let Some((item_id, label)) = self.menu_state.select_at_path(&self.model.items, path) {
            self.close_menu();
            cx.emit(PopupMenuEvent::Select { item_id, label });
            return true;
        }

        false
    }

    fn handle_trigger_bounds(&mut self, bounds: &Bounds<Pixels>, _window: &mut Window, _cx: &mut Context<Self>) {
        self.trigger_bounds = Some(*bounds);
    }

    fn handle_trigger_click(&mut self, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.model.enabled {
            if self.open {
                self.close_menu();
            } else {
                self.open_menu_with(None);
            }
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, path: &[usize], event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.select_item_at_path(path, cx) {
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !hovered || !self.open {
            return;
        }

        if self.menu_state.hover_root_item(&self.model.items, index) {
            cx.notify();
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_down(self.model.enabled, window, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_mouse_up() {
            cx.notify();
        }
    }

    fn handle_mouse_down_out(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.close_menu();
            cx.notify();
        }
    }

    fn open_menu_at_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let navigator = MenuNavigator::new(&self.model.items);
        let active_path = if first {
            navigator.first_root()
        } else {
            navigator.last_root()
        }
        .map(MenuPath::Root);

        if self.open_menu_with(active_path) {
            cx.notify();
        }
    }

    fn step_active_item(&mut self, direction: FloatingMenuStepDirection, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.open {
            return;
        }

        if self.menu_state.step(&self.model.items, direction) {
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.open {
            return;
        }

        if self.menu_state.move_to_boundary(&self.model.items, first) {
            cx.notify();
        }
    }

    fn open_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.open {
            return;
        }

        if self.menu_state.open_active_submenu(&self.model.items) {
            cx.notify();
        }
    }

    fn close_active_submenu(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.open {
            return;
        }

        if self.menu_state.close_active_submenu() {
            cx.notify();
        }
    }

    fn activate_active_item(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if !self.open {
            self.open_menu_at_boundary(true, cx);
            return;
        }

        if !self.open {
            self.open_menu_at_boundary(true, cx);
            return;
        }

        match self.menu_state.activate(&self.model.items) {
            FloatingMenuActivateResult::None => {}
            FloatingMenuActivateResult::OpenedSubmenu => cx.notify(),
            FloatingMenuActivateResult::Select { item_id, label } => {
                self.close_menu();
                cx.emit(PopupMenuEvent::Select { item_id, label });
                cx.notify();
            }
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.step_active_item(FloatingMenuStepDirection::Previous, cx);
        } else {
            self.open_menu_at_boundary(false, cx);
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.step_active_item(FloatingMenuStepDirection::Next, cx);
        } else {
            self.open_menu_at_boundary(true, cx);
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.move_active_to_boundary(true, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.move_active_to_boundary(false, cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_open_submenu(&mut self, _: &OpenSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.open_active_submenu(cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_close_submenu(&mut self, _: &CloseSubmenu, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.close_active_submenu(cx);
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_active_item(cx);
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.close_menu();
            cx.notify();
        } else {
            cx.propagate();
        }
    }
}

impl Focusable for PopupMenu {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::Menu.context())
                    .on_action(cx.listener(Self::handle_escape_focus))
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
