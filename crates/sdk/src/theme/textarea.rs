use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};
use crate::controls::textarea::TextAreaState;

#[derive(Clone, Copy, Debug)]
pub struct TextAreaAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub line_gap: f32,
    pub radius: f32,
    pub border_width: f32,
}

pub trait TextAreaTheme: Send + Sync {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextAreaTheme {
    tokens: ThemeTokens,
}

pub fn default_textarea_theme() -> Arc<dyn TextAreaTheme> {
    static THEME: OnceLock<Arc<dyn TextAreaTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextAreaTheme::default())).clone()
}

pub const TEXTAREA_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "TextArea",
    parts: &[
        ThemePartUsage {
            part: "background",
            token: "form.input.background",
            states: &["default"],
            appearance_fields: &["TextAreaAppearance.background"],
        },
        ThemePartUsage {
            part: "hover background",
            token: "state.hover.background",
            states: &["hovered"],
            appearance_fields: &["TextAreaAppearance.background"],
        },
        ThemePartUsage {
            part: "foreground",
            token: "form.input.foreground",
            states: &["default", "focused"],
            appearance_fields: &["TextAreaAppearance.foreground"],
        },
        ThemePartUsage {
            part: "border",
            token: "form.input.border",
            states: &["default", "hovered", "disabled"],
            appearance_fields: &["TextAreaAppearance.border"],
        },
        ThemePartUsage {
            part: "placeholder",
            token: "form.input.placeholder",
            states: &["empty"],
            appearance_fields: &["TextAreaAppearance.placeholder"],
        },
        ThemePartUsage {
            part: "invalid border",
            token: "action.danger.background",
            states: &["invalid"],
            appearance_fields: &["TextAreaAppearance.border"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["TextAreaAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["TextAreaAppearance.foreground", "TextAreaAppearance.placeholder"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focus visible"],
            appearance_fields: &["TextAreaAppearance.focus_ring"],
        },
    ],
};

impl DefaultTextAreaTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextAreaTheme for DefaultTextAreaTheme {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let (background, foreground, border, placeholder) = if enabled {
            (
                if state.focused {
                    palette.form.input.background
                } else if state.hovered {
                    palette.state.hover.background
                } else {
                    palette.form.input.background
                },
                palette.form.input.foreground,
                if state.invalid {
                    palette.action.danger.background
                } else {
                    palette.form.input.border
                },
                palette.form.input.placeholder,
            )
        } else {
            (
                palette.state.disabled.background,
                palette.state.disabled.foreground,
                palette.form.input.border,
                palette.state.disabled.foreground,
            )
        };

        TextAreaAppearance {
            background,
            foreground,
            border,
            placeholder,
            focus_ring: (enabled && state.focus_visible).then_some(palette.focus.ring),
            typography: typography.text.body,
            min_height: metrics.control_height(size) * 2.75,
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            line_gap: metrics.spacing.s1,
            radius: metrics.radius(size),
            border_width: metrics.border_width.default,
        }
    }
}
