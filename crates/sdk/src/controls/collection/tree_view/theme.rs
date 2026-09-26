use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::scales::snap_to_pixel;
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
            // Keep tree affordances aligned with the shared control-size icon scale while
            // giving the chevron a slightly lighter visual weight than the content icon.
            icon_size: snap_to_pixel(metrics.icon_size(size), scale_factor),
            chevron_size: snap_to_pixel((metrics.icon_size(size) - 2.0).max(0.0), scale_factor),
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
    fn resolve_row(&self, state: InteractionState, selected: bool, size: ControlSize) -> TreeViewPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let selected = selected && !state.disabled;
        let background = match layer {
            _ if selected => Some(palette.navigation.selected_background),
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else if selected {
            palette.navigation.selected_foreground
        } else {
            palette.app.foreground
        };

        let mut row_typography = typography.text.label;
        crate::controls::textfield::apply_control_size_typography(&mut row_typography, typography, size);

        TreeViewPalette {
            background,
            foreground,
            icon_color: if selected {
                foreground
            } else {
                palette.navigation.muted_foreground
            },
            chevron_color: if selected {
                foreground
            } else {
                palette.navigation.muted_foreground
            },
            typography: row_typography,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultTreeViewTheme, TreeViewScale, TreeViewTheme};
    use crate::theme::{ControlSize, InteractionState, MetricTokens, ThemeTokens};

    #[test]
    fn selected_rows_remain_highlighted_without_hover_or_focus() {
        for tokens in [ThemeTokens::light(), ThemeTokens::dark()] {
            let theme = DefaultTreeViewTheme::new(tokens.clone());
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
            ] {
                let selected = theme.resolve_row(state, true, ControlSize::Md);
                assert_eq!(selected.background, Some(tokens.palette.navigation.selected_background));
                assert_eq!(selected.foreground, tokens.palette.navigation.selected_foreground);
                assert_eq!(selected.icon_color, selected.foreground);
                assert_eq!(selected.chevron_color, selected.foreground);
            }
            assert_eq!(theme.resolve_row(InteractionState::default(), false, ControlSize::Md).background, None);
            let disabled = InteractionState { disabled: true, ..Default::default() };
            assert_eq!(theme.resolve_row(disabled, true, ControlSize::Md).background, None);
            assert_eq!(
                theme.resolve_row(disabled, true, ControlSize::Md).foreground,
                tokens.palette.state.disabled.foreground
            );
        }
    }

    #[test]
    fn affordance_sizes_follow_control_size_scale() {
        let metrics = MetricTokens::default();

        assert_eq!(TreeViewScale::compute(ControlSize::Sm, &metrics, 1.0).icon_size, 14.0);
        assert_eq!(TreeViewScale::compute(ControlSize::Sm, &metrics, 1.0).chevron_size, 12.0);
        assert_eq!(TreeViewScale::compute(ControlSize::Md, &metrics, 1.0).icon_size, 16.0);
        assert_eq!(TreeViewScale::compute(ControlSize::Md, &metrics, 1.0).chevron_size, 14.0);
        assert_eq!(TreeViewScale::compute(ControlSize::Lg, &metrics, 1.0).icon_size, 18.0);
        assert_eq!(TreeViewScale::compute(ControlSize::Lg, &metrics, 1.0).chevron_size, 16.0);
    }
}
