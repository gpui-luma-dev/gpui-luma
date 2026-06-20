use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

#[derive(Clone, Debug)]
pub struct PopupMenuPalette {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub trigger_typography: LumaTextStyle,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Debug)]
pub struct PopupMenuLook {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_offset_y: f32,
    pub floating_menu: FloatingMenuLook,
}

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> PopupMenuPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(&self, state: InteractionState, scale: &StandardBoxScale) -> PopupMenuLook {
        compose_popup_menu_appearance(&self.resolve(state), scale)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPopupMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPopupMenuTheme::default())).clone()
}

impl DefaultPopupMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PopupMenuTheme for DefaultPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => palette.app.background,
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        PopupMenuPalette {
            trigger_background,
            trigger_foreground,
            trigger_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            trigger_typography: typography.text.label,
            floating_menu: default_floating_menu_look(&self.tokens, size),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_popup_menu_appearance(palette: &PopupMenuPalette, scale: &StandardBoxScale) -> PopupMenuLook {
    PopupMenuLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_border: palette.trigger_border,
        focus_ring: palette.focus_ring,
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.height * 0.44,
        menu_offset_y: scale.gap * 0.5,
        floating_menu: palette.floating_menu.clone(),
    }
}
