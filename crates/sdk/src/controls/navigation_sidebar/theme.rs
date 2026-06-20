use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarContainerLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarSectionLook {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarItemLook {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub icon_size: f32,
}

pub trait NavigationSidebarTheme: Send + Sync {
    fn resolve_container(&self) -> NavigationSidebarContainerLook;
    fn resolve_section(&self) -> NavigationSidebarSectionLook;
    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemLook;
    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> NavigationSidebarItemLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultNavigationSidebarTheme {
    tokens: ThemeTokens,
}

pub fn default_navigation_sidebar_theme() -> Arc<dyn NavigationSidebarTheme> {
    static THEME: OnceLock<Arc<dyn NavigationSidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultNavigationSidebarTheme::default())).clone()
}

impl DefaultNavigationSidebarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }

    fn base_item(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size_metrics = metrics.for_size(size);
        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.navigation.foreground
        };

        NavigationSidebarItemLook {
            background: None,
            foreground,
            icon_color: foreground,
            focus_ring: state.focused.then_some(palette.focus.ring),
            typography: typography.text.label,
            radius: metrics.radius(size),
            height: 30.0,
            padding_x: 8.0,
            gap: size_metrics.gap,
            icon_size: 16.0,
        }
    }
}

impl NavigationSidebarTheme for DefaultNavigationSidebarTheme {
    fn resolve_container(&self) -> NavigationSidebarContainerLook {
        let navigation = &self.tokens.palette.navigation;

        NavigationSidebarContainerLook {
            background: navigation.background,
            foreground: navigation.foreground,
            border: navigation.border,
        }
    }

    fn resolve_section(&self) -> NavigationSidebarSectionLook {
        NavigationSidebarSectionLook {
            label_color: self.tokens.palette.navigation.muted_foreground,
            typography: self.tokens.typography.text.caption,
            height: 20.0,
        }
    }

    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemLook {
        let mut appearance = self.base_item(state, size);
        let palette = &self.tokens.palette;

        appearance.background = match state.layer() {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        appearance
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> NavigationSidebarItemLook {
        let mut appearance = self.base_item(state, size);
        let palette = &self.tokens.palette;

        appearance.background = match (selected, state.layer()) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (true, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (true, InteractionLayer::Default) => Some(palette.navigation.selected_background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (false, InteractionLayer::Default) => None,
        };

        if selected && !state.disabled {
            appearance.foreground = palette.navigation.selected_foreground;
            appearance.icon_color = palette.navigation.selected_foreground;
        }

        appearance
    }
}
