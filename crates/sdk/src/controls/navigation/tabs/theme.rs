use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct TabsListLook {
    pub background: Option<Hsla>,
    pub border: Option<Hsla>,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TabsItemLook {
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub height: f32,
    pub indicator_height: f32,
}

pub trait TabsTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook;
    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook;
    fn font_family(&self) -> SharedString;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTabsTheme {
    tokens: ThemeTokens,
}

pub fn default_tabs_theme() -> Arc<dyn TabsTheme> {
    static THEME: OnceLock<Arc<dyn TabsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTabsTheme::default())).clone()
}

impl DefaultTabsTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TabsTheme for DefaultTabsTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        TabsListLook {
            background: (!enabled).then_some(palette.state.disabled.background),
            border: None,
            radius: metrics.radius(size),
            padding: 0.0,
            gap: metrics.gap(size),
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let label_typography = match size {
            ControlSize::Sm => typography.text.caption,
            ControlSize::Md => typography.text.label,
            ControlSize::Lg => typography.text.body,
        };

        let active_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.selected.background,
            InteractionLayer::Default => palette.state.selected.background,
        };

        TabsItemLook {
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
            label_typography,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            height: label_typography.line_height + metrics.padding_y(size) * 2.0,
            indicator_height: 2.0,
        }
    }

    fn font_family(&self) -> SharedString {
        self.tokens.typography.font.sans.family.clone().into()
    }
}
