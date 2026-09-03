use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Pixels, SharedString, Window, div, prelude::*, px};

use super::control::PopoverButton;

pub type PopoverTrigger<D> = Arc<dyn for<'a> Fn(&PopoverRenderModel<'a, D>, &mut App) -> AnyElement + 'static>;
pub type PopoverContent<D> = Arc<dyn Fn(&PopoverRenderModel<'_, D>, &mut Window, &mut App) -> AnyElement + 'static>;

#[derive(Clone, Debug)]
pub enum PopoverButtonEvent {
    OpenChanged { open: bool },
}

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
    pub(super) id: SharedString,
    pub(super) data: D,
    pub(super) trigger: PopoverTrigger<D>,
    pub(super) content: PopoverContent<D>,
    pub(super) placement: PopoverPlacement,
    pub(super) dismiss_policy: PopoverDismissPolicy,
    pub(super) offset_y: Pixels,
    pub(super) window_margin: Pixels,
    pub(super) initial_content_size: Option<gpui::Size<Pixels>>,
    pub(super) focus_on_open: bool,
    pub(super) animated: bool,
    pub(super) trigger_on_pointer_down: bool,
    pub(super) measure_trigger: bool,
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
            measure_trigger: true,
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
    /// Uses bounds supplied by the host instead of measuring the trigger child.
    pub fn measure_trigger(mut self, value: bool) -> Self {
        self.measure_trigger = value;
        self
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<PopoverButton<D>> {
        cx.new(|cx| PopoverButton::from_builder(self, cx))
    }
}
