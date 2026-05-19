use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, Focusable, IntoElement, Render, SharedString, Subscription, Window,
    div, prelude::*,
};

use super::focus::{CompositeFocus, CompositeFocusDirection};
use crate::controls::command::button::{Button, ButtonEvent, ButtonTemplate, HasPresenter};
use crate::controls::radio_button;
use crate::focus::{NextFocus, PreviousFocus};
use crate::keyhandling::{ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionMode {
    SingleRequired,
    SingleAllowNone,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::SingleRequired
    }
}

#[derive(Clone, Debug)]
pub enum RadioGroupEvent<T> {
    Change {
        selected_value: Option<T>,
        #[allow(dead_code)]
        selected_id: Option<SharedString>,
    },
}

pub type RadioGroupTemplate<T> =
    Arc<dyn Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync>;

#[derive(Clone)]
pub struct RadioItemState<T> {
    pub is_selected: bool,
    pub value: T,
}

pub struct RadioGroupBuilder<T> {
    id: SharedString,
    items: Vec<T>,
    selected: Option<T>,
    mode: SelectionMode,
    item_id: Option<Arc<dyn Fn(&T) -> SharedString + Send + Sync>>,
    item_label: Option<Arc<dyn Fn(&T) -> SharedString + Send + Sync>>,
    item_button_template: Option<Arc<dyn ButtonTemplate<RadioItemState<T>>>>,
    template: Option<RadioGroupTemplate<T>>,
}

pub struct RadioGroup<T> {
    id: SharedString,
    buttons: Vec<Entity<Button<RadioItemState<T>>>>,
    items: Vec<RadioGroupItem<T>>,
    selected_index: Option<usize>,
    focus: CompositeFocus,
    mode: SelectionMode,
    template: RadioGroupTemplate<T>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct RadioGroupItem<T> {
    value: T,
    id: SharedString,
    label: SharedString,
}

pub fn new<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T> {
    RadioGroupBuilder {
        id: id.into(),
        items: Vec::new(),
        selected: None,
        mode: SelectionMode::default(),
        item_id: None,
        item_label: None,
        item_button_template: None,
        template: None,
    }
}

impl<T> RadioGroupBuilder<T>
where
    T: Clone + Eq + 'static,
{
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn selected(mut self, selected: T) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn item_id(mut self, item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static) -> Self {
        self.item_id = Some(Arc::new(item_id));
        self
    }

    pub fn item_label(mut self, item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static) -> Self {
        self.item_label = Some(Arc::new(item_label));
        self
    }

    pub fn item_template(mut self, item_button_template: Arc<dyn ButtonTemplate<RadioItemState<T>>>) -> Self {
        self.item_button_template = Some(item_button_template);
        self
    }

    pub fn mode(mut self, mode: SelectionMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn template(mut self, template: RadioGroupTemplate<T>) -> Self {
        self.template = Some(template);
        self
    }

    pub fn with_template<F>(self, template: F) -> Self
    where
        F: Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync + 'static,
    {
        self.template(Arc::new(template))
    }

    pub fn build(self, cx: &mut Context<RadioGroup<T>>) -> RadioGroup<T> {
        let item_id = self.item_id.expect("radio group requires item_id");
        let item_label = self.item_label.expect("radio group requires item_label");
        let item_button_template = self.item_button_template.expect("radio group requires item template");
        let items: Vec<RadioGroupItem<T>> = self
            .items
            .into_iter()
            .map(|value| RadioGroupItem { id: item_id(&value), label: item_label(&value), value })
            .collect();

        let mut selected_index =
            self.selected.as_ref().and_then(|selected| items.iter().position(|item| &item.value == selected));

        if selected_index.is_none() && self.mode == SelectionMode::SingleRequired {
            selected_index = (!items.is_empty()).then_some(0);
        }

        let mut buttons = Vec::with_capacity(items.len());
        let mut subscriptions = Vec::with_capacity(items.len());

        for (index, item) in items.iter().enumerate() {
            let button_id = format!("{}-{}", self.id, item.id);
            let is_selected = Some(index) == selected_index;
            let button = radio_button::new(button_id)
                .data(RadioItemState { is_selected, value: item.value.clone() })
                .label(item.label.clone())
                .tab_stop(false)
                .template(item_button_template.clone())
                .spawn(cx);

            subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.handle_button_click(index, cx);
                }
            }));

            buttons.push(button);
        }

        let template = self.template.expect("radio group requires a template");
        let active_index = selected_index.or_else(|| first_item_index(&items));

        RadioGroup {
            id: self.id,
            buttons,
            items,
            selected_index,
            focus: CompositeFocus::new(cx.focus_handle().tab_stop(true), active_index),
            mode: self.mode,
            template,
            _subscriptions: subscriptions,
        }
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<RadioGroup<T>> {
        cx.new(|cx| self.build(cx))
    }
}

impl<T> RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    pub fn buttons(&self) -> &[Entity<Button<RadioItemState<T>>>] {
        &self.buttons
    }

    pub fn selected_id(&self) -> Option<SharedString> {
        self.selected_index.and_then(|index| self.items.get(index).map(|item| item.id.clone()))
    }

    pub fn selected_value(&self) -> Option<T> {
        self.selected_index.and_then(|index| self.items.get(index).map(|item| item.value.clone()))
    }

    fn handle_button_click(&mut self, index: usize, cx: &mut Context<Self>) {
        self.focus.set_active_index(Some(index));

        let was_selected = self.selected_index == Some(index);
        let next_selected_index = match self.mode {
            SelectionMode::SingleRequired => Some(index),
            SelectionMode::SingleAllowNone => {
                if was_selected {
                    None
                } else {
                    Some(index)
                }
            }
        };

        if next_selected_index == self.selected_index {
            return;
        }

        self.selected_index = next_selected_index;
        self.sync_button_states(cx);
        cx.emit(RadioGroupEvent::Change { selected_value: self.selected_value(), selected_id: self.selected_id() });
        cx.notify();
    }

    fn sync_button_states(&mut self, cx: &mut Context<Self>) {
        let selected_index = self.selected_index;
        for (index, (button, item)) in self.buttons.iter().zip(self.items.iter()).enumerate() {
            let selected = Some(index) == selected_index;
            let value = item.value.clone();
            button.update(cx, move |button, cx| {
                button.set_data(RadioItemState { is_selected: selected, value }, cx);
            });
        }
    }

    fn focus_active_or_first_button(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus.focus_active_or_first(self.items.len(), |index| {
            focus_button_at(&self.buttons, index, window, cx);
        });
    }

    fn move_active_to_boundary(&mut self, first: bool, window: &mut Window, cx: &mut Context<Self>) {
        let buttons = self.buttons.clone();
        if self
            .focus
            .move_to_boundary(self.items.len(), first, |index| focus_button_at(&buttons, index, window, cx))
        {
            cx.notify();
        }
    }

    fn move_active(&mut self, direction: CompositeFocusDirection, window: &mut Window, cx: &mut Context<Self>) {
        let buttons = self.buttons.clone();
        if self
            .focus
            .move_active(self.items.len(), direction, |index| focus_button_at(&buttons, index, window, cx))
        {
            cx.notify();
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(CompositeFocusDirection::Previous, window, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(CompositeFocusDirection::Next, window, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, window, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, window, cx);
    }

    fn handle_next_focus(&mut self, _: &NextFocus, window: &mut Window, cx: &mut Context<Self>) {
        // Child radio buttons are programmatically focusable for visible focus styling,
        // but they are not tab stops. Restore focus to the group before Tab traversal
        // so next/previous focus is computed from the container's position.
        self.focus.focus_next(window, cx);
    }

    fn handle_previous_focus(&mut self, _: &PreviousFocus, window: &mut Window, cx: &mut Context<Self>) {
        // Child radio buttons are programmatically focusable for visible focus styling,
        // but they are not tab stops. Restore focus to the group before Tab traversal
        // so next/previous focus is computed from the container's position.
        self.focus.focus_previous(window, cx);
    }
}

fn first_item_index<T>(items: &[RadioGroupItem<T>]) -> Option<usize> {
    items.first().map(|_| 0)
}

fn focus_button_at<D, T>(buttons: &[Entity<Button<D>>], index: usize, window: &mut Window, cx: &mut Context<T>)
where
    D: Clone + 'static,
{
    if let Some(button) = buttons.get(index) {
        button.update(cx, |button, cx| {
            button.focus_handle(cx).focus(window, cx);
        });
    }
}

impl<T> Focusable for RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus.focus_handle().clone()
    }
}

impl<T> EventEmitter<RadioGroupEvent<T>> for RadioGroup<T> where T: Clone + Eq + 'static {}

impl<T> Render for RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus.focus_handle().is_focused(window) {
            // The group owns Tab navigation, but the child radio button owns the visible
            // focus indicator. When the group is focused, hand visual focus to the
            // active item, or the first item if no active item has been established.
            self.focus_active_or_first_button(window, cx);
        }

        div()
            .id(self.id.clone())
            .track_focus(self.focus.focus_handle())
            .key_context(ControlKeyProfile::TabList.context())
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_next_focus))
            .on_action(cx.listener(Self::handle_previous_focus))
            .child((self.template)(self, window, cx))
            .into_any_element()
    }
}
