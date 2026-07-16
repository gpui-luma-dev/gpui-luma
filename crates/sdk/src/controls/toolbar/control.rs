use gpui::{App, Context, Focusable, IntoElement, Render, Window, div, prelude::*};

use super::model::{ToolbarBuilder, ToolbarItemKind, ToolbarItemRenderModel, ToolbarModel, ToolbarRenderModel};
use super::template::{ToolbarRenderedItem, render_fallback_toolbar_item};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use crate::theme::observe_theme_revision;

pub struct Toolbar {
    model: ToolbarModel,
    focus_handle: gpui::FocusHandle,
    active_index: Option<usize>,
    item_focus_handles: Vec<Option<gpui::FocusHandle>>,
}

impl Toolbar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<gpui::SharedString>) -> ToolbarBuilder {
        ToolbarBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ToolbarBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let active_index = first_focusable_item_index(&builder.model);
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            active_index,
            item_focus_handles: Vec::new(),
        }
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = super::ToolbarItem>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.item_focus_handles.clear();
        self.normalize_active_index();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.normalize_active_index();
        cx.notify();
    }

    fn normalize_active_index(&mut self) {
        if self.active_index.is_some_and(|index| self.can_focus_index(index)) {
            return;
        }

        self.active_index = first_focusable_item_index(&self.model);
    }

    fn can_focus_index(&self, index: usize) -> bool {
        self.model.enabled
            && self
                .model
                .items
                .get(index)
                .is_some_and(|item| item.enabled && item.kind != ToolbarItemKind::Separator)
    }

    fn focus_handle_for_index(&self, index: usize) -> Option<gpui::FocusHandle> {
        self.item_focus_handles
            .get(index)
            .and_then(Clone::clone)
            .or_else(|| self.model.items.get(index).and_then(|item| item.focus_handle.clone()))
    }

    fn focused_item_index(&self, window: &Window) -> Option<usize> {
        self.item_focus_handles.iter().enumerate().find_map(|(index, focus_handle)| {
            focus_handle.as_ref().is_some_and(|focus| focus.is_focused(window)).then_some(index)
        })
    }

    fn focus_active_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.normalize_active_index();

        if let Some(index) = self.active_index
            && let Some(focus_handle) = self.focus_handle_for_index(index)
        {
            focus_handle.focus(window, cx);
        } else {
            self.focus_handle.focus(window, cx);
        }
    }

    fn move_active(&mut self, direction: ToolbarDirection, window: &mut Window, cx: &mut Context<Self>) {
        let current_index = self.focused_item_index(window).or(self.active_index);

        if let Some(index) = next_focusable_item_index(&self.model, current_index, direction) {
            self.active_index = Some(index);
            self.focus_active_item(window, cx);
            cx.notify();
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, window: &mut Window, cx: &mut Context<Self>) {
        let index = if first {
            first_focusable_item_index(&self.model)
        } else {
            last_focusable_item_index(&self.model)
        };

        if let Some(index) = index {
            self.active_index = Some(index);
            self.focus_active_item(window, cx);
            cx.notify();
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ToolbarDirection::Previous, window, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ToolbarDirection::Next, window, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, window, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, window, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_active_item(window, cx);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolbarDirection {
    Previous,
    Next,
}

impl Focusable for Toolbar {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Toolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.normalize_active_index();
        let item_models: Vec<_> = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| ToolbarItemRenderModel {
                id: item.id.clone(),
                kind: item.kind,
                index,
                enabled: self.model.enabled && item.enabled,
            })
            .collect();
        let model = ToolbarRenderModel { id: &self.model.id, items: item_models, enabled: self.model.enabled };
        let mut item_focus_handles = Vec::with_capacity(self.model.items.len());
        let rendered_items = self
            .model
            .items
            .iter()
            .zip(model.items.iter())
            .map(|(item, item_model)| {
                let (element, focus_handle) = item
                    .content
                    .as_ref()
                    .map(|content| {
                        let hosted = content.present(item_model, window, cx);
                        (hosted.element, hosted.focus_handle)
                    })
                    .unwrap_or_else(|| (render_fallback_toolbar_item(item_model), None));
                item_focus_handles.push(focus_handle.or_else(|| item.focus_handle.clone()));

                ToolbarRenderedItem { kind: item.kind, element }
            })
            .collect();
        self.item_focus_handles = item_focus_handles;

        div()
            .child(
                self.model
                    .template
                    .render(&model, rendered_items, window, cx)
                    .track_focus(&self.focus_handle)
                    .key_context(ControlKeyProfile::TabList.context())
                    .on_action(cx.listener(Self::handle_select_previous_item))
                    .on_action(cx.listener(Self::handle_select_next_item))
                    .on_action(cx.listener(Self::handle_select_first_item))
                    .on_action(cx.listener(Self::handle_select_last_item))
                    .on_action(cx.listener(Self::handle_activate_control)),
            )
            .into_any_element()
    }
}

fn first_focusable_item_index(model: &ToolbarModel) -> Option<usize> {
    model
        .items
        .iter()
        .position(|item| model.enabled && item.enabled && item.kind != ToolbarItemKind::Separator)
}

fn last_focusable_item_index(model: &ToolbarModel) -> Option<usize> {
    model
        .items
        .iter()
        .rposition(|item| model.enabled && item.enabled && item.kind != ToolbarItemKind::Separator)
}

fn next_focusable_item_index(
    model: &ToolbarModel,
    current_index: Option<usize>,
    direction: ToolbarDirection,
) -> Option<usize> {
    let len = model.items.len();
    if !model.enabled || len == 0 {
        return None;
    }

    let step = match direction {
        ToolbarDirection::Previous => len - 1,
        ToolbarDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ToolbarDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        let item = &model.items[index];
        if item.enabled && item.kind != ToolbarItemKind::Separator {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}
