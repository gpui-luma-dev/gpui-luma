use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, prelude::*,
};

use super::model::{
    ControlGroupBuilder, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupModel, ControlGroupRenderModel,
    ControlGroupArrowPolicy, ControlGroupFocusStrategy, ControlGroupFocusTarget, ControlGroupStateMode,
    ControlSelectionMode,
};
use super::template::ControlGroupTemplateHandlers;
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
pub enum ControlGroupEvent {
    Activate { activated_id: SharedString },
    Change { changed_id: SharedString, selected: bool, selected_ids: Vec<SharedString> },
}

pub struct ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    model: ControlGroupModel<T>,
    focus_handle: gpui::FocusHandle,
    focus_subscription: Option<Subscription>,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
}

impl<T> EventEmitter<ControlGroupEvent> for ControlGroupControl<T> where T: ControlGroupItemLike + 'static {}

impl<T> ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ControlGroupBuilder<T> {
        ControlGroupBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: ControlGroupBuilder<T>, cx: &mut Context<Self>) -> Self {
        normalize_model(&mut builder.model);
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            focus_subscription: None,
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

    pub fn active_id(&self) -> Option<&SharedString> {
        self.model.active_id.as_ref()
    }

    pub fn selection_mode(&self) -> ControlSelectionMode {
        self.model.selection_mode
    }

    pub fn state_mode(&self) -> ControlGroupStateMode {
        self.model.state_mode
    }

    pub fn set_state_mode(&mut self, mode: ControlGroupStateMode, cx: &mut Context<Self>) {
        self.model.state_mode = mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_selected_ids<I, S>(&mut self, selected_ids: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_selection_mode(&mut self, selection_mode: ControlSelectionMode, cx: &mut Context<Self>) {
        self.model.selection_mode = selection_mode;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_layout(&mut self, layout: super::model::ControlGroupLayout, cx: &mut Context<Self>) {
        self.model.layout = layout;
        cx.notify();
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = T>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_item = None;
        self.pressed_item = None;
        normalize_model(&mut self.model);
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_subscription = None;

        if !enabled {
            self.hovered_item = None;
            self.pressed_item = None;
        }

        cx.notify();
    }

    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let active_id = active_id.into();
        if enabled_item_index_by_id(&self.model.items, Some(&active_id)).is_some() {
            self.model.active_id = Some(active_id);
            cx.notify();
        }
    }

    pub fn clear_active(&mut self, cx: &mut Context<Self>) {
        self.model.active_id = None;
        normalize_active_id(&mut self.model);
        cx.notify();
    }

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

    pub fn set_item_template(
        &mut self,
        item_template: Option<crate::controls::control_group::ControlGroupItemTemplate<T>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_template = item_template;
        cx.notify();
    }

    pub fn set_item_element_template(
        &mut self,
        item_element_template: Option<crate::controls::control_group::ControlGroupItemElementTemplate<T>>,
        cx: &mut Context<Self>,
    ) {
        self.model.item_element_template = item_element_template;
        cx.notify();
    }

    pub fn set_template(
        &mut self,
        template: crate::controls::control_group::ControlGroupTemplate<T>,
        cx: &mut Context<Self>,
    ) {
        self.model.template = template;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &mut Window, cx: &mut App) -> ControlGroupRenderModel<'a, T> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let effective_selected_ids = self.model.effective_selected_ids();
        let item_count = self.model.items.len();
        let keyboard_focus_visible = window.last_input_was_keyboard();

        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.is_enabled();
                let selected = selected_ids_contain(effective_selected_ids, item.id());
                let active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == item.id());
                let mut item_active = enabled && focus.focused && active;
                let mut item_focus_visible = enabled && focus.focus_visible && active;

                if enabled
                    && active
                    && self.model.focus_strategy == ControlGroupFocusStrategy::RovingItemFocus
                    && let Some(target) = self.focus_target_for_index(index, window, cx)
                    && target.focus_handle.is_focused(window)
                {
                    item_active = true;
                    item_focus_visible = keyboard_focus_visible;
                }

                ControlGroupItemRenderModel {
                    group_id: &self.model.id,
                    item,
                    index,
                    sibling_count: item_count,
                    selected,
                    active,
                    enabled,
                    state: CompositeItemState {
                        hovered: enabled && self.hovered_item == Some(index),
                        pressed: enabled && self.pressed_item == Some(index),
                        disabled: !enabled,
                        selected,
                        active: item_active,
                        focus_visible: item_focus_visible,
                    },
                    selection_mode: self.model.selection_mode,
                }
            })
            .collect();

        ControlGroupRenderModel {
            id: &self.model.id,
            items,
            selected_ids: effective_selected_ids,
            active_id: self.model.active_id.as_ref(),
            selection_mode: self.model.selection_mode,
            state_mode: self.model.state_mode,
            enabled: self.model.enabled,
            layout: self.model.layout,
            focus,
            focus_strategy: self.model.focus_strategy,
            item_template: self.model.item_template.as_ref(),
            item_element_template: self.model.item_element_template.as_ref(),
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> ControlGroupTemplateHandlers {
        ControlGroupTemplateHandlers {
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
                    Box::new(cx.listener(move |this, event, _window, cx| {
                        this.handle_item_click(index, event, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(ControlGroupItemLike::is_enabled)
    }

    fn active_index(&self) -> Option<usize> {
        enabled_item_index_by_id(&self.model.items, self.model.active_id.as_ref())
    }

    fn commit_toggle_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let Some(item) = self.model.items.get(index) else {
            return false;
        };

        let item_id = item.id().clone();
        let current = self.model.effective_selected_ids();
        let next_selected_ids = compute_next_selected_ids(&self.model.items, current, self.model.selection_mode, index);
        let selected = selected_ids_contain(&next_selected_ids, &item_id);

        if next_selected_ids == current {
            cx.emit(ControlGroupEvent::Activate { activated_id: item_id });
            return false;
        }

        if self.model.state_mode == ControlGroupStateMode::Unmanaged {
            self.model.default_selected_ids = next_selected_ids.clone();
            normalize_model(&mut self.model);
        }

        cx.emit(ControlGroupEvent::Change { changed_id: item_id.clone(), selected, selected_ids: next_selected_ids });
        cx.emit(ControlGroupEvent::Activate { activated_id: item_id });
        cx.notify();
        true
    }

    fn focus_target_for_index(
        &self,
        index: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<ControlGroupFocusTarget> {
        let item = self.model.items.get(index)?;
        let provider = self.model.focus_target_provider.as_ref()?;
        provider(item, window, cx)
    }

    fn focus_item_for_strategy(&self, index: usize, window: &mut Window, cx: &mut App) {
        match self.model.focus_strategy {
            ControlGroupFocusStrategy::ActiveDescendant => {
                self.focus_handle.focus(window, cx);
            }
            ControlGroupFocusStrategy::RovingItemFocus => {
                if let Some(target) = self.focus_target_for_index(index, window, cx) {
                    target.focus_handle.focus(window, cx);
                } else {
                    self.focus_handle.focus(window, cx);
                }
            }
        }
    }

    fn active_focus_target(&self, window: &mut Window, cx: &mut App) -> Option<ControlGroupFocusTarget> {
        let index = self.active_index()?;
        self.focus_target_for_index(index, window, cx)
    }

    fn group_handles_arrow_action(&self, window: &mut Window, cx: &mut App) -> bool {
        if self.model.focus_strategy == ControlGroupFocusStrategy::ActiveDescendant {
            return true;
        }

        !matches!(
            self.active_focus_target(window, cx).map(|target| {
                let focused = target.focus_handle.is_focused(window);
                (target.arrow_policy, focused)
            }),
            Some((ControlGroupArrowPolicy::ChildOwnsWhenFocused, true))
        )
    }

    fn redirect_roving_focus_on_group_entry(&self, window: &mut Window, cx: &mut App) {
        if self.model.focus_strategy != ControlGroupFocusStrategy::RovingItemFocus {
            return;
        }

        if let Some(target) = self.active_focus_target(window, cx) {
            target.focus_handle.focus(window, cx);
        }
    }

    fn set_active_index(&mut self, index: usize, window: Option<&mut Window>, cx: &mut Context<Self>) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let Some(item) = self.model.items.get(index) else {
            return false;
        };

        if self.model.active_id.as_ref().is_some_and(|active_id| active_id == item.id()) {
            return false;
        }

        self.model.active_id = Some(item.id().clone());
        if let Some(window) = window {
            self.focus_item_for_strategy(index, window, cx);
        }
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
            self.model.active_id = Some(self.model.items[index].id().clone());
            self.focus_item_for_strategy(index, window, cx);
            cx.notify();
        }
    }

    fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_item.is_some() {
            self.pressed_item = None;
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.commit_toggle_index(index, cx);
    }

    fn move_active(&mut self, direction: ControlGroupDirection, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if let Some(next_index) = next_enabled_index(&self.model.items, self.active_index(), direction) {
            let changed = self.set_active_index(next_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(next_index, cx);
            }
        }
    }

    fn move_active_to_boundary(&mut self, first: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let next_index = if first {
            first_enabled_index(&self.model.items)
        } else {
            last_enabled_index(&self.model.items)
        };

        if let Some(next_index) = next_index {
            let changed = self.set_active_index(next_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(next_index, cx);
            }
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(window, cx) {
            self.move_active(ControlGroupDirection::Previous, window, cx);
        }
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(window, cx) {
            self.move_active(ControlGroupDirection::Next, window, cx);
        }
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(window, cx) {
            self.move_active_to_boundary(true, window, cx);
        }
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(window, cx) {
            self.move_active_to_boundary(false, window, cx);
        }
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
pub(crate) enum ControlGroupDirection {
    Previous,
    Next,
}

impl<T> Focusable for ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_subscription.is_none() {
            self.focus_subscription = Some(cx.on_focus(&self.focus_handle, window, Self::handle_group_focus_entry));
        }

        let model = self.render_model(window, cx);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                (self.model.template)(&model, handlers, window, cx)
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

impl<T> ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn handle_group_focus_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.redirect_roving_focus_on_group_entry(window, cx);
    }
}

pub(crate) fn normalize_model<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    normalize_selected_ids(model);
    normalize_active_id(model);
}

pub(crate) fn normalize_selected_ids<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    model.default_selected_ids =
        normalize_selected_id_list(&model.items, &model.default_selected_ids, model.selection_mode);

    if let Some(managed_selected_ids) = model.managed_selected_ids.clone() {
        model.managed_selected_ids =
            Some(normalize_selected_id_list(&model.items, &managed_selected_ids, model.selection_mode));
    }
}

pub(crate) fn normalize_active_id<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    if enabled_item_index_by_id(&model.items, model.active_id.as_ref()).is_some() {
        return;
    }

    model.active_id = model
        .effective_selected_ids()
        .iter()
        .find(|selected_id| enabled_item_index_by_id(&model.items, Some(selected_id)).is_some())
        .cloned()
        .or_else(|| model.items.iter().find(|item| item.is_enabled()).map(|item| item.id().clone()));
}

fn normalize_selected_id_list<T>(
    items: &[T],
    selected_ids: &[SharedString],
    selection_mode: ControlSelectionMode,
) -> Vec<SharedString>
where
    T: ControlGroupItemLike + 'static,
{
    let mut normalized = Vec::new();

    for selected_id in selected_ids {
        if normalized.iter().any(|id| id == selected_id) {
            continue;
        }

        if items.iter().any(|item| item.is_enabled() && item.id() == selected_id) {
            normalized.push(selected_id.clone());
        }

        if matches!(selection_mode, ControlSelectionMode::SingleRequired | ControlSelectionMode::SingleAllowNone)
            && !normalized.is_empty()
        {
            break;
        }
    }

    if selection_mode == ControlSelectionMode::SingleRequired
        && normalized.is_empty()
        && let Some(first_enabled_id) = items.iter().find(|item| item.is_enabled()).map(|item| item.id().clone())
    {
        normalized.push(first_enabled_id);
    }

    normalized
}

pub(crate) fn enabled_item_index_by_id<T>(items: &[T], item_id: Option<&SharedString>) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    let item_id = item_id?;
    items.iter().position(|item| item.is_enabled() && item.id() == item_id)
}

pub(crate) fn first_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    items.iter().position(ControlGroupItemLike::is_enabled)
}

pub(crate) fn last_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    items.iter().rposition(ControlGroupItemLike::is_enabled)
}

pub(crate) fn next_enabled_index<T>(
    items: &[T],
    current_index: Option<usize>,
    direction: ControlGroupDirection,
) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        ControlGroupDirection::Previous => len - 1,
        ControlGroupDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ControlGroupDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].is_enabled() {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

fn selected_ids_contain(selected_ids: &[SharedString], item_id: &SharedString) -> bool {
    selected_ids.iter().any(|selected_id| selected_id == item_id)
}

fn compute_next_selected_ids<T>(
    items: &[T],
    current: &[SharedString],
    selection_mode: ControlSelectionMode,
    toggled_index: usize,
) -> Vec<SharedString>
where
    T: ControlGroupItemLike + 'static,
{
    let Some(item) = items.get(toggled_index) else {
        return current.to_vec();
    };
    if !item.is_enabled() {
        return current.to_vec();
    }

    match selection_mode {
        ControlSelectionMode::SingleRequired => vec![item.id().clone()],
        ControlSelectionMode::SingleAllowNone => {
            if selected_ids_contain(current, item.id()) {
                Vec::new()
            } else {
                vec![item.id().clone()]
            }
        }
        ControlSelectionMode::Multiple => {
            if selected_ids_contain(current, item.id()) {
                current.iter().filter(|selected_id| *selected_id != item.id()).cloned().collect()
            } else {
                let mut next = current.to_vec();
                next.push(item.id().clone());
                normalize_selected_id_list(items, &next, ControlSelectionMode::Multiple)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{
        ControlGroupDirection, ControlGroupModel, ControlGroupStateMode, ControlSelectionMode,
        compute_next_selected_ids, enabled_item_index_by_id, next_enabled_index, normalize_active_id,
        normalize_selected_ids,
    };
    use crate::controls::control_group::{
        ControlGroupFocusStrategy, ControlGroupItem, ControlGroupLayout, default_control_group_template,
    };

    #[test]
    fn normalize_single_required_falls_back_to_first_enabled_item() {
        let mut model = model_with_selected(
            ControlSelectionMode::SingleRequired,
            ControlGroupStateMode::Unmanaged,
            vec!["missing".into()],
        );

        normalize_selected_ids(&mut model);

        assert_eq!(model.default_selected_ids, vec![SharedString::from("compact")]);
    }

    #[test]
    fn normalize_single_allow_none_keeps_empty_selection() {
        let mut model =
            model_with_selected(ControlSelectionMode::SingleAllowNone, ControlGroupStateMode::Unmanaged, Vec::new());

        normalize_selected_ids(&mut model);

        assert!(model.default_selected_ids.is_empty());
    }

    #[test]
    fn normalize_multiple_keeps_all_valid_unique_ids() {
        let mut model = model_with_selected(
            ControlSelectionMode::Multiple,
            ControlGroupStateMode::Unmanaged,
            vec!["compact".into(), "compact".into(), "expanded".into(), "missing".into()],
        );

        normalize_selected_ids(&mut model);

        assert_eq!(model.default_selected_ids, vec![SharedString::from("compact"), SharedString::from("expanded")]);
    }

    #[test]
    fn normalize_active_prefers_selected_then_first_enabled() {
        let mut model = model_with_selected(
            ControlSelectionMode::SingleAllowNone,
            ControlGroupStateMode::Unmanaged,
            vec!["expanded".into()],
        );
        model.active_id = Some("missing".into());

        normalize_active_id(&mut model);

        assert_eq!(model.active_id, Some(SharedString::from("expanded")));
    }

    #[test]
    fn enabled_item_index_ignores_disabled_items() {
        let items = vec![
            ControlGroupItem::new("compact").enabled(false),
            ControlGroupItem::new("comfortable"),
            ControlGroupItem::new("expanded"),
        ];

        assert_eq!(enabled_item_index_by_id(&items, Some(&SharedString::from("compact"))), None);
        assert_eq!(enabled_item_index_by_id(&items, Some(&SharedString::from("comfortable"))), Some(1));
    }

    #[test]
    fn next_enabled_index_skips_disabled_items() {
        let items = vec![
            ControlGroupItem::new("compact"),
            ControlGroupItem::new("comfortable").enabled(false),
            ControlGroupItem::new("expanded"),
        ];

        assert_eq!(next_enabled_index(&items, Some(0), ControlGroupDirection::Next), Some(2));
        assert_eq!(next_enabled_index(&items, Some(2), ControlGroupDirection::Next), Some(0));
        assert_eq!(next_enabled_index(&items, Some(2), ControlGroupDirection::Previous), Some(0));
    }

    #[test]
    fn single_allow_none_toggles_off_selected_item() {
        let items = items();
        let next = compute_next_selected_ids(
            &items,
            &[SharedString::from("comfortable")],
            ControlSelectionMode::SingleAllowNone,
            1,
        );

        assert!(next.is_empty());
    }

    #[test]
    fn single_required_keeps_one_selected_item() {
        let items = items();
        let next = compute_next_selected_ids(
            &items,
            &[SharedString::from("comfortable")],
            ControlSelectionMode::SingleRequired,
            1,
        );

        assert_eq!(next, vec![SharedString::from("comfortable")]);
    }

    #[test]
    fn multiple_toggles_membership() {
        let items = items();
        let next =
            compute_next_selected_ids(&items, &[SharedString::from("compact")], ControlSelectionMode::Multiple, 2);

        assert_eq!(next, vec![SharedString::from("compact"), SharedString::from("expanded")]);

        let toggled_off = compute_next_selected_ids(&items, &next, ControlSelectionMode::Multiple, 0);
        assert_eq!(toggled_off, vec![SharedString::from("expanded")]);
    }

    fn model_with_selected(
        selection_mode: ControlSelectionMode,
        state_mode: ControlGroupStateMode,
        selected_ids: Vec<SharedString>,
    ) -> ControlGroupModel<ControlGroupItem> {
        ControlGroupModel {
            id: "density".into(),
            items: items(),
            default_selected_ids: selected_ids,
            managed_selected_ids: None,
            active_id: None,
            selection_mode,
            state_mode,
            selection_follows_active: false,
            focus_strategy: ControlGroupFocusStrategy::ActiveDescendant,
            focus_target_provider: None,
            enabled: true,
            layout: ControlGroupLayout::default(),
            template: default_control_group_template(),
            item_template: None,
            item_element_template: None,
        }
    }

    fn items() -> Vec<ControlGroupItem> {
        vec![
            ControlGroupItem::new("compact"),
            ControlGroupItem::new("comfortable"),
            ControlGroupItem::new("expanded"),
        ]
    }
}
