use gpui::{
    App, ClickEvent, Context, EventEmitter, Focusable, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Window, div, prelude::*,
};

use super::{
    TabsNavigationBuilder, TabsNavigationItem, TabsNavigationRenderItem, TabsNavigationRenderModel,
    TabsNavigationTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::controls::tabs_navigation::model::TabsNavigationModel;
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};

#[derive(Clone, Debug)]
pub enum TabsNavigationEvent {
    Activate { tab_id: SharedString, label: SharedString },
}

pub struct TabsNavigation {
    model: TabsNavigationModel,
    focus_handle: gpui::FocusHandle,
    hovered_item: Option<usize>,
    pressed_item: Option<usize>,
}

impl EventEmitter<TabsNavigationEvent> for TabsNavigation {}

impl TabsNavigation {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TabsNavigationBuilder {
        TabsNavigationBuilder::new(id)
    }

    pub(crate) fn from_builder(mut builder: TabsNavigationBuilder, cx: &mut Context<Self>) -> Self {
        normalize_active_id(&mut builder.model);
        let enabled = builder.model.enabled;

        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            hovered_item: None,
            pressed_item: None,
        }
    }

    pub fn active_id(&self) -> Option<&SharedString> {
        self.model.active_id.as_ref()
    }

    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let active_id = active_id.into();
        if self.can_activate_id(&active_id) {
            self.model.active_id = Some(active_id);
            cx.notify();
        }
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = TabsNavigationItem>, cx: &mut Context<Self>) {
        self.model.items = items.into_iter().collect();
        self.hovered_item = None;
        self.pressed_item = None;
        normalize_active_id(&mut self.model);
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

    fn render_model<'a>(&'a self, window: &Window) -> TabsNavigationRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.enabled;
                let active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == &item.id);

                TabsNavigationRenderItem {
                    id: &item.id,
                    label: &item.label,
                    active,
                    enabled,
                    state: CompositeItemState {
                        hovered: enabled && self.hovered_item == Some(index),
                        pressed: enabled && self.pressed_item == Some(index),
                        disabled: !enabled,
                        selected: active,
                        active: enabled && focus.focused && active,
                        focus_visible: enabled && focus.focus_visible && active,
                    },
                }
            })
            .collect();

        TabsNavigationRenderModel {
            id: &self.model.id,
            items,
            active_id: self.model.active_id.as_ref(),
            enabled: self.model.enabled,
            focus,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> TabsNavigationTemplateHandlers {
        TabsNavigationTemplateHandlers {
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

    fn can_activate_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(|item| item.enabled)
    }

    fn can_activate_id(&self, active_id: &SharedString) -> bool {
        self.model.items.iter().any(|item| item.enabled && &item.id == active_id)
    }

    fn active_index(&self) -> Option<usize> {
        active_enabled_index(&self.model.items, self.model.active_id.as_ref())
    }

    fn activate_index(&mut self, index: usize, emit_if_current: bool, cx: &mut Context<Self>) -> bool {
        if !self.can_activate_item(index) {
            return false;
        }

        let item = &self.model.items[index];
        let already_active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == &item.id);
        if already_active && !emit_if_current {
            return false;
        }

        let tab_id = item.id.clone();
        let label = item.label.clone();
        if !already_active {
            self.model.active_id = Some(tab_id.clone());
            cx.notify();
        }

        cx.emit(TabsNavigationEvent::Activate { tab_id, label });
        true
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_activate_item(index) {
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
        if self.can_activate_item(index) {
            self.pressed_item = Some(index);
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

    fn handle_item_click(&mut self, index: usize, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_index(index, true, cx);
    }

    fn activate_next_enabled(&mut self, direction: TabsNavigationDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if let Some(next_index) = next_enabled_index(&self.model.items, self.active_index(), direction) {
            self.activate_index(next_index, false, cx);
        }
    }

    fn activate_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let next_index = if first {
            first_enabled_index(&self.model.items)
        } else {
            last_enabled_index(&self.model.items)
        };

        if let Some(next_index) = next_index {
            self.activate_index(next_index, false, cx);
        }
    }

    fn handle_select_previous_item(&mut self, _: &SelectPreviousItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_next_enabled(TabsNavigationDirection::Previous, cx);
    }

    fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_next_enabled(TabsNavigationDirection::Next, cx);
    }

    fn handle_select_first_item(&mut self, _: &SelectFirstItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_boundary(true, cx);
    }

    fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.activate_boundary(false, cx);
    }

    fn handle_activate_control(&mut self, _: &ActivateControl, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if let Some(index) = self.active_index() {
            self.activate_index(index, true, cx);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TabsNavigationDirection {
    Previous,
    Next,
}

impl Focusable for TabsNavigation {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TabsNavigation {
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

pub(crate) fn normalize_active_id(model: &mut TabsNavigationModel) {
    if model
        .active_id
        .as_ref()
        .is_some_and(|active_id| model.items.iter().any(|item| item.enabled && &item.id == active_id))
    {
        return;
    }

    model.active_id = model.items.iter().find(|item| item.enabled).map(|item| item.id.clone());
}

pub(crate) fn active_enabled_index(items: &[TabsNavigationItem], active_id: Option<&SharedString>) -> Option<usize> {
    let active_id = active_id?;

    items.iter().position(|item| item.enabled && &item.id == active_id)
}

pub(crate) fn first_enabled_index(items: &[TabsNavigationItem]) -> Option<usize> {
    items.iter().position(|item| item.enabled)
}

pub(crate) fn last_enabled_index(items: &[TabsNavigationItem]) -> Option<usize> {
    items.iter().rposition(|item| item.enabled)
}

pub(crate) fn next_enabled_index(
    items: &[TabsNavigationItem],
    current_index: Option<usize>,
    direction: TabsNavigationDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        TabsNavigationDirection::Previous => len - 1,
        TabsNavigationDirection::Next => 1,
    };
    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == TabsNavigationDirection::Previous => len - 1,
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

#[cfg(test)]
mod tests {
    use super::{
        TabsNavigationDirection, TabsNavigationModel, active_enabled_index, next_enabled_index, normalize_active_id,
    };
    use crate::controls::tabs_navigation::{TabsNavigationItem, default_tabs_navigation_template};

    #[test]
    fn normalize_keeps_valid_active_tab() {
        let mut model = model_with_active(Some("metrics"));

        normalize_active_id(&mut model);

        assert_eq!(model.active_id.as_ref().map(|id| id.to_string()), Some("metrics".to_string()));
    }

    #[test]
    fn normalize_falls_back_to_first_enabled_tab() {
        let mut model = model_with_active(Some("missing"));

        normalize_active_id(&mut model);

        assert_eq!(model.active_id.as_ref().map(|id| id.to_string()), Some("overview".to_string()));
    }

    #[test]
    fn navigation_skips_disabled_tabs_and_wraps() {
        let items = vec![
            TabsNavigationItem::new("overview"),
            TabsNavigationItem::new("activity").enabled(false),
            TabsNavigationItem::new("settings"),
        ];

        assert_eq!(next_enabled_index(&items, Some(0), TabsNavigationDirection::Next), Some(2));
        assert_eq!(next_enabled_index(&items, Some(2), TabsNavigationDirection::Next), Some(0));
        assert_eq!(next_enabled_index(&items, Some(0), TabsNavigationDirection::Previous), Some(2));
    }

    #[test]
    fn active_index_ignores_disabled_active_tab() {
        let items = vec![TabsNavigationItem::new("overview"), TabsNavigationItem::new("activity").enabled(false)];
        let active = "activity".into();

        assert_eq!(active_enabled_index(&items, Some(&active)), None);
    }

    fn model_with_active(active_id: Option<&str>) -> TabsNavigationModel {
        TabsNavigationModel {
            id: "project-tabs".into(),
            items: vec![
                TabsNavigationItem::new("overview").label("Overview"),
                TabsNavigationItem::new("metrics").label("Metrics"),
                TabsNavigationItem::new("settings").label("Settings"),
            ],
            active_id: active_id.map(|id| id.to_string().into()),
            enabled: true,
            template: default_tabs_navigation_template(),
        }
    }
}
