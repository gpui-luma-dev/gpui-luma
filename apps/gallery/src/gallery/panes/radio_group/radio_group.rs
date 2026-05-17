use std::sync::Arc;

use gpui::{AnyElement, Context, EventEmitter, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonTemplate, HasPresenter};
use gpui_luma::controls::radio_button::{self, RadioButton};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(in crate::gallery) enum SelectionMode {
    SingleRequired,
    SingleAllowNone,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::SingleRequired
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum RadioGroupLayout {
    Vertical,
    Horizontal,
}

impl Default for RadioGroupLayout {
    fn default() -> Self {
        Self::Vertical
    }
}

#[derive(Clone, Debug)]
pub(in crate::gallery) enum RadioGroupEvent {
    Change { selected_id: Option<SharedString> },
}

pub type RadioGroupTemplate<T> =
    Arc<dyn Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync>;

#[allow(dead_code)]
pub(in crate::gallery) struct RadioGroupBuilder<T> {
    id: SharedString,
    items: Vec<T>,
    selected: Option<T>,
    mode: SelectionMode,
    layout: RadioGroupLayout,
    item_id: Arc<dyn Fn(&T) -> SharedString + Send + Sync>,
    item_label: Arc<dyn Fn(&T) -> SharedString + Send + Sync>,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
    template: Option<RadioGroupTemplate<T>>,
}

pub(in crate::gallery) struct RadioGroup<T> {
    buttons: Vec<RadioButton>,
    items: Vec<RadioGroupItem<T>>,
    selected_index: Option<usize>,
    mode: SelectionMode,
    template: RadioGroupTemplate<T>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
#[allow(dead_code)]
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
        layout: RadioGroupLayout::default(),
        item_id: Arc::new(item_id),
        item_label: Arc::new(item_label),
        item_button_template,
        template: None,
    }
}

pub(in crate::gallery) fn vertical_group<T>(
    id: impl Into<SharedString>,
    item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupBuilder<T> {
    new(id, item_id, item_label, item_button_template)
}

pub(in crate::gallery) fn horizontal_group<T>(
    id: impl Into<SharedString>,
    item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupBuilder<T>
where
    T: Clone + Eq + 'static,
{
    new(id, item_id, item_label, item_button_template).layout(RadioGroupLayout::Horizontal)
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

    pub(in crate::gallery) fn layout(mut self, layout: RadioGroupLayout) -> Self {
        self.layout = layout;
        self
    }

    pub(in crate::gallery) fn with_template<F>(mut self, template: F) -> Self
    where
        F: Fn(&RadioGroup<T>, &mut Window, &mut Context<RadioGroup<T>>) -> AnyElement + Send + Sync + 'static,
    {
        self.template = Some(Arc::new(template));
        self
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
                .template(self.item_button_template.clone())
                .spawn(cx);

            subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    this.handle_button_click(index, cx);
                }
            }));

            buttons.push(button);
        }

        let template = self.template.unwrap_or_else(|| match self.layout {
            RadioGroupLayout::Vertical => vertical_group_template(),
            RadioGroupLayout::Horizontal => horizontal_group_template(),
        });

        RadioGroup { buttons, items, selected_index, mode: self.mode, template, _subscriptions: subscriptions }
    }
}

impl<T> RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    pub(in crate::gallery) fn selected_id(&self) -> Option<SharedString> {
        self.selected_index.and_then(|index| self.items.get(index).map(|item| item.id.clone()))
    }

    fn handle_button_click(&mut self, index: usize, cx: &mut Context<Self>) {
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
}

impl<T> EventEmitter<RadioGroupEvent> for RadioGroup<T> where T: Clone + Eq + 'static {}

impl<T> Render for RadioGroup<T>
where
    T: Clone + Eq + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        (self.template)(self, window, cx)
    }
}

pub(in crate::gallery) fn vertical_group_template<T: 'static>() -> RadioGroupTemplate<T> {
    Arc::new(|group, _window, _cx| {
        div().flex().flex_col().gap_2().children(group.buttons.iter().cloned()).into_any_element()
    })
}

pub(in crate::gallery) fn horizontal_group_template<T: 'static>() -> RadioGroupTemplate<T> {
    Arc::new(|group, _window, _cx| {
        div().flex().items_center().gap_3().children(group.buttons.iter().cloned()).into_any_element()
    })
}
