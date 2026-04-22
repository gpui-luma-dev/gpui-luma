use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Copy, Debug)]
pub struct TabsNavigationListAppearance {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TabsNavigationItemAppearance {
    pub background: Option<Hsla>,
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
    pub focus_ring: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub height: f32,
    pub indicator_height: f32,
}

pub trait TabsNavigationTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool) -> TabsNavigationListAppearance;
    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTabsNavigationTheme {
    tokens: ThemeTokens,
}

pub fn default_tabs_navigation_theme() -> Arc<dyn TabsNavigationTheme> {
    static THEME: OnceLock<Arc<dyn TabsNavigationTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTabsNavigationTheme::default())).clone()
}

pub const TABS_NAVIGATION_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Tabs Navigation",
    parts: &[
        ThemePartUsage {
            part: "list background",
            token: "surface.subtle.background",
            states: &["enabled"],
            appearance_fields: &["TabsNavigationListAppearance.background"],
        },
        ThemePartUsage {
            part: "list border",
            token: "border.default",
            states: &["default", "enabled", "disabled"],
            appearance_fields: &["TabsNavigationListAppearance.border"],
        },
        ThemePartUsage {
            part: "inactive item hover background",
            token: "state.hover.background",
            states: &["inactive hovered"],
            appearance_fields: &["TabsNavigationItemAppearance.background"],
        },
        ThemePartUsage {
            part: "inactive item pressed background",
            token: "state.pressed.background",
            states: &["inactive pressed"],
            appearance_fields: &["TabsNavigationItemAppearance.background"],
        },
        ThemePartUsage {
            part: "active item background",
            token: "state.selected.background",
            states: &["active"],
            appearance_fields: &["TabsNavigationItemAppearance.background"],
        },
        ThemePartUsage {
            part: "active item hover background",
            token: "action.primary.hover_background",
            states: &["active hovered"],
            appearance_fields: &["TabsNavigationItemAppearance.background"],
        },
        ThemePartUsage {
            part: "active item pressed background",
            token: "action.primary.pressed_background",
            states: &["active pressed"],
            appearance_fields: &["TabsNavigationItemAppearance.background"],
        },
        ThemePartUsage {
            part: "inactive label",
            token: "app.foreground",
            states: &["inactive"],
            appearance_fields: &["TabsNavigationItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "active label and pressed indicator",
            token: "state.selected.foreground",
            states: &["active", "active pressed"],
            appearance_fields: &["TabsNavigationItemAppearance.label_color", "TabsNavigationItemAppearance.indicator"],
        },
        ThemePartUsage {
            part: "active indicator",
            token: "focus.ring",
            states: &["active"],
            appearance_fields: &["TabsNavigationItemAppearance.indicator"],
        },
        ThemePartUsage {
            part: "disabled list background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["TabsNavigationListAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled label",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["TabsNavigationItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["TabsNavigationItemAppearance.focus_ring"],
        },
    ],
};

impl DefaultTabsNavigationTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TabsNavigationTheme for DefaultTabsNavigationTheme {
    fn resolve_list(&self, enabled: bool) -> TabsNavigationListAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;

        TabsNavigationListAppearance {
            background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            border: palette.border.default,
            radius: metrics.radius(size),
            padding: 3.0,
            gap: 2.0,
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();

        let background = match (active, layer) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.action.primary.pressed_background),
            (true, InteractionLayer::Hovered) => Some(palette.action.primary.hover_background),
            (true, InteractionLayer::Default) => Some(palette.state.selected.background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.state.hover.background),
            (false, InteractionLayer::Default) => None,
        };

        TabsNavigationItemAppearance {
            background,
            label_color: match (active, state.disabled) {
                (_, true) => palette.state.disabled.foreground,
                (true, false) => palette.state.selected.foreground,
                (false, false) => palette.app.foreground,
            },
            indicator: active.then_some(if state.pressed {
                palette.state.selected.foreground
            } else {
                palette.focus.ring
            }),
            focus_ring: state.focused.then_some(palette.focus.ring),
            label_typography: typography.text.label,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            height: metrics.control_height(size),
            indicator_height: 2.0,
        }
    }
}
