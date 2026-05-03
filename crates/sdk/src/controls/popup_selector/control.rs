use gpui::{
    App, Bounds, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Pixels,
    Render, SharedString, Window, div, prelude::*,
};

use super::{PopupSelectorBuilder, PopupSelectorPlacement, PopupSelectorRenderModel, PopupSelectorTemplateHandlers};
use crate::controls::interaction::ControlInteraction;
use crate::controls::popup_selector::model::{
    PopupSelectorItemTemplate, PopupSelectorModel, SelectorItem, SelectorItemLike, normalize_popup_selector_items,
};
use crate::controls::state::{ControlFocusState, MenuPath};
use crate::focus::EscapeFocus;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum PopupSelectorEvent {
    Change { item_id: SharedString, label: SharedString },
}

pub struct PopupSelector<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    model: PopupSelectorModel<T>,
    open: bool,
    trigger_bounds: Option<Bounds<Pixels>>,
    selected_index: Option<usize>,
    active_index: Option<usize>,
    interaction: ControlInteraction,
}

impl<T> EventEmitter<PopupSelectorEvent> for PopupSelector<T> where T: SelectorItemLike + 'static {}

impl PopupSelector<SelectorItem> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopupSelectorBuilder<SelectorItem> {
        PopupSelectorBuilder::new(id)
    }
}

impl<T> PopupSelector<T>
where
    T: SelectorItemLike + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new_with_items(id: impl Into<SharedString>) -> PopupSelectorBuilder<T> {
        PopupSelectorBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: PopupSelectorBuilder<T>, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let selected_index = builder.model.selected_id.as_ref().and_then(|selected_id| {
            builder.model.items.iter().position(|item| item.id() == selected_id && item.enabled())
        });

        Self {
            model: builder.model,
            open: false,
            trigger_bounds: None,
            selected_index,
            active_index: None,
            interaction: ControlInteraction::new(enabled, cx),
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.model.label = label.into();
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        let previous_selection = self.selected_id().cloned();
        self.model.items = normalize_popup_selector_items(items);
        self.selected_index = previous_selection.as_ref().and_then(|id| self.index_by_id(id));
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

    pub fn set_placement(&mut self, placement: PopupSelectorPlacement, cx: &mut Context<Self>) {
        self.model.placement = placement;
        cx.notify();
    }

    pub fn set_item_template(&mut self, template: Option<PopupSelectorItemTemplate<T>>, cx: &mut Context<Self>) {
        self.model.item_template = template;
        cx.notify();
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.selected_index.and_then(|index| self.model.items.get(index)).map(SelectorItemLike::id)
    }

    pub fn set_selected_id(&mut self, item_id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        let item_id = item_id.into();
        let changed = self.select_by_id(&item_id, false, cx);
        if changed {
            cx.notify();
        }
        changed
    }

    fn render_model<'a>(&'a self, window: &Window) -> PopupSelectorRenderModel<'a, T> {
        PopupSelectorRenderModel {
            id: &self.model.id,
            label: self.trigger_label(),
            selected_icon: self
                .selected_index
                .and_then(|index| self.model.items.get(index))
                .and_then(SelectorItemLike::icon),
            selected_index: self.selected_index,
            items: &self.model.items,
            open: self.open,
            trigger_bounds: self.trigger_bounds,
            placement: self.model.placement,
            active_path: self.active_index.map(MenuPath::Root),
            enabled: self.model.enabled,
            item_template: self.model.item_template.as_ref(),
            focus: ControlFocusState::from_focus_handle(self.model.enabled, self.interaction.focus_handle(), window),
            state: self.interaction.render_state(self.model.enabled, window),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> PopupSelectorTemplateHandlers {
        let selectable_indices = self.selectable_indices();

        PopupSelectorTemplateHandlers {
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
            item_clicks: selectable_indices
                .into_iter()
                .map(|index| {
                    Box::new(cx.listener(move |this, event, _window, cx| {
                        this.handle_item_click(index, event, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn trigger_label(&self) -> &SharedString {
        self.selected_index
            .and_then(|index| self.model.items.get(index))
            .map_or(&self.model.label, SelectorItemLike::label_text)
    }

    fn index_by_id(&self, item_id: &SharedString) -> Option<usize> {
        self.model.items.iter().position(|item| item.id() == item_id && self.is_selectable_item(item))
    }

    fn is_selectable_item(&self, item: &T) -> bool {
        item.enabled()
    }

    fn selectable_indices(&self) -> Vec<usize> {
        self.model
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| self.is_selectable_item(item).then_some(index))
            .collect()
    }

    fn first_selectable_index(&self) -> Option<usize> {
        self.model.items.iter().position(|item| self.is_selectable_item(item))
    }

    fn last_selectable_index(&self) -> Option<usize> {
        self.model.items.iter().rposition(|item| self.is_selectable_item(item))
    }

    fn step_selectable_index(&self, current: Option<usize>, forward: bool) -> Option<usize> {
        let indices = self.selectable_indices();
        if indices.is_empty() {
            return None;
        }

        match current.and_then(|current| indices.iter().position(|index| *index == current)) {
            Some(position) => {
                let next = if forward {
                    (position + 1) % indices.len()
                } else if position == 0 {
                    indices.len() - 1
                } else {
                    position - 1
                };
                Some(indices[next])
            }
            None => {
                if forward {
                    indices.first().copied()
                } else {
                    indices.last().copied()
                }
            }
        }
    }

    fn close_menu(&mut self) {
        self.open = false;
        self.active_index = None;
    }

    fn open_menu_with_active(&mut self, active_index: Option<usize>) -> bool {
        let next_active =
            active_index.filter(|index| self.model.items.get(*index).is_some_and(|item| self.is_selectable_item(item)));
        let changed = !self.open || self.active_index != next_active;
        self.open = true;
        self.active_index = next_active;
        changed
    }

    fn open_with_default_active(&mut self) -> bool {
        let active = self
            .selected_index
            .filter(|index| self.model.items.get(*index).is_some_and(|item| self.is_selectable_item(item)))
            .or_else(|| self.first_selectable_index());
        self.open_menu_with_active(active)
    }

    fn select_index(&mut self, index: usize, emit: bool, cx: &mut Context<Self>) -> bool {
        let Some(item) = self.model.items.get(index) else {
            return false;
        };
        if !self.is_selectable_item(item) {
            return false;
        }

        let item_id = item.id().clone();
        let label = item.label_text().clone();
        let changed = self.selected_index != Some(index);
        self.selected_index = Some(index);
        self.close_menu();

        if emit {
            cx.emit(PopupSelectorEvent::Change { item_id, label });
        }

        changed
    }

    fn select_by_id(&mut self, item_id: &SharedString, emit: bool, cx: &mut Context<Self>) -> bool {
        let Some(index) = self.index_by_id(item_id) else {
            return false;
        };

        self.select_index(index, emit, cx)
    }

    fn activate_active_item(&mut self, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if !self.open {
            if self.open_with_default_active() {
                cx.notify();
            }
            return;
        }

        if let Some(active_index) = self.active_index
            && self.select_index(active_index, true, cx)
        {
            cx.notify();
        }
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
                self.open_with_default_active();
            }
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if self.select_index(index, true, cx) {
            cx.notify();
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !hovered || !self.open || !self.model.items.get(index).is_some_and(|item| self.is_selectable_item(item)) {
            return;
        }

        if self.active_index != Some(index) {
            self.active_index = Some(index);
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

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.open {
            let next = self.step_selectable_index(self.active_index, false);
            if self.active_index != next {
                self.active_index = next;
                cx.notify();
            }
        } else if self.open_menu_with_active(self.last_selectable_index()) {
            cx.notify();
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.open {
            let next = self.step_selectable_index(self.active_index, true);
            if self.active_index != next {
                self.active_index = next;
                cx.notify();
            }
        } else if self.open_menu_with_active(self.first_selectable_index()) {
            cx.notify();
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            let next = self.first_selectable_index();
            if self.active_index != next {
                self.active_index = next;
                cx.notify();
            }
        } else if self.model.enabled {
            cx.propagate();
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            let next = self.last_selectable_index();
            if self.active_index != next {
                self.active_index = next;
                cx.notify();
            }
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

impl<T> Focusable for PopupSelector<T>
where
    T: SelectorItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl<T> Render for PopupSelector<T>
where
    T: SelectorItemLike + 'static,
{
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
                    .on_action(cx.listener(Self::handle_activate_control)),
            )
            .into_any_element()
    }
}
