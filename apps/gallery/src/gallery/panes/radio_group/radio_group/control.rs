use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, EventEmitter, Focusable, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*,
};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonTemplate, HasPresenter};
use gpui_luma::controls::radio_button::{self, RadioButton};
use gpui_luma::focus::{NextFocus, PreviousFocus};
use gpui_luma::keyhandling::{ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem};

use super::focus::{CompositeFocus, CompositeFocusDirection};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum SelectionMode {
    SingleRequired,
    SingleAllowNone,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::SingleRequired
    }
}

#[derive(Clone, Debug)]
pub(in crate::gallery) enum RadioGroupEvent {
    Change { selected_id: Option<SharedString> },
}

pub type RadioGroupTemplate<T> =
    Arc<dyn Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync>;

pub(in crate::gallery) struct RadioGroupBuilder<T> {
    id: SharedString,
    items: Vec<T>,
    selected: Option<T>,
    mode: SelectionMode,
    item_id: Arc<dyn Fn(&T) -> SharedString + Send + Sync>,
    item_label: Arc<dyn Fn(&T) -> SharedString + Send + Sync>,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
    template: Option<RadioGroupTemplate<T>>,
}

pub(in crate::gallery) struct RadioGroup<T> {
    id: SharedString,
    buttons: Vec<RadioButton>,
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

pub(in crate::gallery) fn new<T>(
    id: impl Into<SharedString>,
    item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupBuilder<T> {
    RadioGroupBuilder {
        id: id.into(),
        items: Vec::new(),
        selected: None,
        mode: SelectionMode::default(),
        item_id: Arc::new(item_id),
        item_label: Arc::new(item_label),
        item_button_template,
        template: None,
    }
}

impl<T> RadioGroupBuilder<T>
where
    T: Clone + Eq + 'static,
{
    pub(in crate::gallery) fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub(in crate::gallery) fn selected(mut self, selected: T) -> Self {
        self.selected = Some(selected);
        self
    }

    pub(in crate::gallery) fn mode(mut self, mode: SelectionMode) -> Self {
        self.mode = mode;
        self
    }

    pub(in crate::gallery) fn template(mut self, template: RadioGroupTemplate<T>) -> Self {
        self.template = Some(template);
        self
    }

    pub(in crate::gallery) fn with_template<F>(self, template: F) -> Self
    where
        F: Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync + 'static,
    {
        self.template(Arc::new(template))
    }

    pub(in crate::gallery) fn build(self, cx: &mut Context<RadioGroup<T>>) -> RadioGroup<T> {
        let items: Vec<RadioGroupItem<T>> = self
            .items
            .into_iter()
            .map(|value| RadioGroupItem { id: (self.item_id)(&value), label: (self.item_label)(&value), value })
            .collect();

        let mut selected_index =
            self.selected.as_ref().and_then(|selected| items.iter().position(|item| &item.value == selected));

        if selected_index.is_none() && self.mode == SelectionMode::SingleRequired {
            selected_index = items.iter().position(|_| true);
        }

        let mut buttons = Vec::with_capacity(items.len());
        let mut subscriptions = Vec::with_capacity(items.len());

        for (index, item) in items.iter().enumerate() {
            let button_id = format!("{}-{}", self.id, item.id);
            let is_selected = Some(index) == selected_index;
            let button = radio_button::new(button_id)
                .data(is_selected)
                .label(item.label.clone())
                .tab_stop(false)
                .template(self.item_button_template.clone())
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
}

impl<T> RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    pub(in crate::gallery) fn selected_id(&self) -> Option<SharedString> {
        self.selected_index.and_then(|index| self.items.get(index).map(|item| item.id.clone()))
    }

    pub(in crate::gallery) fn buttons(&self) -> &[RadioButton] {
        &self.buttons
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
        cx.emit(RadioGroupEvent::Change { selected_id: self.selected_id() });
        cx.notify();
    }

    fn sync_button_states(&mut self, cx: &mut Context<Self>) {
        let selected_index = self.selected_index;
        for (index, button) in self.buttons.iter().enumerate() {
            let selected = Some(index) == selected_index;
            button.update(cx, |button, cx| button.set_data(selected, cx));
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

fn focus_button_at<T>(buttons: &[RadioButton], index: usize, window: &mut Window, cx: &mut Context<T>) {
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

impl<T> EventEmitter<RadioGroupEvent> for RadioGroup<T> where T: Clone + Eq + 'static {}

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
