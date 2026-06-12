use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::AdornerSpec;
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, ListRowScale, LumaTextStyle, MetricTokens, ThemeTokens,
};

const ROW_HOVER_ACCENT_ALPHA: f32 = 0.4;

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
pub struct ListViewRowPalette {
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
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
    pub label_baseline_shift: f32,
}

pub trait ListViewTheme: Send + Sync {
    fn resolve_appearance(&self, enabled: bool, focused: bool, size: ControlSize) -> ListViewAppearance;
    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> ListViewRowPalette;
    fn metrics(&self) -> MetricTokens;

    fn resolve_row_appearance(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &ListRowScale,
    ) -> ListViewRowAppearance {
        compose_list_view_row_appearance(&self.resolve_row(selected, state, size), scale)
    }
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

    fn resolve_row(&self, selected: bool, state: InteractionState, _size: ControlSize) -> ListViewRowPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };
        let layer = state.layer();

        let selected_background = palette.surface.subtle.background;
        let hover_background = Hsla { a: ROW_HOVER_ACCENT_ALPHA, ..palette.data.accent_1 };

        let background = if state.disabled && !state.focused {
            transparent
        } else if selected || state.focused {
            selected_background
        } else {
            match layer {
                InteractionLayer::Disabled => transparent,
                InteractionLayer::Pressed => palette.state.pressed.background,
                InteractionLayer::Hovered => hover_background,
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

        ListViewRowPalette {
            background,
            label_color,
            divider: palette.form.input.border,
            adorner: None,
            label_typography: typography.text.label,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_list_view_row_appearance(
    palette: &ListViewRowPalette,
    scale: &ListRowScale,
) -> ListViewRowAppearance {
    ListViewRowAppearance {
        background: palette.background,
        label_color: palette.label_color,
        divider: palette.divider,
        adorner: palette.adorner,
        label_typography: palette.label_typography,
        radius: 0.0,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        min_height: scale.min_height,
        label_baseline_shift: scale.label_baseline_shift,
    }
}
