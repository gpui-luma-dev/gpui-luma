use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, hsla};

use super::PopupMenuTriggerStyle;
use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaLayoutCacheExt, LumaTextStyle, MetricTokens, StandardBoxScale,
    ThemeTokens,
};

#[derive(Clone, Debug)]
pub struct PopupMenuPalette {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Debug)]
pub struct PopupMenuLook {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_offset_y: f32,
    pub floating_menu: FloatingMenuLook,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PopupMenuTriggerMetrics {
    pub size: ControlSize,
    pub menu_size: ControlSize,
    pub without_elevation: bool,
    pub icon_only: bool,
    pub trigger_radius_override: Option<f32>,
}

pub trait PopupMenuTheme: Send + Sync {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
        scale_factor: f32,
        cx: &mut gpui::App,
    ) -> PopupMenuLook {
        let scale = cx.use_cached_layout(
            self.metrics(),
            crate::theme::LayoutCacheKey { size: metrics.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics_tokens| StandardBoxScale::compute(metrics.size, metrics_tokens, scale_factor),
        );
        let mut look = compose_popup_menu_look(&self.resolve(trigger_style, metrics, state), &scale);
        if let Some(radius) = metrics.trigger_radius_override {
            look.trigger_radius = radius;
        }
        look
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPopupMenuTheme {
    tokens: ThemeTokens,
}

pub fn default_popup_menu_theme() -> Arc<dyn PopupMenuTheme> {
    static THEME: OnceLock<Arc<dyn PopupMenuTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPopupMenuTheme::default())).clone()
}

impl DefaultPopupMenuTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PopupMenuTheme for DefaultPopupMenuTheme {
    fn resolve(
        &self,
        trigger_style: PopupMenuTriggerStyle,
        metrics: PopupMenuTriggerMetrics,
        state: InteractionState,
    ) -> PopupMenuPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;
        let layer = state.layer();

        let (trigger_background, trigger_foreground, trigger_border, trigger_shadow) = match trigger_style {
            PopupMenuTriggerStyle::Outline => {
                let background = match layer {
                    InteractionLayer::Disabled => palette.state.disabled.background,
                    InteractionLayer::Pressed => palette.state.pressed.background,
                    InteractionLayer::Hovered => palette.state.hover.background,
                    InteractionLayer::Default => palette.app.background,
                };
                (
                    background,
                    if state.disabled {
                        palette.state.disabled.foreground
                    } else {
                        palette.app.foreground
                    },
                    Some(palette.border.default),
                    Some(self.tokens.elevation.control.to_box_shadows()),
                )
            }
            PopupMenuTriggerStyle::Ghost => {
                let background = match layer {
                    InteractionLayer::Disabled => palette.state.disabled.background,
                    InteractionLayer::Pressed | InteractionLayer::Hovered => palette.state.hover.background,
                    InteractionLayer::Default => hsla(0.0, 0.0, 0.0, 0.0),
                };
                (
                    background,
                    if state.disabled {
                        palette.state.disabled.foreground
                    } else {
                        palette.app.foreground
                    },
                    None,
                    None,
                )
            }
        };

        let trigger_shadow = if metrics.without_elevation {
            None
        } else {
            trigger_shadow
        };

        PopupMenuPalette {
            trigger_background,
            trigger_foreground,
            trigger_border,
            trigger_shadow,
            trigger_typography: typography.text.label,
            floating_menu: default_floating_menu_look(&self.tokens, metrics.menu_size),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub fn compose_popup_menu_look(palette: &PopupMenuPalette, scale: &StandardBoxScale) -> PopupMenuLook {
    PopupMenuLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.height * 0.44,
        menu_offset_y: scale.gap * 0.5,
        floating_menu: palette.floating_menu.clone(),
    }
}
