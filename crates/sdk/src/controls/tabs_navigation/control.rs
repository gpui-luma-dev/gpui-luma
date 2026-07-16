use gpui::{App, Context, Entity, EventEmitter, Focusable, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::{TabsNavigationBuilder, TabsNavigationItem};
use crate::controls::control_group::{ControlGroupControl, ControlGroupEvent, ControlGroupLayout, ControlSelectionMode};
use crate::controls::tabs_navigation::model::TabsNavigationWidthMode;
use crate::theme::ControlSize;

#[derive(Clone, Debug)]
pub enum TabsNavigationEvent {
    Activate { tab_id: SharedString, label: SharedString },
}

pub struct TabsNavigation {
    group: Entity<ControlGroupControl<TabsNavigationItem>>,
    items: Vec<TabsNavigationItem>,
    active_id: Option<SharedString>,
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
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
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
        let ControlGroupEvent::Activate { activated_id } = event else {
            return;
        };

        let Some((tab_id, label)) = self
            .items
            .iter()
            .find_map(|item| (item.id() == activated_id).then(|| (item.id().clone(), item.label_text().clone())))
        else {
            return;
        };

        self.active_id = Some(tab_id.clone());
        cx.emit(TabsNavigationEvent::Activate { tab_id, label });
    }
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
