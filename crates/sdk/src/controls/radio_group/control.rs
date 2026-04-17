use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent,
    Render, SharedString, Window, div, prelude::*,
};

use super::{
    RadioGroupBuilder, RadioGroupItem, RadioGroupRenderItem, RadioGroupRenderModel,
    RadioGroupTemplateHandlers,
};
use crate::controls::radio_group::model::RadioGroupModel;
use crate::theme::InteractionState;

#[derive(Clone, Debug)]
pub enum RadioGroupEvent {
    Change {
        selected_id: SharedString,
        label: SharedString,
    },
}

pub struct RadioGroup {
    model: RadioGroupModel,
    focus_handle: gpui::FocusHandle,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
}

impl EventEmitter<RadioGroupEvent> for RadioGroup {}

impl RadioGroup {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> RadioGroupBuilder {
        RadioGroupBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: RadioGroupBuilder, cx: &mut Context<Self>) -> Self {
        normalize_selected_id(&mut builder.model);
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            hovered_item: None,
            pressed_item: None,
        }
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.model.selected_id.as_ref()
    }

    pub fn set_selected(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_id = selected_id.into();
        if self.can_select_id(&selected_id) {
            self.model.selected_id = Some(selected_id);
            cx.notify();
        }
    }

    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = RadioGroupItem>,
        cx: &mut Context<Self>,
    ) {
        self.model.items = items.into_iter().collect();
        self.hovered_item = None;
        self.pressed_item = None;
        normalize_selected_id(&mut self.model);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);

        if !enabled {
            self.hovered_item = None;
            self.pressed_item = None;
        }

        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> RadioGroupRenderModel<'a> {
        let group_focused = self.model.enabled && self.focus_handle.is_focused(window);
        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.enabled;
                let selected = self
                    .model
                    .selected_id
                    .as_ref()
                    .is_some_and(|selected_id| selected_id == &item.id);

                RadioGroupRenderItem {
                    id: &item.id,
                    label: &item.label,
                    selected,
                    enabled,
                    state: InteractionState {
                        hovered: enabled && self.hovered_item == Some(index),
                        pressed: enabled && self.pressed_item == Some(index),
                        focused: enabled && group_focused && selected,
                        disabled: !enabled,
                    },
                }
            })
            .collect();

        RadioGroupRenderModel {
            id: &self.model.id,
            items,
            selected_id: self.model.selected_id.as_ref(),
            enabled: self.model.enabled,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> RadioGroupTemplateHandlers {
        RadioGroupTemplateHandlers {
            item_hovers: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_item_hover(index, *hovered, cx);
                    })) as _
                })
                .collect(),
            item_mouse_downs: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, event, window, cx| {
                        this.handle_item_mouse_down(index, event, window, cx);
                    })) as _
                })
                .collect(),
            item_mouse_ups: (0..self.model.items.len())
                .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as _)
                .collect(),
            item_mouse_up_outs: (0..self.model.items.len())
                .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as _)
                .collect(),
            item_clicks: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, event, window, cx| {
                        this.handle_item_click(index, event, window, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn can_select_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(|item| item.enabled)
    }

    fn can_select_id(&self, selected_id: &SharedString) -> bool {
        self.model
            .items
            .iter()
            .any(|item| item.enabled && &item.id == selected_id)
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_select_item(index) {
            return;
        }

        if hovered {
            if self.hovered_item != Some(index) {
                self.hovered_item = Some(index);
                cx.notify();
            }
        } else if self.hovered_item == Some(index) {
            self.hovered_item = None;
            if self.pressed_item == Some(index) {
                self.pressed_item = None;
            }
            cx.notify();
        }
    }

    fn handle_item_mouse_down(
        &mut self,
        index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_select_item(index) {
            self.pressed_item = Some(index);
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn handle_item_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.pressed_item.is_some() {
            self.pressed_item = None;
            cx.notify();
        }
    }

    fn handle_item_click(
        &mut self,
        index: usize,
        _event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_select_item(index) {
            return;
        }

        let item = &self.model.items[index];
        if self
            .model
            .selected_id
            .as_ref()
            .is_some_and(|selected_id| selected_id == &item.id)
        {
            return;
        }

        let selected_id = item.id.clone();
        let label = item.label.clone();
        self.model.selected_id = Some(selected_id.clone());
        cx.emit(RadioGroupEvent::Change { selected_id, label });
        cx.notify();
    }
}

impl Focusable for RadioGroup {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RadioGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(&self.focus_handle),
            )
            .into_any_element()
    }
}

fn normalize_selected_id(model: &mut RadioGroupModel) {
    if model.selected_id.as_ref().is_some_and(|selected_id| {
        model
            .items
            .iter()
            .any(|item| item.enabled && &item.id == selected_id)
    }) {
        return;
    }

    model.selected_id = model
        .items
        .iter()
        .find(|item| item.enabled)
        .map(|item| item.id.clone());
}

#[cfg(test)]
mod tests {
    use super::{RadioGroupItem, RadioGroupModel, normalize_selected_id};
    use crate::controls::radio_group::default_radio_group_template;

    #[test]
    fn normalize_keeps_valid_selected_item() {
        let mut model = model_with_selected(Some("comfortable"));

        normalize_selected_id(&mut model);

        assert_eq!(
            model.selected_id.as_ref().map(|id| id.to_string()),
            Some("comfortable".to_string())
        );
    }

    #[test]
    fn normalize_falls_back_to_first_enabled_item() {
        let mut model = model_with_selected(Some("missing"));

        normalize_selected_id(&mut model);

        assert_eq!(
            model.selected_id.as_ref().map(|id| id.to_string()),
            Some("compact".to_string())
        );
    }

    fn model_with_selected(selected_id: Option<&str>) -> RadioGroupModel {
        RadioGroupModel {
            id: "density".into(),
            items: vec![
                RadioGroupItem::new("compact").label("Compact"),
                RadioGroupItem::new("comfortable").label("Comfortable"),
                RadioGroupItem::new("expanded").label("Expanded"),
            ],
            selected_id: selected_id.map(|id| id.to_string().into()),
            enabled: true,
            template: default_radio_group_template(),
        }
    }
}
