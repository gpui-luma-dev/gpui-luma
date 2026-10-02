use std::{sync::Arc, time::Duration};
use gpui::{SharedString, AnyElement};
use super::{TooltipRenderModel, TooltipTemplate, TooltipTheme, bubble};

/// Preferred side; the opposite side is used when there is insufficient room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    #[default]
    Above,
    Below,
}

/// Escape can suppress help until the target is left, or disable this attachment permanently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipDismissal {
    #[default]
    UntilLeave,
    Permanently,
}

/// Observable presentation lifecycle, independent of the owner's input events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooltipEvent {
    Shown,
    Hidden,
}

/// Configuration is independent of the target control and its button/slider template.
pub struct Tooltip {
    pub(super) text: SharedString,
    pub(super) shortcut: Option<SharedString>,
    pub(super) delay: Duration,
    pub(super) hide_delay: Duration,
    pub(super) duration: Option<Duration>,
    pub(super) placement: TooltipPlacement,
    pub(super) dismissal: TooltipDismissal,
    pub(super) on_event: Option<Arc<dyn Fn(TooltipEvent) + Send + Sync>>,
    pub(super) theme: Option<Arc<dyn TooltipTheme>>,
    pub(super) once: bool,
    pub(super) enabled: bool,
    pub(super) template: TooltipTemplate,
}

impl Tooltip {
    /// Create ordinary help using the owner's theme and default timing.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            shortcut: None,
            delay: Duration::from_millis(500),
            hide_delay: Duration::from_millis(180),
            duration: None,
            placement: TooltipPlacement::Above,
            dismissal: TooltipDismissal::UntilLeave,
            on_event: None,
            theme: None,
            once: false,
            enabled: true,
            template: Arc::new(bubble),
        }
    }

    /// Optional shortcut or hint rendered alongside the help text.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Hover show delay. Keyboard focus opens immediately.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Grace period after hover/focus leaves the target and bubble.
    pub fn hide_delay(mut self, delay: Duration) -> Self {
        self.hide_delay = delay;
        self
    }

    /// Maximum visible time, starting at actual presentation. None keeps help open
    /// while its trigger remains active. Expiry suppresses reopening until leave.
    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// Preferred side, with automatic flipping and viewport clamping.
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Escape suppression policy. Other dismissal triggers suppress until leave.
    pub fn dismissal(mut self, dismissal: TooltipDismissal) -> Self {
        self.dismissal = dismissal;
        self
    }

    /// Called once per actual presentation and once when that presentation ends.
    pub fn on_event(mut self, callback: impl Fn(TooltipEvent) + Send + Sync + 'static) -> Self {
        self.on_event = Some(Arc::new(callback));
        self
    }

    /// Override the look bound to the owner. Ordinary help uses the owner's look.
    pub fn theme(mut self, theme: Arc<dyn TooltipTheme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Show once per attachment lifetime, counting only an actually rendered bubble.
    pub fn show_once(mut self) -> Self {
        self.once = true;
        self
    }

    /// Disabled configurations do not activate or alter the owner's input behavior.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Presentation override; lifecycle, anchoring, and theme resolution stay in the SDK.
    pub fn template(mut self, template: impl Fn(&TooltipRenderModel) -> AnyElement + Send + Sync + 'static) -> Self {
        self.template = Arc::new(template);
        self
    }
}
