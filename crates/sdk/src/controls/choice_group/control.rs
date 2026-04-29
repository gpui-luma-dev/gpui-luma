use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Window, div, prelude::*,
};

use super::{
    ChoiceGroupBuilder, ChoiceGroupItem, ChoiceGroupItemContentModel, ChoiceGroupItemPosition, ChoiceGroupModel,
    ChoiceGroupRenderItem, ChoiceGroupRenderModel, ChoiceGroupSelectionMode, ChoiceGroupStateMode,
    ChoiceGroupTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum ChoiceGroupEvent {
    Change {
        item_id: SharedString,
        label: SharedString,
        value: SharedString,
        selected: bool,
        selected_ids: Vec<SharedString>,
    },
}

pub struct ChoiceGroupControl {
    model: ChoiceGroupModel,
    focus_handle: gpui::FocusHandle,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
}

impl EventEmitter<ChoiceGroupEvent> for ChoiceGroupControl {}

impl ChoiceGroupControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ChoiceGroupBuilder {
        ChoiceGroupBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: ChoiceGroupBuilder, cx: &mut Context<Self>) -> Self {
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
        self.model.effective_selected_ids()
    }

    pub fn selected_id(&self) -> Option<&SharedString> {
        self.model.effective_selected_id()
    }

    pub fn selection_mode(&self) -> ChoiceGroupSelectionMode {
        self.model.selection_mode
    }

    pub fn state_mode(&self) -> ChoiceGroupStateMode {
        self.model.state_mode
    }

    pub fn set_state_mode(&mut self, mode: ChoiceGroupStateMode, cx: &mut Context<Self>) {
        self.model.state_mode = mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    /// Sets unmanaged/default selection.
    /// In managed mode, this updates fallback defaults only.
    pub fn set_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_selection_mode(&mut self, selection_mode: ChoiceGroupSelectionMode, cx: &mut Context<Self>) {
        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = ChoiceGroupItem>, cx: &mut Context<Self>) {
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

    /// Sets externally managed source-of-truth selected ids.
    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.set_managed_selected_ids(selected_ids);
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn clear_managed_selected_ids(&mut self, cx: &mut Context<Self>) {
        self.model.clear_managed_selected_ids();
        normalize_model(&mut self.model);
        cx.notify();
    }

    fn render_model<'a>(&'a self, _window: &Window) -> ChoiceGroupRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, _window);
        let effective_selected_ids = self.model.effective_selected_ids();
        let item_count = self.model.items.len();

        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.enabled;
                let selected = selected_ids_contain(effective_selected_ids, &item.id);
                let active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == &item.id);
                let position = item_position(index, item_count);

                let state = CompositeItemState {
                    hovered: enabled && self.hovered_item == Some(index),
                    pressed: enabled && self.pressed_item == Some(index),
                    disabled: !enabled,
                    selected,
                    // selected is persistent state; active/focus are interaction state
                    active: enabled && focus.focused && active,
                    focus_visible: enabled && focus.focus_visible && active,
                };

                let content_model = ChoiceGroupItemContentModel {
                    group_id: self.model.id.clone(),
                    item_id: item.id.clone(),
                    item_label: item.label.clone(),
                    item_value: item.value.clone(),
                    selected,
                    enabled,
                    position,
                    state,
                    selection_mode: self.model.selection_mode,
                    layout: self.model.layout,
                    group_enabled: self.model.enabled,
                };

                ChoiceGroupRenderItem {
                    id: &item.id,
                    label: &item.label,
                    value: &item.value,
                    selected,
                    enabled,
                    position,
                    state,
                    content_model,
                }
            })
            .collect();

        ChoiceGroupRenderModel {
            id: &self.model.id,
            items,
            content: &self.model.content,
            item_button_template: self.model.item_button_template.as_ref(),
            selected_ids: effective_selected_ids,
            active_id: self.model.active_id.as_ref(),
            selection_mode: self.model.selection_mode,
            layout: self.model.layout,
            state_mode: self.model.state_mode,
            kind: self.model.kind,
            size: self.model.size,
            enabled: self.model.enabled,
            focus,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ChoiceGroupTemplateHandlers {
        ChoiceGroupTemplateHandlers {
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

    fn commit_toggle_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let item = &self.model.items[index];
        let item_id = item.id.clone();
        let label = item.label.clone();
        let value = item.value.clone();

        let current = self.model.effective_selected_ids();
        let next_selected_ids = compute_next_selected_ids(&self.model.items, current, self.model.selection_mode, index);
        let selected = selected_ids_contain(&next_selected_ids, &item_id);

        if self.model.state_mode == ChoiceGroupStateMode::Unmanaged {
            self.model.default_selected_ids = next_selected_ids.clone();
            normalize_model(&mut self.model);
        }

        cx.emit(ChoiceGroupEvent::Change { item_id, label, value, selected, selected_ids: next_selected_ids });
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

        self.commit_toggle_index(index, cx);
    }

    fn move_active(&mut self, direction: ChoiceGroupDirection, cx: &mut Context<Self>) {
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
        self.move_active(ChoiceGroupDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(ChoiceGroupDirection::Next, cx);
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
            self.commit_toggle_index(index, cx);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChoiceGroupDirection {
    Previous,
    Next,
}

impl Focusable for ChoiceGroupControl {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ChoiceGroupControl {
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

pub(crate) fn normalize_model(model: &mut ChoiceGroupModel) {
    normalize_selected_ids(model);
    normalize_active_id(model);
}

pub(crate) fn normalize_selected_ids(model: &mut ChoiceGroupModel) {
    model.default_selected_ids =
        normalize_selected_id_list(&model.items, &model.default_selected_ids, model.selection_mode);

    if let Some(managed_selected_ids) = model.managed_selected_ids.clone() {
        model.managed_selected_ids =
            Some(normalize_selected_id_list(&model.items, &managed_selected_ids, model.selection_mode));
    }
}

pub(crate) fn normalize_active_id(model: &mut ChoiceGroupModel) {
    if enabled_item_index_by_id(&model.items, model.active_id.as_ref()).is_some() {
        return;
    }

    model.active_id = model
        .effective_selected_ids()
        .iter()
        .find(|selected_id| enabled_item_index_by_id(&model.items, Some(selected_id)).is_some())
        .cloned()
        .or_else(|| model.items.iter().find(|item| item.enabled).map(|item| item.id.clone()));
}

fn normalize_selected_id_list(
    items: &[ChoiceGroupItem],
    selected_ids: &[SharedString],
    selection_mode: ChoiceGroupSelectionMode,
) -> Vec<SharedString> {
    let mut normalized = Vec::new();

    for selected_id in selected_ids {
        if normalized.iter().any(|id| id == selected_id) {
            continue;
        }

        if items.iter().any(|item| item.enabled && &item.id == selected_id) {
            normalized.push(selected_id.clone());
        }

        if selection_mode == ChoiceGroupSelectionMode::Single && !normalized.is_empty() {
            break;
        }
    }

    normalized
}

pub(crate) fn enabled_item_index_by_id(items: &[ChoiceGroupItem], item_id: Option<&SharedString>) -> Option<usize> {
    let item_id = item_id?;
    items.iter().position(|item| item.enabled && &item.id == item_id)
}

pub(crate) fn first_enabled_index(items: &[ChoiceGroupItem]) -> Option<usize> {
    items.iter().position(|item| item.enabled)
}

pub(crate) fn last_enabled_index(items: &[ChoiceGroupItem]) -> Option<usize> {
    items.iter().rposition(|item| item.enabled)
}

pub(crate) fn next_enabled_index(
    items: &[ChoiceGroupItem],
    current_index: Option<usize>,
    direction: ChoiceGroupDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        ChoiceGroupDirection::Previous => len - 1,
        ChoiceGroupDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ChoiceGroupDirection::Previous => len - 1,
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

fn compute_next_selected_ids(
    items: &[ChoiceGroupItem],
    current: &[SharedString],
    selection_mode: ChoiceGroupSelectionMode,
    toggled_index: usize,
) -> Vec<SharedString> {
    let Some(item) = items.get(toggled_index) else {
        return current.to_vec();
    };
    if !item.enabled {
        return current.to_vec();
    }

    let already_selected = selected_ids_contain(current, &item.id);

    match selection_mode {
        ChoiceGroupSelectionMode::Single => {
            if already_selected {
                Vec::new()
            } else {
                vec![item.id.clone()]
            }
        }
        ChoiceGroupSelectionMode::Multiple => {
            if already_selected {
                current.iter().filter(|selected_id| *selected_id != &item.id).cloned().collect()
            } else {
                let mut next = current.to_vec();
                next.push(item.id.clone());
                normalize_selected_id_list(items, &next, ChoiceGroupSelectionMode::Multiple)
            }
        }
    }
}

fn item_position(index: usize, item_count: usize) -> ChoiceGroupItemPosition {
    match (index, item_count) {
        (_, 0 | 1) => ChoiceGroupItemPosition::Only,
        (0, _) => ChoiceGroupItemPosition::First,
        (index, item_count) if index + 1 == item_count => ChoiceGroupItemPosition::Last,
        _ => ChoiceGroupItemPosition::Middle,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{SharedString, prelude::*};

    use super::{
        ChoiceGroupDirection, ChoiceGroupModel, ChoiceGroupSelectionMode, compute_next_selected_ids,
        enabled_item_index_by_id, next_enabled_index, normalize_active_id, normalize_selected_ids,
    };
    use crate::controls::button_family::ButtonKind;
    use crate::controls::choice_group::{
        ChoiceGroupItem, ChoiceGroupLayout, ChoiceGroupSize, ChoiceGroupStateMode, default_choice_group_template,
    };

    #[test]
    fn normalize_single_keeps_first_valid_default_selection() {
        let mut model = model_with_selected(
            ChoiceGroupSelectionMode::Single,
            ["missing", "bottom", "left"],
            Some(["left", "top"]),
            Some("missing"),
        );

        normalize_selected_ids(&mut model);

        assert_eq!(selected_ids_to_strings(&model.default_selected_ids), vec!["bottom"]);
        assert_eq!(selected_ids_to_strings(model.managed_selected_ids.as_deref().unwrap_or_default()), vec!["left"]);
    }

    #[test]
    fn normalize_multiple_removes_missing_disabled_and_duplicates() {
        let mut model = model_with_selected(
            ChoiceGroupSelectionMode::Multiple,
            ["missing", "top", "right", "top", "bottom"],
            Some(["left", "left", "missing"]),
            None,
        );

        normalize_selected_ids(&mut model);

        assert_eq!(selected_ids_to_strings(&model.default_selected_ids), vec!["top", "bottom"]);
        assert_eq!(selected_ids_to_strings(model.managed_selected_ids.as_deref().unwrap_or_default()), vec!["left"]);
    }

    #[test]
    fn normalize_active_prefers_effective_selection_then_first_enabled_item() {
        let mut model =
            model_with_selected(ChoiceGroupSelectionMode::Multiple, ["bottom"], Some(["left"]), Some("missing"));
        model.state_mode = ChoiceGroupStateMode::Managed;

        normalize_active_id(&mut model);
        assert_eq!(model.active_id.as_ref().map(ToString::to_string), Some("left".to_string()));

        model.managed_selected_ids = None;
        model.default_selected_ids.clear();
        model.active_id = Some(ss("missing"));
        normalize_active_id(&mut model);

        assert_eq!(model.active_id.as_ref().map(ToString::to_string), Some("top".to_string()));
    }

    #[test]
    fn navigation_skips_disabled_items_and_wraps() {
        let items = items();
        assert_eq!(next_enabled_index(&items, Some(0), ChoiceGroupDirection::Next), Some(1));
        assert_eq!(next_enabled_index(&items, Some(1), ChoiceGroupDirection::Next), Some(3));
        assert_eq!(next_enabled_index(&items, Some(0), ChoiceGroupDirection::Previous), Some(3));
    }

    #[test]
    fn enabled_item_index_ignores_disabled_item() {
        let items = items();
        let item_id = ss("right");
        assert_eq!(enabled_item_index_by_id(&items, Some(&item_id)), None);
    }

    #[test]
    fn toggle_math_single_and_multiple() {
        let items = items();

        let current = vec![ss("top")];
        let next_single = compute_next_selected_ids(&items, &current, ChoiceGroupSelectionMode::Single, 0);
        assert!(next_single.is_empty());

        let next_single_select_other = compute_next_selected_ids(&items, &current, ChoiceGroupSelectionMode::Single, 1);
        assert_eq!(selected_ids_to_strings(&next_single_select_other), vec!["bottom"]);

        let current_multi = vec![ss("top")];
        let next_multi = compute_next_selected_ids(&items, &current_multi, ChoiceGroupSelectionMode::Multiple, 3);
        assert_eq!(selected_ids_to_strings(&next_multi), vec!["top", "left"]);
    }

    fn model_with_selected<const D: usize, const M: usize>(
        selection_mode: ChoiceGroupSelectionMode,
        default_ids: [&str; D],
        managed_ids: Option<[&str; M]>,
        active_id: Option<&str>,
    ) -> ChoiceGroupModel {
        ChoiceGroupModel {
            id: "choice".into(),
            items: items(),
            default_selected_ids: default_ids.into_iter().map(|s| s.to_string().into()).collect(),
            managed_selected_ids: managed_ids
                .map(|ids| ids.into_iter().map(|s| s.to_string().into()).collect::<Vec<SharedString>>()),
            active_id: active_id.map(|s| s.to_string().into()),
            selection_mode,
            layout: ChoiceGroupLayout::Horizontal,
            state_mode: ChoiceGroupStateMode::Unmanaged,
            kind: ButtonKind::Standard,
            size: ChoiceGroupSize::Md,
            enabled: true,
            template: default_choice_group_template(),
            content: std::sync::Arc::new(|_, _| gpui::div().into_any_element()),
            item_button_template: None,
        }
    }

    fn items() -> Vec<ChoiceGroupItem> {
        vec![
            ChoiceGroupItem::new(ss("top"), ss("top")).label(ss("Top")),
            ChoiceGroupItem::new(ss("bottom"), ss("bottom")).label(ss("Bottom")),
            ChoiceGroupItem::new(ss("right"), ss("right")).label(ss("Right")).enabled(false),
            ChoiceGroupItem::new(ss("left"), ss("left")).label(ss("Left")),
        ]
    }

    fn selected_ids_to_strings(ids: &[SharedString]) -> Vec<String> {
        ids.iter().map(ToString::to_string).collect()
    }

    fn ss(value: &'static str) -> SharedString {
        SharedString::new_static(value)
    }
}
