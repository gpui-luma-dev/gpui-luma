use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonFamilyRole {
    #[default]
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub focus_ring: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyLook {
    pub background: Hsla,
    pub foreground: Hsla,
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub focus_ring: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
}

pub trait ButtonFamilyTheme: Send + Sync {
    fn resolve(&self, role: ButtonFamilyRole, size: ControlSize, state: InteractionState) -> ButtonFamilyPalette;
    fn metrics(&self) -> MetricTokens;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultButtonFamilyTheme {
    tokens: ThemeTokens,
}

pub fn default_button_family_theme() -> Arc<dyn ButtonFamilyTheme> {
    static THEME: OnceLock<Arc<dyn ButtonFamilyTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultButtonFamilyTheme::default())).clone()
}

impl DefaultButtonFamilyTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ButtonFamilyTheme for DefaultButtonFamilyTheme {
    fn resolve(&self, role: ButtonFamilyRole, _size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        native_button_palette(&self.tokens, role, state)
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

fn native_button_palette(tokens: &ThemeTokens, role: ButtonFamilyRole, state: InteractionState) -> ButtonFamilyPalette {
    let palette = &tokens.palette;
    let typography = &tokens.typography;
    let layer = state.layer();

    let (base_background, base_foreground, border) = match role {
        ButtonFamilyRole::Toggle { selected: true } => {
            (palette.state.selected.background, palette.state.selected.foreground, palette.border.default)
        }
        _ => (palette.surface.subtle.background, palette.app.foreground, palette.border.default),
    };

    let foreground = if state.disabled {
        palette.state.disabled.foreground
    } else {
        base_foreground
    };

    let background = match layer {
        InteractionLayer::Disabled => palette.state.disabled.background,
        InteractionLayer::Pressed => palette.state.pressed.background,
        InteractionLayer::Hovered => palette.state.hover.background,
        InteractionLayer::Default => base_background,
    };

    ButtonFamilyPalette {
        background,
        foreground,
        border: Some(border),
        focus_ring: palette.focus.ring,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}

/// Resolves the border color used for layout and focus-ring placement.
///
/// Absent borders are treated as transparent (borderless).
pub fn button_family_effective_border(border: Option<Hsla>) -> Hsla {
    border.unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.0))
}

/// Builds a focus-ring adorner from resolved look colors and scaffold metrics.
///
/// Borderless controls (`border.a == 0`) use an inset ring flush with the edge; bordered
/// controls use an oversize ring outside the border box.
pub fn button_family_focus_adorner(
    focused: bool,
    border: Option<Hsla>,
    focus_ring: Hsla,
    metrics: &MetricTokens,
) -> Option<AdornerSpec> {
    if !focused {
        return None;
    }

    let effective_border = button_family_effective_border(border);
    let borderless = effective_border.a <= 0.0;
    let (placement, distance) = if borderless {
        (AdornerPlacement::Inset, 0.0)
    } else {
        (AdornerPlacement::Oversize, metrics.border_width.default + metrics.focus.width)
    };

    Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
        color: focus_ring,
        placement,
        distance,
        width: metrics.focus.width,
    }))
}

pub fn compose_button_family_look(
    palette: &ButtonFamilyPalette,
    role: ButtonFamilyRole,
    scale: &StandardBoxScale,
    pill_radius: f32,
) -> ButtonFamilyLook {
    ButtonFamilyLook {
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        focus_ring: palette.focus_ring,
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        radius: match role {
            ButtonFamilyRole::Icon => pill_radius,
            _ => scale.radius,
        },
        padding_x: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_x,
        },
        padding_y: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_y,
        },
        gap: scale.gap,
        height: scale.height,
    }
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::*;

    #[test]
    fn effective_border_is_transparent_when_absent() {
        let effective = button_family_effective_border(None);
        assert_eq!(effective.a, 0.0);
    }

    #[test]
    fn focus_adorner_is_inset_when_borderless() {
        let metrics = MetricTokens::default();
        let ring = hsla(200.0, 1.0, 0.5, 1.0);

        let adorner = button_family_focus_adorner(true, None, ring, &metrics).expect("focused adorner");
        match adorner {
            AdornerSpec::FocusRing(spec) => {
                assert_eq!(spec.placement, AdornerPlacement::Inset);
                assert_eq!(spec.distance, 0.0);
                assert_eq!(spec.color, ring);
            }
        }
    }

    #[test]
    fn focus_adorner_is_oversize_when_border_visible() {
        let metrics = MetricTokens::default();
        let border = hsla(0.0, 0.0, 0.0, 1.0);
        let ring = hsla(200.0, 1.0, 0.5, 1.0);

        let adorner = button_family_focus_adorner(true, Some(border), ring, &metrics).expect("focused adorner");
        match adorner {
            AdornerSpec::FocusRing(spec) => {
                assert_eq!(spec.placement, AdornerPlacement::Oversize);
                assert_eq!(spec.distance, metrics.border_width.default + metrics.focus.width);
            }
        }
    }

    #[test]
    fn focus_adorner_absent_when_not_focused() {
        let metrics = MetricTokens::default();
        assert!(
            button_family_focus_adorner(false, Some(hsla(0.0, 0.0, 0.0, 1.0)), hsla(0.0, 0.0, 1.0, 1.0), &metrics)
                .is_none()
        );
    }
}
