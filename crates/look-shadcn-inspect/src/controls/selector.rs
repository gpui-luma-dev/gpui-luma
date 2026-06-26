//! Inspect metadata for `selector`.

use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{ShadcnModeTokens, ShadcnTextFieldStyle};

use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};
use super::textfield::inspect_textfield_color_palette;

pub struct SelectorInspectPalette {
    pub trigger_background: gpui_luma_look_shadcn::ResolvedColor,
    pub trigger_foreground: gpui_luma_look_shadcn::ResolvedColor,
    pub trigger_border: gpui_luma_look_shadcn::ResolvedColor,
    pub focus_ring: Option<gpui_luma_look_shadcn::ResolvedColor>,
    pub items_panel: FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct SelectorInspectMetrics {
    pub trigger: crate::controls::button::ButtonInspectMetrics,
    pub items_panel: FloatingMenuInspectMetrics,
}

fn selector_textfield_state(state: InteractionState) -> TextFieldState {
    TextFieldState {
        hovered: state.hovered,
        focused: state.focused,
        focus_visible: state.focused,
        ..TextFieldState::default()
    }
}

pub fn inspect_selector_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> SelectorInspectPalette {
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);
    let trigger = inspect_textfield_color_palette(
        mode,
        theme_mode,
        ShadcnTextFieldStyle::Input,
        selector_textfield_state(state),
        !state.disabled,
    );

    SelectorInspectPalette {
        trigger_background: trigger.background,
        trigger_foreground: trigger.foreground,
        trigger_border: trigger.border,
        focus_ring: trigger.focus_ring,
        items_panel: menu,
    }
}

pub fn inspect_selector_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SelectorInspectMetrics {
    use gpui_luma::controls::button_family::ButtonFamilyRole;

    SelectorInspectMetrics {
        trigger: crate::controls::button::inspect_button_metrics(
            mode,
            theme_mode,
            gpui_luma_look_shadcn::ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        items_panel: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}
