//! Look-owned tabs builder. Spawn synthesizes the SDK [`gpui_luma::controls::tabs::Tabs`].

use gpui::{App, Context, Div, Entity, IntoElement, Render, SharedString, Stateful, Window};
use std::collections::HashMap;
use std::time::Duration;
use gpui_luma::controls::tabs::{TabsBuilder, TabsContent, TabsItem, TabsRenderModel, TabsWidthMode};
use gpui_luma::infra::icon::DisclosureIcons;

use crate::look::{Look, resolve_look};
use crate::tabs::{TabsSize, TabsVariant, tabs_template_for};

type TabsModifier = Box<dyn Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of tabs: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Tabs {
    callbacks: gpui_luma::infra::callbacks::EventCallbacks<gpui_luma::controls::tabs::TabsEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: TabsVariant,
    size: TabsSize,
    width_mode: TabsWidthMode,
    items: Vec<TabsItem>,
    active_id: Option<SharedString>,
    reordering: bool,
    enabled: bool,
    animated: bool,
    contents: HashMap<SharedString, TabsContent>,
    fade_duration: Option<Duration>,
    disclosure_icons: Option<DisclosureIcons>,
    modifiers: Vec<TabsModifier>,
}

impl Tabs {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    /// Observe the complete drag lifecycle through the semantic event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui_luma::controls::tabs::TabsDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }
    /// Observe `DragStarted` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::DragStarted { source, keys } = event {
                callback(owner, *source, keys, cx);
            }
        })
    }

    /// Observe `ItemsRemoved` through the existing drag/drop lifecycle.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, gpui::EntityId, &Vec<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::ItemsRemoved { source, target, keys } = event {
                callback(owner, *source, *target, keys, cx);
            }
        })
    }

    /// Observe `ItemsAdded` through the existing drag/drop lifecycle.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::ItemsAdded { source, target, keys, before } = event {
                callback(owner, *source, *target, keys, before, cx);
            }
        })
    }

    /// Observe `ItemsReordered` through the existing drag/drop lifecycle.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::ItemsReordered { target, keys, before } = event {
                callback(owner, *target, keys, before, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::Dropped { source, target, keys, before, changed } = event
            {
                callback(owner, *source, *target, keys, before, *changed, cx);
            }
        })
    }

    /// Observe `DropRejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            &Option<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::DropRejected { source, target, keys, before, error } =
                event
            {
                callback(owner, *source, *target, keys, before, error, cx);
            }
        })
    }

    /// Observe `DragEnded` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &Vec<gpui::SharedString>,
            gpui_luma::infra::drag_drop::DragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let gpui_luma::infra::drag_drop::DragDropEvent::DragEnded { source, keys, reason } = event {
                callback(owner, *source, keys, *reason, cx);
            }
        })
    }

    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui_luma::controls::tabs::TabsEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `TabsEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::Change { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Activate` through the existing event stream.
    pub fn on_activate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::Activate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::Reactivate` through the existing event stream.
    pub fn on_reactivate<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::Reactivate { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    /// Handle `TabsEvent::DropdownRequested` through the existing event stream.
    pub fn on_dropdown_requested<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            &gpui::SharedString,
            &gpui::SharedString,
            &Option<gpui::Bounds<gpui::Pixels>>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::DropdownRequested { tab_id, label, bounds } = event {
                callback(owner, tab_id, label, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemBoundsChanged` through the existing event stream.
    pub fn on_item_bounds_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::Bounds<gpui::Pixels>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::ItemBoundsChanged { tab_id, bounds } = event {
                callback(owner, tab_id, bounds, cx);
            }
        })
    }

    /// Handle `TabsEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TabsEvent::ItemFocused` through the existing event stream.
    pub fn on_item_focused<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &gpui::SharedString, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let gpui_luma::controls::tabs::TabsEvent::ItemFocused { tab_id, label } = event {
                callback(owner, tab_id, label, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            variant: TabsVariant::default(),
            size: TabsSize::default(),
            width_mode: TabsWidthMode::default(),
            items: Vec::new(),
            active_id: None,
            reordering: false,
            enabled: true,
            animated: true,
            contents: HashMap::new(),
            fade_duration: None,
            disclosure_icons: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn line(self) -> Self {
        self.variant(TabsVariant::Line)
    }

    pub fn surface(self) -> Self {
        self.variant(TabsVariant::Surface)
    }

    pub fn size(mut self, size: TabsSize) -> Self {
        self.size = size;
        self
    }

    pub fn width_mode(mut self, width_mode: TabsWidthMode) -> Self {
        self.width_mode = width_mode;
        self
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.items = items.into_iter().collect();
        self.contents.retain(|id, _| self.items.iter().any(|item| item.id() == id));
        self
    }

    /// Add a content-bearing tab backed by an existing view entity.
    pub fn tab<V: Render + 'static>(
        self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        view: Entity<V>,
    ) -> Self {
        self.tab_with(id, label, move |_, _| view.clone())
    }

    /// Add a content-bearing tab with a presenter for the selected screen.
    pub fn tab_with<F, E>(self, id: impl Into<SharedString>, label: impl Into<SharedString>, presenter: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        self.tab_content(TabsItem::new(id).label(label), TabsContent::new(presenter))
    }

    /// Add an item and its content, preserving item options.
    pub fn tab_content(mut self, item: TabsItem, content: TabsContent) -> Self {
        self.contents.insert(item.id().clone(), content);
        self.items.push(item);
        self
    }

    /// Fade incoming content on selection changes; first render stays fully visible.
    pub fn fade_in(mut self, duration: Duration) -> Self {
        self.fade_duration = Some(duration);
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.active_id = Some(active_id.into());
        self
    }

    /// Opt into SDK same-strip tab reordering.
    pub fn reordering(mut self, enabled: bool) -> Self {
        self.reordering = enabled;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.disclosure_icons = Some(icons);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<gpui_luma::controls::tabs::Tabs> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> TabsBuilder {
        let template = tabs_template_for(&look, self.variant);
        let mut builder = gpui_luma::controls::tabs::Tabs::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .width_mode(self.width_mode)
            .enabled(self.enabled)
            .reordering(self.reordering)
            .animated(self.animated)
            .stylesheet(&look.common_stylesheet(), "radix");
        if let Some(duration) = self.fade_duration {
            builder = builder.fade_in(duration);
        }
        for item in self.items {
            if let Some(content) = self.contents.get(item.id()) {
                builder = builder.tab_content(item, content.clone());
            } else {
                builder = builder.item(item);
            }
        }
        if let Some(active_id) = self.active_id {
            builder = builder.active(active_id);
        }
        if let Some(icons) = self.disclosure_icons {
            builder = builder.disclosure_icons(icons);
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ControlSize;

    #[test]
    fn look_edits_apply_to_future_builders_only() {
        use gpui_luma::theme::stylesheet::{CommonStylesheet, MotionSource};
        let look = Look::built_in();
        let existing = Tabs::new("existing").into_sdk_builder(look.clone());
        look.set_common_stylesheet(CommonStylesheet::default());
        assert_eq!(existing.body_motion().duration, Duration::from_millis(300));
        let new = Tabs::new("new").into_sdk_builder(look);
        assert_eq!(new.body_motion().source, MotionSource::SdkFallback);
    }

    #[test]
    fn body_motion_inherits_look_and_respects_overrides() {
        use gpui_luma::theme::stylesheet::MotionSource;
        let look = Look::built_in();
        let inherited = Tabs::new("motion").into_sdk_builder(look.clone()).body_motion();
        assert_eq!(inherited.duration, Duration::from_millis(300));
        assert_eq!(
            inherited.source,
            MotionSource::Look { name: "radix".into(), field: "common.tabs.content.fade_in_ms" }
        );
        let explicit = Tabs::new("motion").fade_in(Duration::ZERO).into_sdk_builder(look.clone()).body_motion();
        assert_eq!(explicit.duration, Duration::ZERO);
        assert_eq!(explicit.source, MotionSource::InstanceOverride);
        let disabled = Tabs::new("motion")
            .fade_in(Duration::from_millis(900))
            .animated(false)
            .into_sdk_builder(look)
            .body_motion();
        assert_eq!(disabled.duration, Duration::ZERO);
        assert_eq!(disabled.source, MotionSource::AnimationDisabled);
    }

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Tabs::new("s").size(TabsSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Tabs::new("s").size(TabsSize::Two).size.control_size(), ControlSize::Md);
    }

    #[test]
    fn items_keep_radix_axes() {
        let tabs = Tabs::new("preview")
            .size(TabsSize::One)
            .active("colors")
            .items([TabsItem::new("colors").label("Colors")]);
        assert_eq!(tabs.size, TabsSize::One);
        assert_eq!(tabs.active_id.as_deref(), Some("colors"));
        assert_eq!(tabs.items.len(), 1);
    }

    #[test]
    fn variant_helpers_set_surface_and_keep_items() {
        let tabs = Tabs::new("preview").surface().size(TabsSize::One).items([TabsItem::new("colors").label("Colors")]);
        assert!(matches!(tabs.variant, TabsVariant::Surface));
        assert_eq!(tabs.size, TabsSize::One);
        assert_eq!(tabs.items.len(), 1);

        let line = Tabs::new("preview").surface().line();
        assert!(matches!(line.variant, TabsVariant::Line));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Tabs::new("ok")
            .look(&look)
            .items([TabsItem::new("one").label("One")])
            .active("one")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
