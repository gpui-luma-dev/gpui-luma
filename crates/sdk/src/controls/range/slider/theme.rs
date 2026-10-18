use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use super::SliderThumbSize;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens, ThemeTokens};

#[derive(Clone, Debug)]
pub struct SliderLook {
    /// Captured radial layout and disabled-track presentation.
    pub decoration: crate::theme::stylesheet::SliderDecoration,
    pub track_background: Hsla,
    pub fill_background: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
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

        let selected_thumb = thumb_size.unwrap_or(size.into());
        let mut look = SliderLook {
            decoration: self.tokens.stylesheet.slider.decoration(size.stylesheet_key()),
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
            width: 0.0,
            height: slider_height(metrics, size, &self.tokens.stylesheet),
            track_height: slider_track_height(metrics, size),
            thumb_size: slider_thumb_size_with_stylesheet(metrics, selected_thumb, &self.tokens.stylesheet),
            radius: metrics.radius.pill,
        };
        let thumb_key = thumb_size.map(|size| thumb_control_size(size).stylesheet_key());
        let geometry = self.tokens.stylesheet.slider.resolve_geometry(size.stylesheet_key(), thumb_key, &look);
        look.width = geometry.width.value_px;
        look.height = geometry.height.value_px;
        look.track_height = geometry.track_height.value_px;
        look.thumb_size = geometry.thumb_size.value_px;
        look
    }
}

fn slider_height(
    metrics: &MetricTokens,
    size: ControlSize,
    stylesheet: &crate::theme::stylesheet::CommonStylesheet,
) -> f32 {
    let recipe = stylesheet.slider.authored_geometry(size.stylesheet_key());
    let floor = recipe
        .height_floor_scale
        .or(crate::theme::stylesheet::CommonStylesheet::sdk_default().slider.geometry.height_floor_scale)
        .unwrap_or_default();
    recipe.height.unwrap_or_else(|| {
        (metrics.control_height(size) - metrics.spacing.s1).max(metrics.control_height(size) * floor)
    })
}

fn slider_track_height(metrics: &MetricTokens, size: ControlSize) -> f32 {
    (metrics.gap(size) - metrics.border_width.strong).max(metrics.border_width.strong)
}

fn thumb_control_size(size: SliderThumbSize) -> ControlSize {
    match size {
        SliderThumbSize::Sm => ControlSize::Sm,
        SliderThumbSize::Md => ControlSize::Md,
        SliderThumbSize::Lg => ControlSize::Lg,
    }
}

fn slider_thumb_size_with_stylesheet(
    metrics: &MetricTokens,
    size: SliderThumbSize,
    stylesheet: &crate::theme::stylesheet::CommonStylesheet,
) -> f32 {
    let size = thumb_control_size(size);
    let recipe = stylesheet.slider.authored_geometry(size.stylesheet_key());
    let factor = recipe
        .thumb_scale
        .or(crate::theme::stylesheet::CommonStylesheet::sdk_default().slider.geometry.thumb_scale)
        .unwrap_or_default();
    recipe.thumb_size.unwrap_or_else(|| slider_height(metrics, size, stylesheet) * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_theme_retains_p3_source_and_supplies_mapped_control_colors() {
        use crate::color::{ColorValue, GamutMapping, gpui_bridge};
        let mut source = crate::theme::LumaThemeMode::light();
        let p3 = ColorValue::display_p3(1.0, 0.0, 0.0, 0.37);
        source.palette.state.selected.background = p3;
        let tokens = ThemeTokens::from_source(source, GamutMapping::CssLocalMinde).unwrap();
        let theme = DefaultSliderTheme::new(tokens);
        let painted = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(painted.fill_background, gpui_bridge::to_hsla(p3, GamutMapping::CssLocalMinde).unwrap());
        assert_eq!(theme.tokens.source().palette.state.selected.background, p3);
        let disabled = theme.resolve(ControlSize::Md, None, InteractionState { disabled: true, ..Default::default() });
        assert_ne!(disabled.fill_background, painted.fill_background);
        assert_eq!(theme.tokens.source().palette.state.selected.background, p3);
    }

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
