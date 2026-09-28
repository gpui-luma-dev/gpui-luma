use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct SidebarSectionLook {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SidebarItemLook {
    pub background: Option<Hsla>,
    pub focus_border: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub typography: LumaTextStyle,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub icon_size: f32,
}

pub trait SidebarTheme: Send + Sync {
    fn foreground(&self) -> Hsla;
    fn resolve_section(&self) -> SidebarSectionLook;
    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> SidebarItemLook;
    fn resolve_item(&self, selected: bool, state: InteractionState, size: ControlSize) -> SidebarItemLook;

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSidebarTheme {
    tokens: ThemeTokens,
}

pub fn default_sidebar_theme() -> Arc<dyn SidebarTheme> {
    static THEME: OnceLock<Arc<dyn SidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSidebarTheme::default())).clone()
}

impl DefaultSidebarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }

    fn base_item(&self, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size_metrics = metrics.for_size(size);
        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.navigation.foreground
        };

        SidebarItemLook {
            background: None,
            focus_border: (state.focused && !state.disabled).then_some(palette.focus.ring),
            foreground,
            icon_color: foreground,
            typography: typography.text.label,
            radius: metrics.radius(size),
            height: 30.0,
            padding_x: 8.0,
            gap: size_metrics.gap,
            icon_size: 16.0,
        }
    }
}

impl SidebarTheme for DefaultSidebarTheme {
    fn foreground(&self) -> Hsla {
        self.tokens.palette.navigation.foreground
    }

    fn resolve_section(&self) -> SidebarSectionLook {
        SidebarSectionLook {
            label_color: self.tokens.palette.navigation.muted_foreground,
            typography: self.tokens.typography.text.caption,
            height: 20.0,
        }
    }

    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        let mut look = self.base_item(state, size);
        let palette = &self.tokens.palette;

        look.background = match state.layer() {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        look
    }

    fn resolve_item(&self, selected: bool, state: InteractionState, size: ControlSize) -> SidebarItemLook {
        let mut look = self.base_item(state, size);
        let palette = &self.tokens.palette;

        look.background = match (selected, state.layer()) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (true, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (true, InteractionLayer::Default) => Some(palette.navigation.selected_background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (false, InteractionLayer::Default) => None,
        };

        if selected && !state.disabled {
            look.foreground = palette.navigation.selected_foreground;
            look.icon_color = palette.navigation.selected_foreground;
        }

        look
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
