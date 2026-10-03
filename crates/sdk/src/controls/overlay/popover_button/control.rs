use std::sync::Arc;

use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, IntoElement, MouseButton, Pixels, Render, SharedString, Window,
    div, prelude::*,
};

use super::model::{
    PopoverButtonBuilder, PopoverButtonEvent, PopoverDismissPolicy, PopoverPlacement, PopoverRenderModel,
    PopoverTrigger,
};
use super::panel::{PopoverPanel, PopoverPanelDismissPolicy, PopoverPanelEvent, PopoverPanelPlacement};

pub struct PopoverButton<D = ()> {
    id: SharedString,
    data: Arc<D>,
    trigger: PopoverTrigger<D>,
    panel: Entity<PopoverPanel>,
    trigger_on_pointer_down: bool,
    measure_trigger: bool,
}

impl<D: 'static> EventEmitter<PopoverButtonEvent> for PopoverButton<D> {}

impl<D: Default + 'static> PopoverButton<D> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopoverButtonBuilder<D> {
        PopoverButtonBuilder::new(id)
    }
    pub(super) fn from_builder(builder: PopoverButtonBuilder<D>, cx: &mut Context<Self>) -> Self {
        let id = builder.id.clone();
        let data = Arc::new(builder.data);
        let content = builder.content;
        let content_id = id.clone();
        let content_data_for_panel = data.clone();
        let panel = PopoverPanel::new(format!("{id}-panel"))
            .content(move |_, window, cx| {
                let model = PopoverRenderModel { id: &content_id, data: &*content_data_for_panel, open: true };
                (content)(&model, window, cx)
            })
            .placement(match builder.placement {
                PopoverPlacement::Smart => PopoverPanelPlacement::SmartStart,
                PopoverPlacement::BelowStart => PopoverPanelPlacement::BelowStart,
                PopoverPlacement::BelowCenter => PopoverPanelPlacement::BelowCenter,
                PopoverPlacement::AboveStart => PopoverPanelPlacement::AboveStart,
            })
            .dismiss_policy(match builder.dismiss_policy {
                PopoverDismissPolicy::CloseOnClickAway => PopoverPanelDismissPolicy::CloseOnClickAway,
                PopoverDismissPolicy::CloseOnFocusLoss => PopoverPanelDismissPolicy::CloseOnFocusLoss,
                PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss => {
                    PopoverPanelDismissPolicy::CloseOnClickAwayOrFocusLoss
                }
                PopoverDismissPolicy::KeepOpen => PopoverPanelDismissPolicy::KeepOpen,
            })
            .offset_y(builder.offset_y)
            .window_margin(builder.window_margin)
            .focus_on_open(builder.focus_on_open)
            .animated(builder.animated);
        let panel = if let Some(size) = builder.initial_content_size {
            panel.initial_content_size(size).spawn(cx)
        } else {
            panel.spawn(cx)
        };
        let measure_trigger = builder.measure_trigger;
        cx.subscribe(&panel, move |_popover, _, event: &PopoverPanelEvent, cx| {
            if let PopoverPanelEvent::OpenChanged { open } = event {
                cx.emit(PopoverButtonEvent::OpenChanged { open: *open });
            }
        })
        .detach();
        Self {
            id,
            data,
            trigger: builder.trigger,
            panel,
            trigger_on_pointer_down: builder.trigger_on_pointer_down,
            measure_trigger,
        }
    }
    pub fn is_open(&self, cx: &App) -> bool {
        self.panel.read(cx).is_open()
    }
    pub fn toggle_guarded(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.toggle_guarded(cx));
    }
    pub fn toggle_guarded_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.toggle_guarded_from(opener, cx));
    }
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.dismiss(cx));
    }
    pub fn set_anchor_bounds(&mut self, bounds: gpui::Bounds<Pixels>, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.set_anchor_bounds(bounds, cx));
    }
}

impl<D: 'static> Render for PopoverButton<D> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = PopoverRenderModel { id: &self.id, data: &*self.data, open: self.panel.read(cx).is_open() };
        let trigger = (self.trigger)(&model, cx);
        let panel_for_bounds = self.panel.clone();
        let panel_for_trigger = self.panel.clone();
        let trigger = if self.trigger_on_pointer_down {
            div()
                .id((self.id.clone(), 0usize))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    panel_for_trigger.update(cx, |panel, cx| panel.toggle_guarded(cx));
                })
                .child(trigger)
        } else {
            div().id((self.id.clone(), 0usize)).child(trigger)
        };
        let mut root = div().relative().child(trigger);
        if self.measure_trigger {
            root = root.on_children_prepainted(move |bounds, _, cx| {
                if let Some(bounds) = bounds.first() {
                    panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(*bounds, cx));
                }
            });
        }
        root = root.child(self.panel.clone());
        root
    }
}
