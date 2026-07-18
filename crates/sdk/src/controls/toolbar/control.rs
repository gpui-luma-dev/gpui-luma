use gpui::{App, Context, Entity, Focusable, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::model::{ToolbarBuilder, ToolbarItem};
use super::theme::ToolbarVariant;
use crate::controls::control_group::{
    ControlGroupArrowPolicy, ControlGroupControl, ControlGroupFocusTarget, ControlGroupLayout, ControlSelectionMode,
};
use crate::theme::ControlSize;

pub struct Toolbar {
    group: Entity<ControlGroupControl<ToolbarItem>>,
    items: Vec<ToolbarItem>,
    size: ControlSize,
    variant: ToolbarVariant,
    template: std::sync::Arc<dyn super::ToolbarTemplate>,
}

impl Toolbar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ToolbarBuilder {
        ToolbarBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ToolbarBuilder, cx: &mut Context<Self>) -> Self {
        let model = builder.model.clone();
        let group = ControlGroupControl::new(model.id)
            .items(model.items.clone())
            .selection_mode(ControlSelectionMode::SingleAllowNone)
            .layout(ControlGroupLayout::Horizontal)
            .roving_item_focus()
            .with_focus_target_provider(|item, _window, _cx| {
                item.focus_handle
                    .clone()
                    .map(|focus_handle| ControlGroupFocusTarget { focus_handle, arrow_policy: item.arrow_policy })
            })
            .enabled(model.enabled)
            .template(builder.control_group_template())
            .spawn(cx);

        Self { group, items: model.items, size: model.size, variant: model.variant, template: model.template }
    }

    pub fn active_id(&self, cx: &App) -> Option<SharedString> {
        self.group.read(cx).active_id().cloned()
    }

    pub fn set_items(&mut self, items: impl IntoIterator<Item = ToolbarItem>, cx: &mut Context<Self>) {
        self.items = items.into_iter().collect();
        self.group.update(cx, |group, cx| {
            group.set_items(self.items.clone(), cx);
        });
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.group.update(cx, |group, cx| {
            group.set_enabled(enabled, cx);
        });
    }

    pub fn set_variant(&mut self, variant: ToolbarVariant, cx: &mut Context<Self>) {
        if self.variant == variant {
            return;
        }
        self.variant = variant;
        self.sync_group_template(cx);
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.sync_group_template(cx);
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::ToolbarTemplate>, cx: &mut Context<Self>) {
        self.template = template;
        self.sync_group_template(cx);
    }

    fn sync_group_template(&mut self, cx: &mut Context<Self>) {
        let template = super::template::toolbar_control_group_template(self.size, self.variant, self.template.clone());
        self.group.update(cx, |group, cx| {
            group.set_template(template, cx);
        });
    }
}

impl Focusable for Toolbar {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.group.read(cx).focus_handle(cx)
    }
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.group.clone()).into_any_element()
    }
}

/// Convenience helper for hosted controls that need caret / horizontal arrow ownership.
pub fn horizontal_arrow_policy() -> ControlGroupArrowPolicy {
    ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused
}
