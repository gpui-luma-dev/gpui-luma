use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui::{
    App, Bounds, Context, Div, Entity, EventEmitter, Focusable, IntoElement, Pixels, Render, SharedString, Window, div,
    prelude::*,
};

use super::indicator::TabsIndicatorMotion;
use super::{TabsBody, TabsBuilder, TabsContent, TabsItem, TabsList};
use crate::controls::control_group::{ControlGroupControl, ControlGroupEvent, ControlGroupLayout, ControlSelectionMode};
use crate::controls::tabs::model::TabsWidthMode;
use crate::infra::lock;
use crate::motion::{DisclosureMotion, VisualTransition};
use crate::theme::ControlSize;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TabsEvent {
    Change { tab_id: SharedString, label: SharedString },
    Activate { tab_id: SharedString, label: SharedString },
    Reactivate { tab_id: SharedString, label: SharedString },
    DropdownRequested { tab_id: SharedString, label: SharedString, bounds: Option<Bounds<Pixels>> },
    ItemBoundsChanged { tab_id: SharedString, bounds: Bounds<Pixels> },
    FocusChanged { focused: bool },
    ItemFocused { tab_id: SharedString, label: SharedString },
}

pub struct Tabs {
    pub(super) group: Entity<ControlGroupControl<TabsItem>>,
    items: Vec<TabsItem>,
    pub(super) contents: HashMap<SharedString, TabsContent>,
    pub(super) body_transition: VisualTransition,
    body_motion: crate::theme::stylesheet::ResolvedMotion,
    inherited_motion: crate::theme::stylesheet::ResolvedMotion,
    list_slot: Option<Entity<TabsList>>,
    body_slot: Option<Entity<TabsBody>>,
    pub(super) active_id: Option<SharedString>,
    pending_changed_id: Option<SharedString>,
    item_bounds: Vec<Option<Bounds<Pixels>>>,
    size: ControlSize,
    width_mode: TabsWidthMode,
    animated: bool,
    disclosure_icons: crate::infra::icon::DisclosureIcons,
    indicator_motion: TabsIndicatorMotion,
    template: std::sync::Arc<dyn super::TabsTemplate>,
    disclosure_transitions: HashMap<SharedString, DisclosureMotion>,
    disclosure_progress: Arc<Mutex<HashMap<SharedString, f32>>>,
}

impl EventEmitter<TabsEvent> for Tabs {}

impl Tabs {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TabsBuilder {
        TabsBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TabsBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model.clone();
        let indicator_motion = TabsIndicatorMotion::new(model.animated);
        let disclosure_progress = Arc::new(Mutex::new(
            builder
                .model
                .items
                .iter()
                .filter_map(|item| item.trailing_accessory_ref().map(|_| (item.id().clone(), item.disclosure_progress)))
                .collect(),
        ));
        let mut group_builder = ControlGroupControl::new(model.id)
            .items(model.items)
            .selection_mode(ControlSelectionMode::SingleRequired)
            .layout(ControlGroupLayout::Horizontal)
            .selection_follows_active(true)
            .enabled(model.enabled)
            .template(builder.control_group_template(indicator_motion.clone(), disclosure_progress.clone()));

        if let Some(active_id) = model.active_id {
            group_builder = group_builder.selected(active_id);
        }

        let group = group_builder.spawn(cx);
        cx.subscribe(&group, |this, _, event: &ControlGroupEvent, cx| {
            this.handle_group_event(event, cx);
        })
        .detach();

        let active_id = group.read(cx).selected_id().cloned();
        let disclosure_transitions = builder
            .model
            .items
            .iter()
            .filter_map(|item| {
                item.trailing_accessory_ref().map(|_| {
                    (item.id().clone(), DisclosureMotion::new(item.disclosure_progress, builder.model.animated))
                })
            })
            .collect();
        let body_motion = builder.model.inherited_motion.clone().with_override(builder.model.fade_duration, true);
        Self {
            group,
            items: builder.model.items,
            contents: builder.model.contents,
            body_transition: VisualTransition::new(1.0, body_motion.duration),
            body_motion,
            inherited_motion: builder.model.inherited_motion,
            list_slot: None,
            body_slot: None,
            active_id,
            pending_changed_id: None,
            item_bounds: Vec::new(),
            size: builder.model.size,
            width_mode: builder.model.width_mode,
            animated: builder.model.animated,
            disclosure_icons: builder.model.disclosure_icons.clone(),
            indicator_motion,
            template: builder.model.template,
            disclosure_transitions,
            disclosure_progress,
        }
    }

    pub(crate) fn initialize_slots(&mut self, cx: &mut Context<Self>) {
        let tabs = cx.entity();
        self.list_slot = Some(cx.new(|cx| TabsList::new(&tabs, cx)));
        self.body_slot = Some(cx.new(|cx| TabsBody::new(&tabs, cx)));
    }

    /// Render only navigation, for layouts that place the body elsewhere.
    pub fn tab_list(&self) -> Div {
        div().children(self.list_slot.clone())
    }

    /// Render the selected panel with the configured body transition.
    pub fn body(&self) -> Div {
        div().w_full().min_h_0().flex_1().flex().flex_col().children(self.body_slot.clone())
    }

    /// Effective body motion for future selections, with its source.
    pub fn body_motion(&self) -> crate::theme::stylesheet::ResolvedMotion {
        self.body_motion.clone().with_override(None, self.animated)
    }

    /// Override the body fade for future selections. `None` restores the look
    /// snapshot captured by the builder; explicit zero disables future fades.
    /// A running transition keeps its duration and progress.
    pub fn set_fade_in(&mut self, duration: Option<std::time::Duration>, cx: &mut Context<Self>) {
        self.body_motion = self.inherited_motion.clone().with_override(duration, true);
        cx.notify();
    }

    /// Replace or attach a panel for an existing item. Unknown IDs return false.
    /// Selection and running motion are preserved, including for disabled items.
    pub fn set_content(&mut self, id: impl Into<SharedString>, content: TabsContent, cx: &mut Context<Self>) -> bool {
        let id = id.into();
        if !self.items.iter().any(|item| item.id() == &id) {
            return false;
        }
        let was_navigation_only = self.contents.is_empty();
        self.contents.insert(id, content);
        if was_navigation_only {
            cx.notify();
        }
        self.refresh_content(cx);
        true
    }

    /// Remove a panel without removing its navigation item or changing selection.
    pub fn remove_content(&mut self, id: impl AsRef<str>, cx: &mut Context<Self>) -> bool {
        let requested_id = id.as_ref();
        let Some(id) = self.contents.keys().find(|id| id.as_ref() == requested_id).cloned() else {
            return false;
        };
        self.contents.remove(&id);
        if self.contents.is_empty() {
            cx.notify();
        }
        self.refresh_content(cx);
        true
    }

    /// Reinvoke the selected presenter on the next body render. Does not emit
    /// selection events or restart motion, and invalidates only the body slot.
    pub fn refresh_content(&mut self, cx: &mut Context<Self>) {
        if let Some(body) = &self.body_slot {
            body.update(cx, |_, cx| cx.notify());
        }
    }

    fn restart_body_transition(&mut self) {
        if self.animated && !self.body_motion.duration.is_zero() && !self.contents.is_empty() {
            self.body_transition = VisualTransition::new(0.0, self.body_motion.duration);
            self.body_transition.set_target(1.0);
        } else {
            self.body_transition.snap_to(1.0);
        }
    }

    pub fn active_id(&self) -> Option<&SharedString> {
        self.active_id.as_ref()
    }

    /// Silently select an enabled item. Invalid IDs and the current ID are no-ops.
    /// Observe the entity for programmatic changes; user actions emit `TabsEvent`.
    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let active_id = active_id.into();
        if self.active_id.as_ref() == Some(&active_id)
            || !self.items.iter().any(|item| item.enabled && item.id() == &active_id)
        {
            return;
        }

        let previous_id = self.active_id.clone();
        self.active_id = Some(active_id.clone());
        self.pending_changed_id = None;
        if previous_id.as_ref() != Some(&active_id) {
            self.restart_body_transition();
            self.retarget_indicator_for_active(true);
        }
        self.group.update(cx, |group, cx| {
            group.set_active(active_id.clone(), cx);
            group.set_selected_ids([active_id], cx);
        });
        cx.notify();
    }

    /// Silently replace items, retaining panels for surviving IDs. An invalid or
    /// disabled selection falls back to the first enabled item (or none).
    /// Item updates finish the current body fade.
    pub fn set_items(&mut self, items: impl IntoIterator<Item = TabsItem>, cx: &mut Context<Self>) {
        self.items = items.into_iter().collect();
        self.contents.retain(|id, _| self.items.iter().any(|item| item.id() == id));
        self.body_transition.snap_to(1.0);
        if !self
            .active_id
            .as_ref()
            .is_some_and(|active_id| self.items.iter().any(|item| item.enabled && item.id() == active_id))
        {
            self.active_id = self.items.iter().find(|item| item.enabled).map(|item| item.id().clone());
        }
        self.pending_changed_id = None;
        self.item_bounds.clear();
        {
            let mut progress = lock::mutex(&self.disclosure_progress);
            progress.clear();
            for item in &self.items {
                if item.trailing_accessory_ref().is_some() {
                    progress.insert(item.id().clone(), item.disclosure_progress);
                }
            }
        }
        self.disclosure_transitions.retain(|id, _| self.items.iter().any(|item| item.id() == id));
        for item in &self.items {
            if item.trailing_accessory_ref().is_some() {
                self.disclosure_transitions
                    .entry(item.id().clone())
                    .or_insert_with(|| DisclosureMotion::new(item.disclosure_progress, self.animated));
            }
        }
        self.indicator_motion.clear_geometry();
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
        cx.notify();
    }

    pub fn set_item_disclosure_open(&mut self, item_id: impl AsRef<str>, open: bool, cx: &mut Context<Self>) -> bool {
        let item_id = item_id.as_ref();
        let Some(item) = self.items.iter_mut().find(|item| item.id().as_ref() == item_id) else {
            return false;
        };
        let changed = item.trailing_accessory.as_mut().is_some_and(|accessory| accessory.set_disclosure_open(open));
        if !changed {
            return false;
        }
        if let Some(transition) = self.disclosure_transitions.get_mut(item.id()) {
            transition.set_target(if open { 1.0 } else { 0.0 });
        }
        self.disclosure_progress
            .lock()
            .expect("tabs disclosure progress lock")
            .entry(item.id().clone())
            .or_insert(item.disclosure_progress);
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
        cx.notify();
        true
    }

    pub fn set_animated(&mut self, animated: bool, cx: &mut Context<Self>) {
        if self.animated == animated {
            return;
        }
        self.animated = animated;
        if !animated {
            self.body_transition.snap_to(1.0);
        }
        self.indicator_motion.set_animated(animated);
        for transition in self.disclosure_transitions.values_mut() {
            transition.set_animated(animated);
        }
        self.refresh_group_template(cx);
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::TabsTemplate>, cx: &mut Context<Self>) {
        self.template = template;
        self.refresh_group_template(cx);
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.refresh_group_template(cx);
    }

    pub fn set_width_mode(&mut self, width_mode: TabsWidthMode, cx: &mut Context<Self>) {
        if self.width_mode == width_mode {
            return;
        }
        self.width_mode = width_mode;
        self.refresh_group_template(cx);
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| {
            group.set_enabled(enabled, cx);
        });
    }

    fn refresh_group_template(&mut self, cx: &mut Context<Self>) {
        let size = self.size;
        let width_mode = self.width_mode;
        let template = self.template.clone();
        let indicator_motion = self.indicator_motion.clone();
        let disclosure_icons = self.disclosure_icons.clone();
        let disclosure_progress = self.disclosure_progress.clone();
        self.group.update(cx, |group, cx| {
            group.set_template(
                super::template::tabs_control_group_template(
                    size,
                    width_mode,
                    template,
                    indicator_motion,
                    disclosure_icons,
                    disclosure_progress,
                ),
                cx,
            );
        });
    }

    fn sync_disclosure_transitions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut animating = false;
        for (id, transition) in &mut self.disclosure_transitions {
            animating |= transition.sync();
            if let Some(item) = self.items.iter_mut().find(|item| item.id() == id) {
                item.disclosure_progress = transition.progress();
                self.disclosure_progress
                    .lock()
                    .expect("tabs disclosure progress lock")
                    .insert(id.clone(), transition.progress());
            }
        }
        if animating {
            self.group.update(cx, |_, cx| cx.notify());
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
    }

    fn retarget_indicator_for_active(&mut self, animate: bool) {
        let Some(active_id) = self.active_id.clone() else {
            return;
        };
        let Some(bounds) = item_bounds_for_id(&self.items, &self.item_bounds, &active_id) else {
            return;
        };
        self.indicator_motion.apply_item_bounds(bounds, animate && self.animated);
    }

    pub(super) fn handle_group_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<Self>) {
        let previous_active = self.active_id.clone();
        let events = translate_group_event(
            &self.items,
            &mut self.active_id,
            &mut self.pending_changed_id,
            &mut self.item_bounds,
            event,
        );

        match event {
            ControlGroupEvent::Change { selected: true, .. } => {
                if previous_active != self.active_id {
                    self.restart_body_transition();
                    self.retarget_indicator_for_active(true);
                }
            }
            ControlGroupEvent::ItemBoundsChanged { item_id, .. } if self.active_id.as_ref() == Some(item_id) => {
                let animate = self.indicator_motion.is_animating();
                self.retarget_indicator_for_active(animate);
            }
            _ => {}
        }

        for event in events {
            cx.emit(event);
        }
        cx.notify();
    }
}

fn translate_group_event(
    items: &[TabsItem],
    active_id: &mut Option<SharedString>,
    pending_changed_id: &mut Option<SharedString>,
    item_bounds: &mut Vec<Option<Bounds<Pixels>>>,
    event: &ControlGroupEvent,
) -> Vec<TabsEvent> {
    match event {
        ControlGroupEvent::Change { changed_id, selected, .. } => {
            if !selected {
                return Vec::new();
            }
            let Some((tab_id, label)) = item_event_payload(items, changed_id) else {
                return Vec::new();
            };
            *active_id = Some(tab_id.clone());
            *pending_changed_id = Some(tab_id.clone());
            vec![TabsEvent::Change { tab_id, label }]
        }
        ControlGroupEvent::Activate { activated_id } => {
            let Some((tab_id, label)) = item_event_payload(items, activated_id) else {
                return Vec::new();
            };
            let selected_changed = pending_changed_id.as_ref().is_some_and(|changed_id| changed_id == &tab_id);
            *pending_changed_id = None;

            let mut events = Vec::new();
            if !selected_changed && active_id.as_ref().is_some_and(|active_id| active_id == &tab_id) {
                events.push(TabsEvent::Reactivate { tab_id: tab_id.clone(), label: label.clone() });
                if item_is_dropdown_trigger(items, &tab_id) {
                    events.push(TabsEvent::DropdownRequested {
                        bounds: item_bounds_for_id(items, item_bounds, &tab_id),
                        tab_id: tab_id.clone(),
                        label: label.clone(),
                    });
                }
            }
            *active_id = Some(tab_id.clone());
            events.push(TabsEvent::Activate { tab_id, label });
            events
        }
        ControlGroupEvent::ItemBoundsChanged { item_id, bounds } => {
            if item_event_payload(items, item_id).is_none() {
                return Vec::new();
            }
            store_item_bounds(items, item_bounds, item_id, *bounds);
            vec![TabsEvent::ItemBoundsChanged { tab_id: item_id.clone(), bounds: *bounds }]
        }
        ControlGroupEvent::FocusChanged { focused } => {
            vec![TabsEvent::FocusChanged { focused: *focused }]
        }
        ControlGroupEvent::ItemFocused { item_id } => {
            let Some((tab_id, label)) = item_event_payload(items, item_id) else {
                return Vec::new();
            };
            vec![TabsEvent::ItemFocused { tab_id, label }]
        }
    }
}

fn item_event_payload(items: &[TabsItem], item_id: &SharedString) -> Option<(SharedString, SharedString)> {
    items
        .iter()
        .find_map(|item| (item.id() == item_id).then(|| (item.id().clone(), item.label_text().clone())))
}

fn item_is_dropdown_trigger(items: &[TabsItem], item_id: &SharedString) -> bool {
    items.iter().any(|item| item.id() == item_id && item.is_dropdown_trigger())
}

fn item_bounds_for_id(
    items: &[TabsItem],
    item_bounds: &[Option<Bounds<Pixels>>],
    item_id: &SharedString,
) -> Option<Bounds<Pixels>> {
    items
        .iter()
        .position(|item| item.id() == item_id)
        .and_then(|index| item_bounds.get(index).copied().flatten())
}

fn store_item_bounds(
    items: &[TabsItem],
    item_bounds: &mut Vec<Option<Bounds<Pixels>>>,
    item_id: &SharedString,
    bounds: Bounds<Pixels>,
) {
    let Some(index) = items.iter().position(|item| item.id() == item_id) else {
        return;
    };
    if item_bounds.len() != items.len() {
        item_bounds.resize(items.len(), None);
    }
    item_bounds[index] = Some(bounds);
}

impl Focusable for Tabs {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.group.read(cx).focus_handle(cx)
    }
}

impl Tabs {
    pub(super) fn sync_list_motion(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sync_disclosure_transitions(window, cx);
        let was_animating = self.indicator_motion.is_animating();
        let is_animating = self.indicator_motion.sync();
        self.indicator_motion.schedule_frame(window, cx);
        if was_animating || is_animating {
            self.group.update(cx, |_, cx| cx.notify());
        }
    }
}

impl Render for Tabs {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div();
        if !self.contents.is_empty() {
            root = root.flex().flex_col().w_full();
        }
        if let Some(list) = &self.list_slot {
            root = root.child(list.clone());
        }
        if !self.contents.is_empty()
            && let Some(body) = &self.body_slot
        {
            root = root.child(body.clone());
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::tabs::TabsItemAccessory;

    fn items() -> Vec<TabsItem> {
        vec![
            TabsItem::new("cards").label("Cards"),
            TabsItem::new("controls").label("Controls").dropdown_trigger(),
            TabsItem::new("theme").label("Theme"),
        ]
    }

    #[test]
    fn dropdown_reactivation_emits_reactivate_dropdown_and_activate() {
        let items = items();
        let bounds = Bounds::new(point(px(10.0), px(20.0)), size(px(120.0), px(32.0)));
        let mut active_id = Some("controls".into());
        let mut pending_changed_id = None;
        let mut item_bounds = Vec::new();
        let mut events = translate_group_event(
            &items,
            &mut active_id,
            &mut pending_changed_id,
            &mut item_bounds,
            &ControlGroupEvent::ItemBoundsChanged { item_id: "controls".into(), bounds },
        );
        events.extend(translate_group_event(
            &items,
            &mut active_id,
            &mut pending_changed_id,
            &mut item_bounds,
            &ControlGroupEvent::Activate { activated_id: "controls".into() },
        ));

        assert_eq!(active_id, Some("controls".into()));
        assert_eq!(events.len(), 4);
        assert!(matches!(
            &events[0],
            TabsEvent::ItemBoundsChanged { tab_id, bounds: event_bounds }
                if tab_id.as_ref() == "controls" && event_bounds == &bounds
        ));
        assert!(matches!(
            &events[1],
            TabsEvent::Reactivate { tab_id, label }
                if tab_id.as_ref() == "controls" && label.as_ref() == "Controls"
        ));
        assert!(matches!(
            &events[2],
            TabsEvent::DropdownRequested { tab_id, label, bounds: event_bounds }
                if tab_id.as_ref() == "controls" && label.as_ref() == "Controls" && event_bounds == &Some(bounds)
        ));
        assert!(matches!(
            &events[3],
            TabsEvent::Activate { tab_id, label }
                if tab_id.as_ref() == "controls" && label.as_ref() == "Controls"
        ));
    }

    #[test]
    fn selected_change_activation_does_not_request_dropdown() {
        let items = items();
        let mut active_id = Some("controls".into());
        let mut pending_changed_id = None;
        let mut item_bounds = Vec::new();
        let mut events = translate_group_event(
            &items,
            &mut active_id,
            &mut pending_changed_id,
            &mut item_bounds,
            &ControlGroupEvent::Change {
                changed_id: "cards".into(),
                selected: true,
                selected_ids: vec!["cards".into()],
            },
        );
        events.extend(translate_group_event(
            &items,
            &mut active_id,
            &mut pending_changed_id,
            &mut item_bounds,
            &ControlGroupEvent::Activate { activated_id: "cards".into() },
        ));

        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            TabsEvent::Change { tab_id, label }
                if tab_id.as_ref() == "cards" && label.as_ref() == "Cards"
        ));
        assert!(matches!(
            &events[1],
            TabsEvent::Activate { tab_id, label }
                if tab_id.as_ref() == "cards" && label.as_ref() == "Cards"
        ));
        assert!(!events.iter().any(|event| matches!(event, TabsEvent::DropdownRequested { .. })));
        assert!(!events.iter().any(|event| matches!(event, TabsEvent::Reactivate { .. })));
    }

    #[test]
    fn disclosure_accessory_tracks_open_state() {
        let mut item = TabsItem::new("controls").label("Controls").dropdown_trigger();
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(false));

        assert!(item.trailing_accessory.as_mut().is_some_and(|accessory| accessory.set_disclosure_open(true)));
        assert!(!item.trailing_accessory.as_mut().is_some_and(|accessory| accessory.set_disclosure_open(true)));
        assert_eq!(item.trailing_accessory_ref().and_then(TabsItemAccessory::is_disclosure_open), Some(true));
    }
}
