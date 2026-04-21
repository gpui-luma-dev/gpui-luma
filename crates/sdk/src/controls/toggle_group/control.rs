use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Window, div, prelude::*,
};

use super::{
    ToggleGroupBuilder, ToggleGroupItem, ToggleGroupItemPosition, ToggleGroupRenderItem, ToggleGroupRenderModel,
    ToggleGroupSelectionMode, ToggleGroupTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::controls::toggle_group::model::ToggleGroupModel;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum ToggleGroupEvent {
    Change { item_id: SharedString, label: SharedString, selected: bool, selected_ids: Vec<SharedString> },
}

pub struct ToggleGroup {
    model: ToggleGroupModel,
    focus_handle: gpui::FocusHandle,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
}

impl EventEmitter<ToggleGroupEvent> for ToggleGroup {}

impl ToggleGroup {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ToggleGroupBuilder {
        ToggleGroupBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: ToggleGroupBuilder, cx: &mut Context<Self>) -> Self {
        normalize_model(&mut builder.model);
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            hovered_item: None,
            pressed_item: None,
        }
    }

    pub fn selected_ids(&self) -> &[SharedString] {
        &self.model.selected_ids
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.model.selected_ids.first()
    }

    pub fn selection_mode(&self) -> ToggleGroupSelectionMode {
        self.model.selection_mode
    }

    pub fn set_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.selected_ids = selected_ids.into_iter().map(Into::into).collect();
        normalize_selected_ids(&mut self.model);
        normalize_active_id(&mut self.model);
        cx.notify();
    }

    pub fn set_selection_mode(&mut self, selection_mode: ToggleGroupSelectionMode, cx: &mut Context<Self>) {
        self.model.selection_mode = selection_mode;
        normalize_selected_ids(&mut self.model);
        normalize_active_id(&mut self.model);
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = ToggleGroupItem>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_item = None;
        self.pressed_item = None;
        normalize_model(&mut self.model);
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

    fn render_model<'a>(&'a self, window: &Window) -> ToggleGroupRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let item_count = self.model.items.len();
        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.enabled;
                let selected = selected_ids_contain(&self.model.selected_ids, &item.id);
                let active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == &item.id);

                ToggleGroupRenderItem {
                    id: &item.id,
                    label: &item.label,
                    selected,
                    enabled,
                    position: item_position(index, item_count),
                    state: CompositeItemState {
                        hovered: enabled && self.hovered_item == Some(index),
                        pressed: enabled && self.pressed_item == Some(index),
                        disabled: !enabled,
                        selected,
                        active: enabled && focus.focused && active,
                        focus_visible: enabled && focus.focus_visible && active,
                    },
                }
            })
            .collect();

        ToggleGroupRenderModel {
            id: &self.model.id,
            items,
            selected_ids: &self.model.selected_ids,
            active_id: self.model.active_id.as_ref(),
            selection_mode: self.model.selection_mode,
            kind: self.model.kind,
            size: self.model.size,
            enabled: self.model.enabled,
            focus,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ToggleGroupTemplateHandlers {
        ToggleGroupTemplateHandlers {
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

    fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(|item| item.enabled)
    }

    fn active_index(&self) -> Option<usize> {
        enabled_item_index_by_id(&self.model.items, self.model.active_id.as_ref())
    }

    fn toggle_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let item = &self.model.items[index];
        let already_selected = selected_ids_contain(&self.model.selected_ids, &item.id);
        let selected = !already_selected;

        match self.model.selection_mode {
            ToggleGroupSelectionMode::Single => {
                if already_selected {
                    self.model.selected_ids.clear();
                } else {
                    self.model.selected_ids = vec![item.id.clone()];
                }
            }
            ToggleGroupSelectionMode::Multiple => {
                if already_selected {
                    self.model.selected_ids.retain(|selected_id| selected_id != &item.id);
                } else {
                    self.model.selected_ids.push(item.id.clone());
                }
            }
        }

        let item_id = item.id.clone();
        let label = item.label.clone();
        let selected_ids = self.model.selected_ids.clone();
        cx.emit(ToggleGroupEvent::Change { item_id, label, selected, selected_ids });
        cx.notify();
        true
    }

    fn set_active_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let item = &self.model.items[index];
        if self.model.active_id.as_ref().is_some_and(|active_id| active_id == &item.id) {
            return false;
        }

        self.model.active_id = Some(item.id.clone());
        cx.notify();
        true
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_item(index) {
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
        if self.can_use_item(index) {
            self.pressed_item = Some(index);
            self.model.active_id = Some(self.model.items[index].id.clone());
            self.focus_handle.focus(window, cx);
            cx.notify();
        }
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_item.is_some() {
            self.pressed_item = None;
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.toggle_index(index, cx);
    }

    fn move_active(&mut self, direction: ToggleGroupDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if let Some(next_index) = next_enabled_index(&self.model.items, self.active_index(), direction) {
            self.set_active_index(next_index, cx);
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let next_index = if first {
            first_enabled_index(&self.model.items)
        } else {
            last_enabled_index(&self.model.items)
        };

        if let Some(next_index) = next_index {
            self.set_active_index(next_index, cx);
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ToggleGroupDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ToggleGroupDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if let Some(index) = self.active_index() {
            self.toggle_index(index, cx);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToggleGroupDirection {
    Previous,
    Next,
}

impl Focusable for ToggleGroup {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToggleGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
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

pub(crate) fn normalize_model(model: &mut ToggleGroupModel) {
    normalize_selected_ids(model);
    normalize_active_id(model);
}

pub(crate) fn normalize_selected_ids(model: &mut ToggleGroupModel) {
    let mut normalized = Vec::new();

    for selected_id in &model.selected_ids {
        if normalized.iter().any(|id| id == selected_id) {
            continue;
        }

        if model.items.iter().any(|item| item.enabled && &item.id == selected_id) {
            normalized.push(selected_id.clone());
        }

        if model.selection_mode == ToggleGroupSelectionMode::Single && !normalized.is_empty() {
            break;
        }
    }

    model.selected_ids = normalized;
}

pub(crate) fn normalize_active_id(model: &mut ToggleGroupModel) {
    if enabled_item_index_by_id(&model.items, model.active_id.as_ref()).is_some() {
        return;
    }

    model.active_id = model
        .selected_ids
        .iter()
        .find(|selected_id| enabled_item_index_by_id(&model.items, Some(selected_id)).is_some())
        .cloned()
        .or_else(|| model.items.iter().find(|item| item.enabled).map(|item| item.id.clone()));
}

pub(crate) fn enabled_item_index_by_id(items: &[ToggleGroupItem], item_id: Option<&SharedString>) -> Option<usize> {
    let item_id = item_id?;

    items.iter().position(|item| item.enabled && &item.id == item_id)
}

pub(crate) fn first_enabled_index(items: &[ToggleGroupItem]) -> Option<usize> {
    items.iter().position(|item| item.enabled)
}

pub(crate) fn last_enabled_index(items: &[ToggleGroupItem]) -> Option<usize> {
    items.iter().rposition(|item| item.enabled)
}

pub(crate) fn next_enabled_index(
    items: &[ToggleGroupItem],
    current_index: Option<usize>,
    direction: ToggleGroupDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        ToggleGroupDirection::Previous => len - 1,
        ToggleGroupDirection::Next => 1,
    };
    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ToggleGroupDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].enabled {
            return Some(index);
        }

        index = (index + step) % len;
    }

    None
}

fn selected_ids_contain(selected_ids: &[SharedString], item_id: &SharedString) -> bool {
    selected_ids.iter().any(|selected_id| selected_id == item_id)
}

fn item_position(index: usize, item_count: usize) -> ToggleGroupItemPosition {
    match (index, item_count) {
        (_, 0 | 1) => ToggleGroupItemPosition::Only,
        (0, _) => ToggleGroupItemPosition::First,
        (index, item_count) if index + 1 == item_count => ToggleGroupItemPosition::Last,
        _ => ToggleGroupItemPosition::Middle,
    }
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{
        ToggleGroupDirection, ToggleGroupModel, enabled_item_index_by_id, next_enabled_index, normalize_active_id,
        normalize_selected_ids,
    };
    use crate::controls::button_family::ButtonKind;
    use crate::controls::toggle_group::{
        ToggleGroupItem, ToggleGroupSelectionMode, ToggleGroupSize, default_toggle_group_template,
    };

    #[test]
    fn normalize_single_keeps_first_valid_selected_item() {
        let mut model =
            model_with_selected(ToggleGroupSelectionMode::Single, ["missing", "bottom", "left"], Some("top"));

        normalize_selected_ids(&mut model);

        assert_eq!(selected_ids(&model), vec!["bottom"]);
    }

    #[test]
    fn normalize_multiple_removes_missing_disabled_and_duplicate_items() {
        let mut model =
            model_with_selected(ToggleGroupSelectionMode::Multiple, ["missing", "top", "right", "top", "bottom"], None);

        normalize_selected_ids(&mut model);

        assert_eq!(selected_ids(&model), vec!["top", "bottom"]);
    }

    #[test]
    fn normalize_active_prefers_selected_item_then_first_enabled_item() {
        let mut model = model_with_selected(ToggleGroupSelectionMode::Multiple, ["bottom"], Some("missing"));

        normalize_active_id(&mut model);

        assert_eq!(model.active_id.as_ref().map(|id| id.to_string()), Some("bottom".to_string()));

        model.selected_ids.clear();
        model.active_id = Some(ss("missing"));
        normalize_active_id(&mut model);

        assert_eq!(model.active_id.as_ref().map(|id| id.to_string()), Some("top".to_string()));
    }

    #[test]
    fn navigation_skips_disabled_items_and_wraps() {
        let items = items();

        assert_eq!(next_enabled_index(&items, Some(0), ToggleGroupDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, Some(1), ToggleGroupDirection::Next), Some(3));
        assert_eq!(next_enabled_index(&items, Some(0), ToggleGroupDirection::Previous), Some(3));
    }

    #[test]
    fn enabled_item_index_ignores_disabled_item() {
        let items = items();
        let item_id = ss("right");

        assert_eq!(enabled_item_index_by_id(&items, Some(&item_id)), None);
    }

    fn model_with_selected<const N: usize>(
        selection_mode: ToggleGroupSelectionMode,
        selected_ids: [&str; N],
        active_id: Option<&str>,
    ) -> ToggleGroupModel {
        ToggleGroupModel {
            id: "placement".into(),
            items: items(),
            selected_ids: selected_ids.into_iter().map(|selected_id| selected_id.to_string().into()).collect(),
            active_id: active_id.map(|active_id| active_id.to_string().into()),
            selection_mode,
            kind: ButtonKind::Default,
            size: ToggleGroupSize::Md,
            enabled: true,
            template: default_toggle_group_template(),
        }
    }

    fn items() -> Vec<ToggleGroupItem> {
        vec![
            ToggleGroupItem::new(ss("top")).label(ss("Top")),
            ToggleGroupItem::new(ss("bottom")).label(ss("Bottom")),
            ToggleGroupItem::new(ss("right")).label(ss("Right")).enabled(false),
            ToggleGroupItem::new(ss("left")).label(ss("Left")),
        ]
    }

    fn selected_ids(model: &ToggleGroupModel) -> Vec<String> {
        model.selected_ids.iter().map(ToString::to_string).collect()
    }

    fn ss(value: &'static str) -> SharedString {
        SharedString::new_static(value)
    }
}
