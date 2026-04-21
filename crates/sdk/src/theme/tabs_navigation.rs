use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, InteractionLayer, InteractionState, ThemeTokens};

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

impl DefaultTabsNavigationTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TabsNavigationTheme for DefaultTabsNavigationTheme {
    fn resolve_list(&self, enabled: bool) -> TabsNavigationListAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;

        TabsNavigationListAppearance {
            background: if enabled {
                colors.surface
            } else {
                colors.surface_disabled
            },
            border: colors.border,
            radius: metrics.radius(size),
            padding: 3.0,
            gap: 2.0,
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;
        let size = ControlSize::Md;
        let layer = state.layer();

        let background = match (active, layer) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(colors.selected_pressed),
            (true, InteractionLayer::Hovered) => Some(colors.selected_hover),
            (true, InteractionLayer::Default) => Some(colors.selected),
            (false, InteractionLayer::Pressed) => Some(colors.surface_pressed),
            (false, InteractionLayer::Hovered) => Some(colors.surface_hover),
            (false, InteractionLayer::Default) => None,
        };

        TabsNavigationItemAppearance {
            background,
            label_color: match (active, state.disabled) {
                (_, true) => colors.text_disabled,
                (true, false) => colors.text_inverse,
                (false, false) => colors.text,
            },
            indicator: active.then_some(if state.pressed {
                colors.text_inverse
            } else {
                colors.focus_ring
            }),
            focus_ring: state.focused.then_some(colors.focus_ring),
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            height: metrics.control_height(size),
            indicator_height: 2.0,
        }
    }
}
