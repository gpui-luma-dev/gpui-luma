use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, MouseButton, Pixels, Render, SharedString, Window, div, prelude::*,
    px,
};

use super::anchored_panel::{AnchoredPanel, AnchoredPanelDismissPolicy, AnchoredPanelPlacement};

pub type PopoverTrigger<D> = Arc<dyn for<'a> Fn(&PopoverRenderModel<'a, D>, &mut App) -> AnyElement + 'static>;
pub type PopoverContent<D> = Arc<dyn Fn(&PopoverRenderModel<'_, D>, &mut Window, &mut App) -> AnyElement + 'static>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPlacement {
    #[default]
    Smart,
    BelowStart,
    BelowCenter,
    AboveStart,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverDismissPolicy {
    #[default]
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
    KeepOpen,
}

pub struct PopoverRenderModel<'a, D> {
    pub id: &'a SharedString,
    pub data: &'a D,
    pub open: bool,
}

pub struct PopoverButtonBuilder<D = ()> {
    id: SharedString,
    data: D,
    trigger: PopoverTrigger<D>,
    content: PopoverContent<D>,
    placement: PopoverPlacement,
    dismiss_policy: PopoverDismissPolicy,
    offset_y: Pixels,
    window_margin: Pixels,
    initial_content_size: Option<gpui::Size<Pixels>>,
    focus_on_open: bool,
    animated: bool,
    trigger_on_pointer_down: bool,
}

impl<D: Default + 'static> PopoverButtonBuilder<D> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            data: D::default(),
            trigger: Arc::new(|_, _| div().into_any_element()),
            content: Arc::new(|_, _, _| div().into_any_element()),
            placement: PopoverPlacement::default(),
            dismiss_policy: PopoverDismissPolicy::default(),
            offset_y: px(0.0),
            window_margin: px(8.0),
            initial_content_size: None,
            focus_on_open: true,
            animated: true,
            trigger_on_pointer_down: true,
        }
    }
    pub fn typed(mut self, data: D) -> Self {
        self.data = data;
        self
    }
    pub fn trigger<F>(mut self, trigger: F) -> Self
    where
        F: for<'a> Fn(&PopoverRenderModel<'a, D>, &mut App) -> AnyElement + 'static,
    {
        self.trigger = Arc::new(trigger);
        self
    }
    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&PopoverRenderModel<'_, D>, &mut Window, &mut App) -> AnyElement + 'static,
    {
        self.content = Arc::new(content);
        self
    }
    pub fn placement(mut self, value: PopoverPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn dismiss_policy(mut self, value: PopoverDismissPolicy) -> Self {
        self.dismiss_policy = value;
        self
    }
    pub fn offset_y(mut self, value: impl Into<Pixels>) -> Self {
        self.offset_y = value.into();
        self
    }
    pub fn window_margin(mut self, value: impl Into<Pixels>) -> Self {
        self.window_margin = value.into();
        self
    }
    pub fn initial_content_size(mut self, value: gpui::Size<Pixels>) -> Self {
        self.initial_content_size = Some(value);
        self
    }
    pub fn focus_on_open(mut self, value: bool) -> Self {
        self.focus_on_open = value;
        self
    }
    pub fn animated(mut self, value: bool) -> Self {
        self.animated = value;
        self
    }
    pub fn trigger_on_pointer_down(mut self, value: bool) -> Self {
        self.trigger_on_pointer_down = value;
        self
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<PopoverButton<D>> {
        cx.new(|cx| PopoverButton::from_builder(self, cx))
    }
}

pub struct PopoverButton<D = ()> {
    id: SharedString,
    data: Arc<D>,
    trigger: PopoverTrigger<D>,
    panel: Entity<AnchoredPanel>,
    trigger_on_pointer_down: bool,
}

impl<D: Default + 'static> PopoverButton<D> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopoverButtonBuilder<D> {
        PopoverButtonBuilder::new(id)
    }
    fn from_builder(builder: PopoverButtonBuilder<D>, cx: &mut Context<Self>) -> Self {
        let id = builder.id.clone();
        let data = Arc::new(builder.data);
        let content = builder.content;
        let content_id = id.clone();
        let content_data_for_panel = data.clone();
        let panel = AnchoredPanel::new(format!("{id}-panel"))
            .content(move |_, window, cx| {
                let model = PopoverRenderModel { id: &content_id, data: &*content_data_for_panel, open: true };
                (content)(&model, window, cx)
            })
            .placement(match builder.placement {
                PopoverPlacement::Smart => AnchoredPanelPlacement::SmartStart,
                PopoverPlacement::BelowStart => AnchoredPanelPlacement::BelowStart,
                PopoverPlacement::BelowCenter => AnchoredPanelPlacement::BelowCenter,
                PopoverPlacement::AboveStart => AnchoredPanelPlacement::AboveStart,
            })
            .dismiss_policy(match builder.dismiss_policy {
                PopoverDismissPolicy::CloseOnClickAway => AnchoredPanelDismissPolicy::CloseOnClickAway,
                PopoverDismissPolicy::CloseOnFocusLoss => AnchoredPanelDismissPolicy::CloseOnFocusLoss,
                PopoverDismissPolicy::CloseOnClickAwayOrFocusLoss => {
                    AnchoredPanelDismissPolicy::CloseOnClickAwayOrFocusLoss
                }
                PopoverDismissPolicy::KeepOpen => AnchoredPanelDismissPolicy::KeepOpen,
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
        Self { id, data, trigger: builder.trigger, panel, trigger_on_pointer_down: builder.trigger_on_pointer_down }
    }
    pub fn is_open(&self, cx: &App) -> bool {
        self.panel.read(cx).is_open()
    }
    pub fn toggle_guarded(&mut self, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.toggle_guarded(cx));
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
                .id(format!("{}-trigger", self.id))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    panel_for_trigger.update(cx, |panel, cx| panel.toggle_guarded(cx));
                })
                .child(trigger)
        } else {
            div().id(format!("{}-trigger", self.id)).child(trigger)
        };
        let mut root = div()
            .relative()
            .on_children_prepainted(move |bounds, _, cx| {
                if let Some(bounds) = bounds.first() {
                    panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(*bounds, cx));
                }
            })
            .child(trigger);
        root = root.child(self.panel.clone());
        root
    }
}
