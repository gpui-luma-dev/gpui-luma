use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, MouseButton, MouseDownEvent,
    Render, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::command::button::HasPresenter;
use gpui_luma::controls::radio_button::RadioButton;
use gpui_luma::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use super::{HeadlessControlGroup, HeadlessGroupDirection};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(dead_code)]
pub(in crate::gallery) enum RadioControlGroupOrientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug)]
pub(in crate::gallery) struct RadioControlGroupChild {
    id: SharedString,
    label: SharedString,
    enabled: bool,
}

impl RadioControlGroupChild {
    pub(in crate::gallery) fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, enabled: true }
    }

    pub(in crate::gallery) fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub(in crate::gallery) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(in crate::gallery) enum RadioControlGroupEvent {
    SelectionChanged { selected_id: SharedString },
}

pub(in crate::gallery) struct RadioControlGroup {
    id: SharedString,
    focus_handle: FocusHandle,
    orientation: RadioControlGroupOrientation,
    children: Vec<RadioControlGroupChild>,
    headless_group: HeadlessControlGroup,
    buttons: Vec<RadioButton>,
    _subscriptions: Vec<Subscription>,
}

pub(in crate::gallery) struct RadioControlGroupBuilder {
    id: SharedString,
    orientation: RadioControlGroupOrientation,
    children: Vec<RadioControlGroupChild>,
    selected_id: Option<SharedString>,
}

impl RadioControlGroupBuilder {
    pub(in crate::gallery) fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            orientation: RadioControlGroupOrientation::default(),
            children: Vec::new(),
            selected_id: None,
        }
    }

    pub(in crate::gallery) fn orientation(mut self, orientation: RadioControlGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub(in crate::gallery) fn vertical(self) -> Self {
        self.orientation(RadioControlGroupOrientation::Vertical)
    }

    #[allow(dead_code)]
    pub(in crate::gallery) fn horizontal(self) -> Self {
        self.orientation(RadioControlGroupOrientation::Horizontal)
    }

    pub(in crate::gallery) fn child(mut self, child: RadioControlGroupChild) -> Self {
        self.children.push(child);
        self
    }

    #[allow(dead_code)]
    pub(in crate::gallery) fn children(mut self, children: impl IntoIterator<Item = RadioControlGroupChild>) -> Self {
        self.children.extend(children);
        self
    }

    pub(in crate::gallery) fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(selected_id.into());
        self
    }

    pub(in crate::gallery) fn spawn(
        self,
        cx: &mut impl AppContext,
        look: Arc<ShadcnLook>,
    ) -> Entity<RadioControlGroup> {
        cx.new(|cx| RadioControlGroup::from_builder(self, look, cx))
    }
}

impl EventEmitter<RadioControlGroupEvent> for RadioControlGroup {}

impl RadioControlGroup {
    pub(in crate::gallery) fn builder(id: impl Into<SharedString>) -> RadioControlGroupBuilder {
        RadioControlGroupBuilder::new(id)
    }

    fn from_builder(builder: RadioControlGroupBuilder, look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let RadioControlGroupBuilder { id, orientation, children, selected_id } = builder;

        let headless_group = HeadlessControlGroup::new(
            children.iter().map(|child| (child.id.clone(), child.enabled)),
            selected_id,
            true,
        );

        let buttons = children
            .iter()
            .map(|child| {
                look.secondary_radio(format!("proto-radio-control-group-{}", child.id))
                    .with_data(false)
                    .tab_stop(false)
                    .enabled(child.enabled)
                    .label(child.label.clone())
                    .spawn(cx)
            })
            .collect::<Vec<_>>();

        let mut subscriptions = Vec::new();
        for (index, button) in buttons.iter().cloned().enumerate() {
            subscriptions.push(cx.subscribe(&button, move |this, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    let child_id = this.children[index].id.clone();
                    this.request_selection(child_id, cx);
                }
            }));
        }

        let control = Self {
            id,
            focus_handle: cx.focus_handle().tab_stop(true),
            orientation,
            children,
            headless_group,
            buttons,
            _subscriptions: subscriptions,
        };

        control.sync_radios(cx);
        control
    }

    pub(in crate::gallery) fn selected_id(&self) -> Option<&SharedString> {
        self.headless_group.selected_id()
    }

    pub(in crate::gallery) fn selected_label(&self) -> Option<&SharedString> {
        let selected_id = self.selected_id()?;
        self.children.iter().find(|child| &child.id == selected_id).map(|child| &child.label)
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        for button in &self.buttons {
            button.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn request_selection(&mut self, selected_id: SharedString, cx: &mut Context<Self>) {
        if self.headless_group.select(selected_id.clone()) {
            self.sync_radios(cx);
            self.emit_selection_changed(selected_id, cx);
        }
    }

    fn sync_radios(&self, cx: &mut Context<Self>) {
        let selected_id = self.headless_group.selected_id().cloned();

        for (button, child) in self.buttons.iter().zip(self.children.iter()) {
            let is_selected = selected_id.as_ref() == Some(&child.id);
            button.update(cx, |button, cx| button.set_data(is_selected, cx));
        }
    }

    fn emit_selection_changed(&self, selected_id: SharedString, cx: &mut Context<Self>) {
        cx.emit(RadioControlGroupEvent::SelectionChanged { selected_id });
        cx.notify();
    }

    fn active_index(&self) -> Option<usize> {
        let active_id = self.headless_group.active_id()?;
        self.children.iter().position(|child| &child.id == active_id)
    }

    fn focus_active(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(button) = self.active_index().and_then(|index| self.buttons.get(index)) {
            let button_focus = button.read(cx).focus_handle(cx);
            window.focus(&button_focus, cx);
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.headless_group.move_active(HeadlessGroupDirection::Previous) {
            self.sync_radios(cx);
            self.focus_active(window, cx);
            if let Some(selected_id) = self.headless_group.selected_id().cloned() {
                self.emit_selection_changed(selected_id, cx);
            }
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.headless_group.move_active(HeadlessGroupDirection::Next) {
            self.sync_radios(cx);
            self.focus_active(window, cx);
            if let Some(selected_id) = self.headless_group.selected_id().cloned() {
                self.emit_selection_changed(selected_id, cx);
            }
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.headless_group.move_active_to_boundary(true) {
            self.sync_radios(cx);
            self.focus_active(window, cx);
            if let Some(selected_id) = self.headless_group.selected_id().cloned() {
                self.emit_selection_changed(selected_id, cx);
            }
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.headless_group.move_active_to_boundary(false) {
            self.sync_radios(cx);
            self.focus_active(window, cx);
            if let Some(selected_id) = self.headless_group.selected_id().cloned() {
                self.emit_selection_changed(selected_id, cx);
            }
        }
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if self.headless_group.activate_active() {
            self.sync_radios(cx);
            if let Some(selected_id) = self.headless_group.selected_id().cloned() {
                self.emit_selection_changed(selected_id, cx);
            }
        }
    }
}

impl Focusable for RadioControlGroup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RadioControlGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_handle.is_focused(window) {
            self.focus_active(window, cx);
        }

        let container = match self.orientation {
            RadioControlGroupOrientation::Vertical => div().flex().flex_col().items_start().gap(px(12.0)),
            RadioControlGroupOrientation::Horizontal => div().flex().items_center().gap(px(16.0)),
        };

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .tab_stop(true)
            .key_context(ControlKeyProfile::TabList.context())
            .on_action(cx.listener(Self::handle_select_previous_item))
            .on_action(cx.listener(Self::handle_select_next_item))
            .on_action(cx.listener(Self::handle_select_first_item))
            .on_action(cx.listener(Self::handle_select_last_item))
            .on_action(cx.listener(Self::handle_activate_control))
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if event.button == MouseButton::Left {
                    this.focus_active(window, cx);
                }
            }))
            .child(container.children(self.buttons.iter().cloned()))
    }
}
