use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::scales::snap_to_pixel;
use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, ThemeTokens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccordionScale {
    pub trigger_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub content_padding_y: f32,
    pub radius: f32,
    pub item_gap: f32,
    pub inner_gap: f32,
    pub icon_size: f32,
    pub chevron_size: f32,
}

impl AccordionScale {
    pub fn compute(size: ControlSize, metrics: &MetricTokens, scale_factor: f32) -> Self {
        let base_height = metrics.control_height(size);

        Self {
            trigger_height: snap_to_pixel(base_height, scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(size), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(size), scale_factor),
            content_padding_y: snap_to_pixel(metrics.padding_y(size) * 1.5, scale_factor),
            radius: metrics.radius(size),
            item_gap: 0.0,
            inner_gap: snap_to_pixel(metrics.gap(size), scale_factor),
            icon_size: 16.0,
            chevron_size: 14.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AccordionPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub border_color: Hsla,
    pub icon_color: Hsla,
    pub chevron_color: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct AccordionContentPalette {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
}

pub trait AccordionTheme: Send + Sync {
    fn resolve_trigger(&self, state: InteractionState, size: ControlSize) -> AccordionPalette;
    fn resolve_content(&self, expanded: bool) -> AccordionContentPalette;
    fn metrics(&self) -> MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAccordionTheme {
    tokens: ThemeTokens,
}

pub fn default_accordion_theme() -> Arc<dyn AccordionTheme> {
    static THEME: OnceLock<Arc<dyn AccordionTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultAccordionTheme::default())).clone()
}

impl DefaultAccordionTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AccordionTheme for DefaultAccordionTheme {
    fn resolve_trigger(&self, state: InteractionState, size: ControlSize) -> AccordionPalette {
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

        let mut trigger_typography = typography.text.label;
        crate::controls::textfield::apply_control_size_typography(&mut trigger_typography, typography, size);

        AccordionPalette {
            background,
            foreground,
            border_color: palette.navigation.border,
            icon_color: foreground,
            chevron_color: palette.navigation.muted_foreground,
            typography: trigger_typography,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn resolve_content(&self, _expanded: bool) -> AccordionContentPalette {
        let palette = &self.tokens.palette;

        AccordionContentPalette { background: None, foreground: palette.app.foreground }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}
