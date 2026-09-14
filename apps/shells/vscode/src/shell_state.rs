#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShellUiState {
    pub customize_layout_open: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellUiAction {
    ToggleCustomizeLayout,
    CustomizeLayoutOpened,
    CustomizeLayoutDismissed,
}

pub fn reduce_shell_ui_state(mut state: ShellUiState, action: ShellUiAction) -> ShellUiState {
    match action {
        ShellUiAction::ToggleCustomizeLayout => state.customize_layout_open = !state.customize_layout_open,
        ShellUiAction::CustomizeLayoutOpened => state.customize_layout_open = true,
        ShellUiAction::CustomizeLayoutDismissed => state.customize_layout_open = false,
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reducer_tracks_dialog_lifecycle() {
        let state = reduce_shell_ui_state(ShellUiState::default(), ShellUiAction::ToggleCustomizeLayout);
        assert!(state.customize_layout_open);

        let state = reduce_shell_ui_state(state, ShellUiAction::CustomizeLayoutDismissed);
        assert!(!state.customize_layout_open);

        let state = reduce_shell_ui_state(state, ShellUiAction::CustomizeLayoutOpened);
        assert!(state.customize_layout_open);
    }
}
