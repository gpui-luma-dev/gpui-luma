use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};

#[derive(Clone, Copy, Debug)]
pub struct TabsNavigationListAppearance {
    pub background: Option<Hsla>,
    pub border: Option<Hsla>,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TabsNavigationItemAppearance {
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
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
            part: "inactive label",
            token: "app.foreground",
            states: &["inactive"],
            appearance_fields: &["TabsNavigationItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "active label and indicator",
            token: "action.prominent.background",
            states: &["active", "active hovered", "active pressed"],
            appearance_fields: &["TabsNavigationItemAppearance.label_color", "TabsNavigationItemAppearance.indicator"],
        },
        ThemePartUsage {
            part: "focus indicator",
            token: "focus.ring",
            states: &["focused", "active focused"],
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
            background: (!enabled).then_some(palette.state.disabled.background),
            border: None,
            radius: metrics.radius(size),
            padding: 0.0,
            gap: metrics.spacing.s5,
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;
        let layer = state.layer();

        let active_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.action.prominent.pressed_background,
            InteractionLayer::Hovered => palette.action.prominent.hover_background,
            InteractionLayer::Default => palette.action.prominent.background,
        };

        TabsNavigationItemAppearance {
            label_color: match (active, state.disabled) {
                (_, true) => palette.state.disabled.foreground,
                (true, false) => active_color,
                (false, false) => palette.app.foreground,
            },
            indicator: active.then_some(if state.focused {
                palette.focus.ring
            } else {
                active_color
            }),
            label_typography: typography.text.label,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            height: typography.text.label.line_height + metrics.spacing.s2,
            indicator_height: 2.0,
        }
    }
}
