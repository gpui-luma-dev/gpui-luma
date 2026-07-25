use gpui::{
    App, Bounds, Context, Entity, EventEmitter, Focusable, IntoElement, Pixels, Render, SharedString, Window, div,
    prelude::*,
};

use super::{TabsNavigationBuilder, TabsNavigationItem};
use crate::controls::control_group::{ControlGroupControl, ControlGroupEvent, ControlGroupLayout, ControlSelectionMode};
use crate::controls::tabs_navigation::model::TabsNavigationWidthMode;
use crate::theme::ControlSize;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TabsNavigationEvent {
    Change { tab_id: SharedString, label: SharedString },
    Activate { tab_id: SharedString, label: SharedString },
    Reactivate { tab_id: SharedString, label: SharedString },
    DropdownRequested { tab_id: SharedString, label: SharedString, bounds: Option<Bounds<Pixels>> },
    ItemBoundsChanged { tab_id: SharedString, bounds: Bounds<Pixels> },
    FocusChanged { focused: bool },
    ItemFocused { tab_id: SharedString, label: SharedString },
}

pub struct TabsNavigation {
    group: Entity<ControlGroupControl<TabsNavigationItem>>,
    items: Vec<TabsNavigationItem>,
    active_id: Option<SharedString>,
    pending_changed_id: Option<SharedString>,
    item_bounds: Vec<Option<Bounds<Pixels>>>,
    size: ControlSize,
    width_mode: TabsNavigationWidthMode,
    template: std::sync::Arc<dyn super::TabsNavigationTemplate>,
}

impl EventEmitter<TabsNavigationEvent> for TabsNavigation {}

impl TabsNavigation {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TabsNavigationBuilder {
        TabsNavigationBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TabsNavigationBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model.clone();
        let mut group_builder = ControlGroupControl::new(model.id)
            .items(model.items)
            .selection_mode(ControlSelectionMode::SingleRequired)
            .layout(ControlGroupLayout::Horizontal)
            .selection_follows_active(true)
            .enabled(model.enabled)
            .template(builder.control_group_template());

        if let Some(active_id) = model.active_id {
            group_builder = group_builder.selected(active_id);
        }

        let group = group_builder.spawn(cx);
        cx.subscribe(&group, |this, _, event: &ControlGroupEvent, cx| {
            this.handle_group_event(event, cx);
        })
        .detach();

        let active_id = group.read(cx).selected_id().cloned();
        Self {
            group,
            items: builder.model.items,
            active_id,
            pending_changed_id: None,
            item_bounds: Vec::new(),
            size: builder.model.size,
            width_mode: builder.model.width_mode,
            template: builder.model.template,
        }
    }

    pub fn active_id(&self) -> Option<&SharedString> {
        self.active_id.as_ref()
    }

    pub fn set_active(&mut self, active_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let active_id = active_id.into();
        if !self.items.iter().any(|item| item.enabled && item.id() == &active_id) {
            return;
        }

        self.active_id = Some(active_id.clone());
        self.pending_changed_id = None;
        self.group.update(cx, |group, cx| {
            group.set_selected_ids([active_id], cx);
        });
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = TabsNavigationItem>, cx: &mut Context<Self>) {
        self.items = items.into_iter().collect();
        if !self
            .active_id
            .as_ref()
            .is_some_and(|active_id| self.items.iter().any(|item| item.enabled && item.id() == active_id))
        {
            self.active_id = self.items.iter().find(|item| item.enabled).map(|item| item.id().clone());
        }
        self.pending_changed_id = None;
        self.item_bounds.clear();
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
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
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
        cx.notify();
        true
    }

    pub fn set_template(
        &mut self,
        template: std::sync::Arc<dyn super::TabsNavigationTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.template = template;
        let size = self.size;
        let width_mode = self.width_mode;
        let template = self.template.clone();
        self.group.update(cx, |group, cx| {
            group.set_template(super::template::tabs_navigation_control_group_template(size, width_mode, template), cx);
        });
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.size == size {
            return;
        }
        self.size = size;
        let width_mode = self.width_mode;
        let template = self.template.clone();
        self.group.update(cx, |group, cx| {
            group.set_template(super::template::tabs_navigation_control_group_template(size, width_mode, template), cx);
        });
    }

    pub fn set_width_mode(&mut self, width_mode: TabsNavigationWidthMode, cx: &mut Context<Self>) {
        if self.width_mode == width_mode {
            return;
        }
        self.width_mode = width_mode;
        let size = self.size;
        let template = self.template.clone();
        self.group.update(cx, |group, cx| {
            group.set_template(super::template::tabs_navigation_control_group_template(size, width_mode, template), cx);
        });
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| {
            group.set_enabled(enabled, cx);
        });
    }

    fn handle_group_event(&mut self, event: &ControlGroupEvent, cx: &mut Context<Self>) {
        let events = translate_group_event(
            &self.items,
            &mut self.active_id,
            &mut self.pending_changed_id,
            &mut self.item_bounds,
            event,
        );
        for event in events {
            cx.emit(event);
        }
    }
}

fn translate_group_event(
    items: &[TabsNavigationItem],
    active_id: &mut Option<SharedString>,
    pending_changed_id: &mut Option<SharedString>,
    item_bounds: &mut Vec<Option<Bounds<Pixels>>>,
    event: &ControlGroupEvent,
) -> Vec<TabsNavigationEvent> {
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
            vec![TabsNavigationEvent::Change { tab_id, label }]
        }
        ControlGroupEvent::Activate { activated_id } => {
            let Some((tab_id, label)) = item_event_payload(items, activated_id) else {
                return Vec::new();
            };
            let selected_changed = pending_changed_id.as_ref().is_some_and(|changed_id| changed_id == &tab_id);
            *pending_changed_id = None;

            let mut events = Vec::new();
            if !selected_changed && active_id.as_ref().is_some_and(|active_id| active_id == &tab_id) {
                events.push(TabsNavigationEvent::Reactivate { tab_id: tab_id.clone(), label: label.clone() });
                if item_is_dropdown_trigger(items, &tab_id) {
                    events.push(TabsNavigationEvent::DropdownRequested {
                        bounds: item_bounds_for_id(items, item_bounds, &tab_id),
                        tab_id: tab_id.clone(),
                        label: label.clone(),
                    });
                }
            }
            *active_id = Some(tab_id.clone());
            events.push(TabsNavigationEvent::Activate { tab_id, label });
            events
        }
        ControlGroupEvent::ItemBoundsChanged { item_id, bounds } => {
            if item_event_payload(items, item_id).is_none() {
                return Vec::new();
            }
            store_item_bounds(items, item_bounds, item_id, *bounds);
            vec![TabsNavigationEvent::ItemBoundsChanged { tab_id: item_id.clone(), bounds: *bounds }]
        }
        ControlGroupEvent::FocusChanged { focused } => {
            vec![TabsNavigationEvent::FocusChanged { focused: *focused }]
        }
        ControlGroupEvent::ItemFocused { item_id } => {
            let Some((tab_id, label)) = item_event_payload(items, item_id) else {
                return Vec::new();
            };
            vec![TabsNavigationEvent::ItemFocused { tab_id, label }]
        }
    }
}

fn item_event_payload(items: &[TabsNavigationItem], item_id: &SharedString) -> Option<(SharedString, SharedString)> {
    items
        .iter()
        .find_map(|item| (item.id() == item_id).then(|| (item.id().clone(), item.label_text().clone())))
}

fn item_is_dropdown_trigger(items: &[TabsNavigationItem], item_id: &SharedString) -> bool {
    items.iter().any(|item| item.id() == item_id && item.is_dropdown_trigger())
}

fn item_bounds_for_id(
    items: &[TabsNavigationItem],
    item_bounds: &[Option<Bounds<Pixels>>],
    item_id: &SharedString,
) -> Option<Bounds<Pixels>> {
    items
        .iter()
        .position(|item| item.id() == item_id)
        .and_then(|index| item_bounds.get(index).copied().flatten())
}

fn store_item_bounds(
    items: &[TabsNavigationItem],
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

impl Focusable for TabsNavigation {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.group.read(cx).focus_handle(cx)
    }
}

impl Render for TabsNavigation {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.group.clone()).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;
    use crate::controls::tabs_navigation::TabsNavigationItemAccessory;

    fn items() -> Vec<TabsNavigationItem> {
        vec![
            TabsNavigationItem::new("cards").label("Cards"),
            TabsNavigationItem::new("controls").label("Controls").dropdown_trigger(),
            TabsNavigationItem::new("theme").label("Theme"),
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
            TabsNavigationEvent::ItemBoundsChanged { tab_id, bounds: event_bounds }
                if tab_id.as_ref() == "controls" && event_bounds == &bounds
        ));
        assert!(matches!(
            &events[1],
            TabsNavigationEvent::Reactivate { tab_id, label }
                if tab_id.as_ref() == "controls" && label.as_ref() == "Controls"
        ));
        assert!(matches!(
            &events[2],
            TabsNavigationEvent::DropdownRequested { tab_id, label, bounds: event_bounds }
                if tab_id.as_ref() == "controls" && label.as_ref() == "Controls" && event_bounds == &Some(bounds)
        ));
        assert!(matches!(
            &events[3],
            TabsNavigationEvent::Activate { tab_id, label }
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
            TabsNavigationEvent::Change { tab_id, label }
                if tab_id.as_ref() == "cards" && label.as_ref() == "Cards"
        ));
        assert!(matches!(
            &events[1],
            TabsNavigationEvent::Activate { tab_id, label }
                if tab_id.as_ref() == "cards" && label.as_ref() == "Cards"
        ));
        assert!(!events.iter().any(|event| matches!(event, TabsNavigationEvent::DropdownRequested { .. })));
        assert!(!events.iter().any(|event| matches!(event, TabsNavigationEvent::Reactivate { .. })));
    }

    #[test]
    fn disclosure_accessory_tracks_open_state() {
        let mut item = TabsNavigationItem::new("controls").label("Controls").dropdown_trigger();
        assert_eq!(
            item.trailing_accessory_ref().and_then(TabsNavigationItemAccessory::is_disclosure_open),
            Some(false)
        );

        assert!(item.trailing_accessory.as_mut().is_some_and(|accessory| accessory.set_disclosure_open(true)));
        assert!(!item.trailing_accessory.as_mut().is_some_and(|accessory| accessory.set_disclosure_open(true)));
        assert_eq!(item.trailing_accessory_ref().and_then(TabsNavigationItemAccessory::is_disclosure_open), Some(true));
    }
}
