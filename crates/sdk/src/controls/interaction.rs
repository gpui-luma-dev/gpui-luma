use gpui::{Context, FocusHandle, Window};

use crate::theme::InteractionState;

pub(crate) struct ControlInteraction {
    state: InteractionState,
    focus_handle: FocusHandle,
}

impl ControlInteraction {
    pub fn new<T>(enabled: bool, cx: &mut Context<T>) -> Self {
        Self {
            state: InteractionState {
                disabled: !enabled,
                ..InteractionState::default()
            },
            focus_handle: cx.focus_handle().tab_stop(enabled),
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.state.disabled = !enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);

        if !enabled {
            self.state.hovered = false;
            self.state.pressed = false;
        }
    }

    pub fn render_state(&self, enabled: bool, window: &Window) -> InteractionState {
        InteractionState {
            focused: enabled && self.focus_handle.is_focused(window),
            disabled: !enabled,
            ..self.state
        }
    }

    pub fn handle_hover(&mut self, hovered: bool) -> bool {
        if self.state.hovered == hovered && (hovered || !self.state.pressed) {
            return false;
        }

        self.state.hovered = hovered;
        if !hovered {
            self.state.pressed = false;
        }

        true
    }

    pub fn handle_mouse_down<T>(
        &mut self,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> bool {
        if !enabled {
            return false;
        }

        self.state.pressed = true;
        self.focus_handle.focus(window, cx);
        true
    }

    pub fn handle_mouse_up(&mut self) -> bool {
        if !self.state.pressed {
            return false;
        }

        self.state.pressed = false;
        true
    }
}
