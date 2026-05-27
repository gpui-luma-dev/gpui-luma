use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuAppearance, default_floating_menu_appearance};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

#[derive(Clone, Debug)]
pub struct PopupMenuAppearance {
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
    pub floating_menu: FloatingMenuAppearance,
}

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPopupMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.popup_menu_theme();
    }

    if let Some(live) = crate::theme::pack::active_live_theme() {
        return live;
    }
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPopupMenuTheme::default())).clone()
}

pub const POPUP_MENU_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Popup Menu",
    parts: &[
        ThemePartUsage {
            part: "trigger background",
            token: "action.ghost.background",
            states: &["default"],
            appearance_fields: &["PopupMenuAppearance.trigger_background"],
        },
        ThemePartUsage {
            part: "trigger hover background",
            token: "action.ghost.hover_background",
            states: &["hovered"],
            appearance_fields: &["PopupMenuAppearance.trigger_background"],
        },
        ThemePartUsage {
            part: "trigger pressed background",
            token: "action.ghost.pressed_background",
            states: &["pressed"],
            appearance_fields: &["PopupMenuAppearance.trigger_background"],
        },
        ThemePartUsage {
            part: "trigger foreground",
            token: "action.ghost.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["PopupMenuAppearance.trigger_foreground"],
        },
        ThemePartUsage {
            part: "trigger border",
            token: "border.default",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["PopupMenuAppearance.trigger_border"],
        },
        ThemePartUsage {
            part: "menu background",
            token: "surface.floating.background",
            states: &["open"],
            appearance_fields: &["PopupMenuAppearance.floating_menu.background"],
        },
        ThemePartUsage {
            part: "menu border",
            token: "surface.floating.border",
            states: &["open"],
            appearance_fields: &["PopupMenuAppearance.floating_menu.border"],
        },
        ThemePartUsage {
            part: "item foreground",
            token: "surface.floating.foreground",
            states: &["open"],
            appearance_fields: &["PopupMenuAppearance.floating_menu.foreground"],
        },
        ThemePartUsage {
            part: "item hover background",
            token: "state.hover.background",
            states: &["item hovered"],
            appearance_fields: &["PopupMenuAppearance.floating_menu.item_hover_background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled", "item disabled"],
            appearance_fields: &[
                "PopupMenuAppearance.trigger_foreground",
                "PopupMenuAppearance.floating_menu.item_disabled_foreground",
            ],
        },
        ThemePartUsage {
            part: "disabled trigger background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["PopupMenuAppearance.trigger_background"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["PopupMenuAppearance.focus_ring"],
        },
    ],
};

impl DefaultPopupMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PopupMenuTheme for DefaultPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.action.ghost.pressed_background,
            InteractionLayer::Hovered => palette.action.ghost.hover_background,
            InteractionLayer::Default => palette.action.ghost.background,
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.action.ghost.foreground
        };

        PopupMenuAppearance {
            trigger_background,
            trigger_foreground,
            trigger_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            trigger_typography: typography.text.label,
            trigger_radius: metrics.radius(size),
            trigger_padding_x: metrics.padding_x(size),
            trigger_padding_y: metrics.padding_y(size),
            trigger_gap: metrics.gap(size),
            trigger_height: metrics.control_height(size),
            trigger_icon_size: metrics.control_height(size) * 0.44,
            menu_offset_y: metrics.gap(size) * 0.5,
            floating_menu: default_floating_menu_appearance(&self.tokens, size),
        }
    }
}
