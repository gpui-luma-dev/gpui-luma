use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ListViewListAppearance {
    pub background: Hsla,
    pub border: Hsla,
    pub header_label_color: Hsla,
    pub header_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

#[derive(Clone, Debug)]
pub struct ListViewRowAppearance {
    pub background: Hsla,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub min_height: f32,
}

pub trait ListViewTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool, focused: bool, size: ControlSize) -> ListViewListAppearance;
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
    fn resolve_list(&self, enabled: bool, _focused: bool, size: ControlSize) -> ListViewListAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ListViewListAppearance {
            background: if enabled {
                palette.form.input.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            header_label_color: if enabled {
                palette.state.selected.foreground
            } else {
                palette.state.disabled.foreground
            },
            header_typography: self.tokens.typography.text.label,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size) * 0.5,
            padding_y: metrics.padding_y(size) * 0.5,
        }
    }

    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> ListViewRowAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };
        let layer = state.layer();

        let background = if state.disabled {
            transparent
        } else if selected {
            palette.state.selected.background
        } else {
            match layer {
                InteractionLayer::Disabled => transparent,
                InteractionLayer::Pressed => palette.state.pressed.background,
                InteractionLayer::Hovered => palette.state.hover.background,
                InteractionLayer::Default if state.focused => palette.state.hover.background,
                InteractionLayer::Default => transparent,
            }
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else if selected {
            palette.state.selected.foreground
        } else {
            palette.app.foreground
        };

        let adorner = if state.focused {
            Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
                color: palette.focus.ring,
                placement: AdornerPlacement::Inset,
                distance: metrics.border_width.default,
                width: metrics.focus.width,
            }))
        } else {
            None
        };

        ListViewRowAppearance {
            background,
            label_color,
            adorner,
            label_typography: typography.text.label,
            radius: metrics.radius(size) * 0.8,
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            min_height: metrics.control_height(size),
        }
    }
}
