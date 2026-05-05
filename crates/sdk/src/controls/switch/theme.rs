use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

#[derive(Clone, Debug)]
pub struct SwitchAppearance {
    pub track_background: Hsla,
    pub track_border: Hsla,
    pub thumb_background: Hsla,
    pub thumb_border: Hsla,
    pub thumb_shadow: Vec<BoxShadow>,
    pub label_color: Hsla,
    pub adorners: Vec<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
}

pub trait SwitchTheme: Send + Sync {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSwitchTheme {
    tokens: ThemeTokens,
}

pub fn default_switch_theme() -> Arc<dyn SwitchTheme> {
    static THEME: OnceLock<Arc<dyn SwitchTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSwitchTheme::default())).clone()
}

pub const SWITCH_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Switch",
    parts: &[
        ThemePartUsage {
            part: "off track background",
            token: "form.input.background",
            states: &["off"],
            appearance_fields: &["SwitchAppearance.track_background"],
        },
        ThemePartUsage {
            part: "off track hover background",
            token: "state.hover.background",
            states: &["off hovered"],
            appearance_fields: &["SwitchAppearance.track_background"],
        },
        ThemePartUsage {
            part: "off track pressed background",
            token: "state.pressed.background",
            states: &["off pressed"],
            appearance_fields: &["SwitchAppearance.track_background"],
        },
        ThemePartUsage {
            part: "off track border",
            token: "form.input.border",
            states: &["off", "off hovered", "off pressed"],
            appearance_fields: &["SwitchAppearance.track_border"],
        },
        ThemePartUsage {
            part: "on track background",
            token: "action.prominent.background",
            states: &["on"],
            appearance_fields: &["SwitchAppearance.track_background", "SwitchAppearance.track_border"],
        },
        ThemePartUsage {
            part: "on track hover background",
            token: "action.prominent.hover_background",
            states: &["on hovered"],
            appearance_fields: &["SwitchAppearance.track_background", "SwitchAppearance.track_border"],
        },
        ThemePartUsage {
            part: "on track pressed background",
            token: "action.prominent.pressed_background",
            states: &["on pressed"],
            appearance_fields: &["SwitchAppearance.track_background", "SwitchAppearance.track_border"],
        },
        ThemePartUsage {
            part: "on thumb",
            token: "action.prominent.foreground",
            states: &["on"],
            appearance_fields: &["SwitchAppearance.thumb_background", "SwitchAppearance.thumb_border"],
        },
        ThemePartUsage {
            part: "off thumb background",
            token: "surface.panel.background",
            states: &["off"],
            appearance_fields: &["SwitchAppearance.thumb_background"],
        },
        ThemePartUsage {
            part: "off thumb border",
            token: "border.default",
            states: &["off"],
            appearance_fields: &["SwitchAppearance.thumb_border"],
        },
        ThemePartUsage {
            part: "label",
            token: "app.foreground",
            states: &["default", "on"],
            appearance_fields: &["SwitchAppearance.label_color"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["SwitchAppearance.track_background", "SwitchAppearance.thumb_border"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["SwitchAppearance.thumb_background", "SwitchAppearance.label_color"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["SwitchAppearance.adorners"],
        },
    ],
};

impl DefaultSwitchTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SwitchTheme for DefaultSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let size = ControlSize::Md;
        let layer = state.layer();

        let track_background = match (on, layer) {
            (_, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => palette.action.prominent.pressed_background,
            (true, InteractionLayer::Hovered) => palette.action.prominent.hover_background,
            (true, InteractionLayer::Default) => palette.action.prominent.background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.form.input.background,
        };

        let adorners = if state.focused {
            vec![AdornerSpec::FocusRing(FocusRingAdornerSpec {
                color: palette.focus.ring,
                placement: AdornerPlacement::Oversize,
                distance: metrics.border_width.default + metrics.focus.width,
                width: metrics.focus.width,
            })]
        } else {
            Vec::new()
        };

        SwitchAppearance {
            track_background,
            track_border: if on && !state.disabled {
                track_background
            } else {
                palette.form.input.border
            },
            thumb_background: if state.disabled {
                palette.state.disabled.foreground
            } else if on {
                palette.action.prominent.foreground
            } else {
                palette.surface.panel.background
            },
            thumb_border: if state.disabled {
                palette.state.disabled.background
            } else if on {
                palette.action.prominent.foreground
            } else {
                palette.border.default
            },
            thumb_shadow: elevation.thumb.to_box_shadows(),
            label_color: if state.disabled {
                palette.state.disabled.foreground
            } else {
                palette.app.foreground
            },
            adorners,
            label_typography: typography.text.label,
            width: 42.0,
            height: 22.0,
            thumb_size: metrics.control_height(size) * 0.5,
            padding: 2.0,
            gap: metrics.gap(size),
            radius: metrics.radius.pill,
        }
    }
}
