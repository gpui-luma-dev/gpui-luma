use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::layout::snap_to_pixel;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeViewScale {
    pub row_height: f32,
    pub base_padding_x: f32,
    pub indentation_width: f32,
    pub inner_gap: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl TreeViewScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let base_height = metrics.control_height(size);

        Self {
            row_height: snap_to_pixel(base_height * 0.85, scale_factor),
            base_padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            indentation_width: snap_to_pixel(16.0, scale_factor),
            inner_gap: snap_to_pixel(metrics.gap(size), scale_factor),
            radius: metrics.radius(size),
            // Icon metrics are derived from label typography in the template; these are fallbacks only.
            icon_size: snap_to_pixel(11.0, scale_factor),
            chevron_size: snap_to_pixel(9.0, scale_factor),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeViewPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait TreeViewTheme: Send + Sync {
    fn resolve_row(&self, state: InteractionState, selected: bool, size: ControlSize) -> TreeViewPalette;
    fn metrics(&self) -> MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTreeViewTheme {
    tokens: ThemeTokens,
}

pub fn default_tree_view_theme() -> Arc<dyn TreeViewTheme> {
    static THEME: OnceLock<Arc<dyn TreeViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTreeViewTheme::default())).clone()
}

impl DefaultTreeViewTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TreeViewTheme for DefaultTreeViewTheme {
    fn resolve_row(&self, state: InteractionState, _selected: bool, size: ControlSize) -> TreeViewPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let background = match layer {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        let mut row_typography = typography.text.label;
        crate::controls::textfield::apply_control_size_typography(&mut row_typography, typography, size);

        TreeViewPalette {
            background,
            foreground,
            icon_color: palette.navigation.muted_foreground,
            chevron_color: palette.navigation.muted_foreground,
            typography: row_typography,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
