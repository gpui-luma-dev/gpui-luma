use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::controls::selector_panel::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::textfield::apply_control_size_typography;
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};
use crate::theme::adorner::{AdornerSpec, focus_ring_adorner};

use super::model::SelectorTriggerStyle;

#[derive(Clone, Debug)]
pub struct SelectorPalette {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_icon: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub adorner: Option<AdornerSpec>,
    pub trigger_typography: LumaTextStyle,
    pub items_panel: SelectorItemsPanelLook,
}

#[derive(Clone, Debug)]
pub struct SelectorLook {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_icon: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub adorner: Option<AdornerSpec>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub menu_offset_y: f32,
    pub items_panel: SelectorItemsPanelLook,
}

pub trait SelectorTheme: Send + Sync {
    fn resolve(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        _size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        compose_selector_look(&self.resolve(trigger_style, state, without_elevation), scale)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSelectorTheme {
    tokens: ThemeTokens,
}

pub fn default_selector_theme() -> Arc<dyn SelectorTheme> {
    static THEME: OnceLock<Arc<dyn SelectorTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSelectorTheme::default())).clone()
}

impl DefaultSelectorTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SelectorTheme for DefaultSelectorTheme {
    fn resolve(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        let _ = without_elevation;
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;

        let trigger_background = match state.layer() {
            InteractionLayer::Disabled => palette.state.disabled.background,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default => match trigger_style {
                SelectorTriggerStyle::Outline => palette.app.background,
                SelectorTriggerStyle::Ghost => gpui::hsla(0.0, 0.0, 0.0, 0.0),
            },
        };
        let trigger_foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };
        let trigger_border = match trigger_style {
            SelectorTriggerStyle::Outline => Some(palette.border.default),
            SelectorTriggerStyle::Ghost => None,
        };

        SelectorPalette {
            trigger_background,
            trigger_foreground,
            trigger_icon: palette.app.muted_foreground,
            trigger_border,
            trigger_shadow: None,
            adorner: focus_ring_adorner(state.focused.then_some(palette.focus.ring), &self.tokens.metrics),
            trigger_typography: typography.text.label,
            items_panel: default_selector_items_panel_look(&self.tokens, ControlSize::Md),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn resolve_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        let mut palette = self.resolve(trigger_style, state, without_elevation);
        apply_control_size_typography(&mut palette.trigger_typography, &self.tokens.typography, size);
        palette.items_panel = default_selector_items_panel_look(&self.tokens, size);
        compose_selector_look(&palette, scale)
    }
}

pub(crate) fn compose_selector_look(palette: &SelectorPalette, scale: &StandardBoxScale) -> SelectorLook {
    SelectorLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: palette.trigger_icon,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        adorner: palette.adorner,
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.height / 3.0,
        menu_offset_y: scale.gap * 0.5,
        items_panel: palette.items_panel.clone(),
    }
}
