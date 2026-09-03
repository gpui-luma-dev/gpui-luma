//! Inspect metadata for `selector`.

use luma::controls::button_family::ButtonFamilyRole;
use luma::controls::selector::SelectorTriggerStyle;
use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{ResolvedColor, ShadcnButtonStyle, ShadcnModeTokens};

use super::button::{inspect_button_color_palette, inspect_button_metrics};
use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};

pub struct SelectorInspectPalette {
    pub trigger_style: SelectorTriggerStyle,
    pub trigger_background: ResolvedColor,
    pub trigger_foreground: ResolvedColor,
    pub trigger_border: ResolvedColor,
    pub items_panel: FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct SelectorInspectMetrics {
    pub trigger: crate::controls::button::ButtonInspectMetrics,
    pub items_panel: FloatingMenuInspectMetrics,
}

fn button_style(trigger_style: SelectorTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        SelectorTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        SelectorTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn inspect_selector_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    size: ControlSize,
) -> SelectorInspectPalette {
    let menu = crate::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);
    let trigger =
        inspect_button_color_palette(mode, theme_mode, button_style(trigger_style), ButtonFamilyRole::Text, state);

    SelectorInspectPalette {
        trigger_style,
        trigger_background: trigger.background,
        trigger_foreground: trigger.foreground,
        trigger_border: trigger.border,
        items_panel: menu,
    }
}

pub fn inspect_selector_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    size: ControlSize,
) -> SelectorInspectMetrics {
    SelectorInspectMetrics {
        trigger: inspect_button_metrics(
            mode,
            theme_mode,
            button_style(trigger_style),
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        items_panel: crate::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}
