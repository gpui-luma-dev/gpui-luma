use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::SliderThumbSize;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens, ThemeTokens};
use crate::theme::adorner::{AdornerSpec, focus_ring_adorner};

#[derive(Clone, Debug)]
pub struct SliderLook {
    pub track_background: Hsla,
    pub fill_background: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub adorner: Option<AdornerSpec>,
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

pub trait SliderTheme: Send + Sync {
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSliderTheme {
    tokens: ThemeTokens,
}

pub fn default_slider_theme() -> Arc<dyn SliderTheme> {
    static THEME: OnceLock<Arc<dyn SliderTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSliderTheme::default())).clone()
}

impl DefaultSliderTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SliderTheme for DefaultSliderTheme {
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let elevation = &self.tokens.elevation;
        let selected = palette.state.selected;
        let fill_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => selected.background,
            InteractionLayer::Default => selected.background,
        };

        SliderLook {
            track_background: if state.disabled {
                palette.state.disabled.background
            } else {
                palette.surface.subtle.background
            },
            fill_background,
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else {
                selected.background
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            adorner: focus_ring_adorner(state.focused.then_some(palette.focus.ring), metrics),
            width: 260.0,
            height: slider_height(metrics, size),
            track_height: slider_track_height(metrics, size),
            thumb_size: slider_thumb_size(metrics, thumb_size.unwrap_or(size.into())),
            radius: metrics.radius.pill,
        }
    }
}

fn slider_height(metrics: &MetricTokens, size: ControlSize) -> f32 {
    (metrics.control_height(size) - metrics.spacing.s1).max(metrics.control_height(size) * 0.5)
}

fn slider_track_height(metrics: &MetricTokens, size: ControlSize) -> f32 {
    (metrics.gap(size) - metrics.border_width.strong).max(metrics.border_width.strong)
}

pub fn slider_thumb_size(metrics: &MetricTokens, size: SliderThumbSize) -> f32 {
    let control_size = match size {
        SliderThumbSize::Sm => ControlSize::Sm,
        SliderThumbSize::Md => ControlSize::Md,
        SliderThumbSize::Lg => ControlSize::Lg,
    };

    slider_height(metrics, control_size) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slider_geometry_uses_metric_tokens() {
        let theme = DefaultSliderTheme::default();
        let small = theme.resolve(ControlSize::Sm, None, InteractionState::default());
        let medium = theme.resolve(ControlSize::Md, None, InteractionState::default());
        let large = theme.resolve(ControlSize::Lg, None, InteractionState::default());

        assert!(small.height < medium.height);
        assert!(medium.height < large.height);
        assert!(small.track_height < medium.track_height);
        assert!(medium.track_height < large.track_height);
        assert!(small.thumb_size < medium.thumb_size);
        assert!(medium.thumb_size < large.thumb_size);
    }

    #[test]
    fn default_slider_geometry_tracks_custom_metric_tokens() {
        let mut tokens = ThemeTokens::default();
        tokens.metrics.control.md.control_height = 60.0;
        tokens.metrics.control.md.gap = 14.0;
        tokens.metrics.spacing.s1 = 6.0;
        tokens.metrics.border_width.strong = 3.0;
        let theme = DefaultSliderTheme::new(tokens.clone());
        let look = theme.resolve(ControlSize::Md, None, InteractionState::default());

        assert_eq!(look.height, 54.0);
        assert_eq!(look.track_height, 11.0);
        assert_eq!(look.thumb_size, 27.0);
    }
}
