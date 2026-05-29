use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::AdornerSpec;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ListViewAppearance {
    pub background: Hsla,
    pub border: Hsla,
    pub header_background: Hsla,
    pub header_label_color: Hsla,
    pub header_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl ListViewAppearance {
    pub fn inner_radius(&self, border_width: f32) -> f32 {
        (self.radius - border_width).max(0.0)
    }
}

#[derive(Clone, Debug)]
pub struct ListViewRowAppearance {
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub min_height: f32,
}

pub trait ListViewTheme: Send + Sync {
    fn resolve_appearance(&self, enabled: bool, focused: bool, size: ControlSize) -> ListViewAppearance;
    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> ListViewRowAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultListViewTheme {
    tokens: ThemeTokens,
}

pub fn default_list_view_theme() -> Arc<dyn ListViewTheme> {
    static THEME: OnceLock<Arc<dyn ListViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultListViewTheme::default())).clone()
}

impl DefaultListViewTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ListViewTheme for DefaultListViewTheme {
    fn resolve_appearance(&self, enabled: bool, _focused: bool, size: ControlSize) -> ListViewAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ListViewAppearance {
            background: if enabled {
                palette.form.input.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            header_background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            header_label_color: if enabled {
                palette.app.muted_foreground
            } else {
                palette.state.disabled.foreground
            },
            header_typography: self.tokens.typography.text.caption,
            radius: metrics.radius(size),
            padding_x: 0.0,
            padding_y: metrics.padding_y(size) * 0.5,
        }
    }

    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> ListViewRowAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };
        let layer = state.layer();

        let row_highlight = palette.surface.subtle.background;

        let background = if state.disabled {
            transparent
        } else if selected {
            row_highlight
        } else {
            match layer {
                InteractionLayer::Disabled => transparent,
                InteractionLayer::Pressed => palette.state.pressed.background,
                InteractionLayer::Hovered => row_highlight,
                InteractionLayer::Default if state.focused => row_highlight,
                InteractionLayer::Default => transparent,
            }
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else if selected {
            palette.surface.subtle.foreground
        } else {
            palette.app.foreground
        };

        ListViewRowAppearance {
            background,
            label_color,
            divider: palette.form.input.border,
            adorner: None,
            label_typography: typography.text.label,
            radius: 0.0,
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            min_height: metrics.control_height(size),
        }
    }
}
