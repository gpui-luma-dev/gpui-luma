use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::controls::selector_panel::{SelectorItemsPanelLook, default_selector_items_panel_look};
use crate::controls::textfield::apply_control_size_typography;
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

use super::model::SelectorTriggerStyle;

#[derive(Clone, Debug)]
pub struct SelectorPalette {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_icon: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
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
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    pub trigger_focus_border: Option<Hsla>,
    pub menu_offset_y: f32,
    pub items_panel: SelectorItemsPanelLook,
}

/// Selector-owned state that is relevant to trigger presentation in addition to
/// generic pointer/focus interaction state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectorVisualState {
    pub interaction: InteractionState,
    pub open: bool,
    pub selected: bool,
    pub invalid: bool,
}

impl SelectorVisualState {
    pub fn resolved_interaction(self) -> InteractionState {
        InteractionState {
            // Open is an active trigger state, but never overrides disabled or pressed.
            hovered: self.open || self.interaction.hovered,
            invalid: self.invalid || self.interaction.invalid,
            ..self.interaction
        }
    }
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

    fn resolve_visual_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        visual_state: SelectorVisualState,
        size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.resolve_look(trigger_style, visual_state.resolved_interaction(), size, scale, without_elevation)
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
        let trigger_border =
            (!state.disabled && state.invalid).then_some(palette.form.input.invalid_border).or(trigger_border);

        SelectorPalette {
            trigger_background,
            trigger_foreground,
            trigger_icon: palette.app.muted_foreground,
            trigger_border,
            trigger_shadow: None,
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
        let mut look = compose_selector_look(&palette, scale);
        if state.invalid && !state.disabled {
            let invalid_border = self.tokens.palette.form.input.invalid_border;
            look.trigger_border = Some(invalid_border);
            look.trigger_focus_border = Some(invalid_border);
        } else if state.focused && !state.disabled && palette.trigger_border.is_some() {
            look.trigger_focus_border = Some(self.tokens.palette.focus.ring);
        }
        look
    }

    fn resolve_visual_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        visual_state: SelectorVisualState,
        size: ControlSize,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.resolve_look(trigger_style, visual_state.resolved_interaction(), size, scale, without_elevation)
    }
}

pub(crate) fn compose_selector_look(palette: &SelectorPalette, scale: &StandardBoxScale) -> SelectorLook {
    SelectorLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: palette.trigger_icon,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.icon_size,
        trigger_focus_border: None,
        menu_offset_y: scale.gap * 0.5,
        items_panel: palette.items_panel.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_state_takes_precedence_over_hover_and_pressed() {
        let theme = DefaultSelectorTheme::default();
        let state = InteractionState { hovered: true, pressed: true, disabled: true, ..InteractionState::default() };

        let disabled = theme.resolve(SelectorTriggerStyle::Outline, state, false);
        let baseline = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { disabled: true, ..InteractionState::default() },
            false,
        );

        assert_eq!(disabled.trigger_background, baseline.trigger_background);
        assert_eq!(disabled.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(disabled.trigger_border, baseline.trigger_border);
    }

    #[test]
    fn pressed_state_takes_precedence_over_hover() {
        let theme = DefaultSelectorTheme::default();
        let pressed = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { pressed: true, hovered: true, ..InteractionState::default() },
            false,
        );
        let baseline = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { pressed: true, ..InteractionState::default() },
            false,
        );

        assert_eq!(pressed.trigger_background, baseline.trigger_background);
        assert_eq!(pressed.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(pressed.trigger_border, baseline.trigger_border);
    }

    #[test]
    fn default_theme_does_not_add_focus_visuals() {
        let theme = DefaultSelectorTheme::default();
        let focused = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { focused: true, ..InteractionState::default() },
            false,
        );
        let baseline = theme.resolve(SelectorTriggerStyle::Outline, InteractionState::default(), false);

        assert_eq!(focused.trigger_background, baseline.trigger_background);
        assert_eq!(focused.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(focused.trigger_border, baseline.trigger_border);
        assert_eq!(focused.trigger_shadow, baseline.trigger_shadow);

        let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0);
        let focused_look = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                interaction: InteractionState { focused: true, ..InteractionState::default() },
                ..Default::default()
            },
            ControlSize::Md,
            &scale,
            false,
        );
        assert!(focused_look.trigger_focus_border.is_some());
    }

    #[test]
    fn open_state_matches_hover_and_disabled_open_matches_disabled() {
        let theme = DefaultSelectorTheme::default();
        let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0);
        let open = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState { open: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );
        let hover = theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );
        let disabled_open = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                open: true,
                interaction: InteractionState { disabled: true, ..Default::default() },
                ..Default::default()
            },
            ControlSize::Md,
            &scale,
            false,
        );
        let disabled = theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState { disabled: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );

        assert_eq!(open.trigger_background, hover.trigger_background);
        assert_eq!(open.trigger_foreground, hover.trigger_foreground);
        assert_eq!(disabled_open.trigger_background, disabled.trigger_background);
        assert_eq!(disabled_open.trigger_foreground, disabled.trigger_foreground);
    }

    #[test]
    fn invalid_state_uses_destructive_border_and_keeps_it_when_focused() {
        let theme = DefaultSelectorTheme::default();
        let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0);
        let look = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                invalid: true,
                interaction: InteractionState { focused: true, ..Default::default() },
                ..Default::default()
            },
            ControlSize::Md,
            &scale,
            false,
        );

        assert_eq!(look.trigger_border, Some(theme.tokens.palette.form.input.invalid_border));
        assert_eq!(look.trigger_focus_border, look.trigger_border);
    }
}
