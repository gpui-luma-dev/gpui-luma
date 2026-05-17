use gpui::{ClickEvent, Context, EventEmitter, MouseDownEvent, MouseUpEvent, Window};

use crate::controls::button_family::ButtonInteractionState;
use crate::controls::interaction::ControlInteraction;
use crate::keyhandling::ActivateControl;

/// Semantic command event emitted by command-like controls.
///
/// This is intentionally presentation-agnostic: it models activation behavior only.
#[derive(Clone, Debug)]
pub enum CommandEvent {
    Click,
}

/// Shared behavior core for command-like controls.
///
/// `CommandCore` owns interaction state and input/event semantics, but does not
/// render any UI. Concrete controls (for example text-command and icon-command
/// presentations) can reuse this behavior and provide their own templates.
pub struct CommandCore {
    interaction: ControlInteraction,
}

impl CommandCore {
    /// Create a new command behavior core.
    pub fn new<T>(enabled: bool, cx: &mut Context<T>) -> Self {
        Self { interaction: ControlInteraction::new(enabled, cx) }
    }

    /// Create a new command behavior core with explicit tab-stop participation.
    pub fn new_with_tab_stop<T>(enabled: bool, tab_stop: bool, cx: &mut Context<T>) -> Self {
        Self { interaction: ControlInteraction::new_with_tab_stop(enabled, tab_stop, cx) }
    }

    /// Propagate enabled changes into interaction state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.interaction.set_enabled(enabled);
    }

    /// Compute render-time interaction state for templates/presenters.
    pub fn render_state(&self, enabled: bool, window: &Window) -> ButtonInteractionState {
        self.interaction.render_state(enabled, window)
    }

    /// Access the focus handle used by this command behavior core.
    pub fn focus_handle(&self) -> &gpui::FocusHandle {
        self.interaction.focus_handle()
    }

    /// Emit a semantic click if enabled.
    pub fn activate<T>(&mut self, enabled: bool, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        if !enabled {
            return false;
        }

        cx.emit(CommandEvent::Click);
        true
    }

    /// Handle pointer click activation (ignores keyboard-generated click events).
    pub fn handle_click<T>(&mut self, enabled: bool, event: &ClickEvent, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        if event.is_keyboard() {
            return false;
        }

        self.activate(enabled, cx)
    }

    /// Handle keyboard action activation.
    pub fn handle_activate_control<T>(&mut self, enabled: bool, _event: &ActivateControl, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        self.activate(enabled, cx)
    }

    /// Update hover state. Returns `true` when caller should notify.
    pub fn handle_hover(&mut self, hovered: bool) -> bool {
        self.interaction.handle_hover(hovered)
    }

    /// Update pressed state on mouse down. Returns `true` when caller should notify.
    pub fn handle_mouse_down<T>(
        &mut self,
        enabled: bool,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> bool {
        self.interaction.handle_mouse_down(enabled, window, cx)
    }

    /// Update pressed state on mouse up. Returns `true` when caller should notify.
    pub fn handle_mouse_up(&mut self, _event: &MouseUpEvent) -> bool {
        self.interaction.handle_mouse_up()
    }
}
