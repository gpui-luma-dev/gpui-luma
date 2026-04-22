use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{
    ControlSize, FloatingMenuAppearance, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage,
    ThemeTokens, ThemeUsage, floating_menu::default_floating_menu_appearance,
};

#[derive(Clone, Debug)]
pub struct ContextMenuAppearance {
    pub target_background: Hsla,
    pub target_foreground: Hsla,
    pub target_border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub target_typography: LumaTextStyle,
    pub target_radius: f32,
    pub target_padding_x: f32,
    pub target_padding_y: f32,
    pub target_min_width: f32,
    pub floating_menu: FloatingMenuAppearance,
}

pub trait ContextMenuTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultContextMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_context_menu_theme() -> Arc<dyn ContextMenuTheme> {
    static THEME: OnceLock<Arc<dyn ContextMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultContextMenuTheme::default())).clone()
}

pub const CONTEXT_MENU_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Context Menu",
    parts: &[
        ThemePartUsage {
            part: "target background",
            token: "action.secondary.background",
            states: &["default"],
            appearance_fields: &["ContextMenuAppearance.target_background"],
        },
        ThemePartUsage {
            part: "target hover background",
            token: "action.secondary.hover_background",
            states: &["hovered"],
            appearance_fields: &["ContextMenuAppearance.target_background"],
        },
        ThemePartUsage {
            part: "target pressed background",
            token: "action.secondary.pressed_background",
            states: &["pressed"],
            appearance_fields: &["ContextMenuAppearance.target_background"],
        },
        ThemePartUsage {
            part: "target foreground",
            token: "action.secondary.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &["ContextMenuAppearance.target_foreground"],
        },
        ThemePartUsage {
            part: "target border",
            token: "border.default",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            appearance_fields: &["ContextMenuAppearance.target_border"],
        },
        ThemePartUsage {
            part: "menu background",
            token: "surface.floating.background",
            states: &["open"],
            appearance_fields: &["ContextMenuAppearance.floating_menu.background"],
        },
        ThemePartUsage {
            part: "menu border",
            token: "surface.floating.border",
            states: &["open"],
            appearance_fields: &["ContextMenuAppearance.floating_menu.border"],
        },
        ThemePartUsage {
            part: "item foreground",
            token: "surface.floating.foreground",
            states: &["open"],
            appearance_fields: &["ContextMenuAppearance.floating_menu.foreground"],
        },
        ThemePartUsage {
            part: "item hover background",
            token: "state.hover.background",
            states: &["item hovered"],
            appearance_fields: &["ContextMenuAppearance.floating_menu.item_hover_background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled", "item disabled"],
            appearance_fields: &[
                "ContextMenuAppearance.target_foreground",
                "ContextMenuAppearance.floating_menu.item_disabled_foreground",
            ],
        },
        ThemePartUsage {
            part: "disabled target background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["ContextMenuAppearance.target_background"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["ContextMenuAppearance.focus_ring"],
        },
    ],
};

impl DefaultContextMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ContextMenuTheme for DefaultContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let target_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.action.secondary.pressed_background,
            InteractionLayer::Hovered => palette.action.secondary.hover_background,
            InteractionLayer::Default => palette.action.secondary.background,
        };
        let target_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.action.secondary.foreground
        };

        ContextMenuAppearance {
            target_background,
            target_foreground,
            target_border: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            target_typography: typography.text.label,
            target_radius: metrics.radius(size),
            target_padding_x: metrics.padding_x(size),
            target_padding_y: metrics.padding_y(size),
            target_min_width: 200.0,
            floating_menu: default_floating_menu_appearance(&self.tokens, size),
        }
    }
}
