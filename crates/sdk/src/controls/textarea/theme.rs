use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{ControlSize, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};
use crate::controls::textarea::TextAreaState;

#[derive(Clone, Debug)]
pub struct TextAreaAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub caret: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: String,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
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
    label: "TextArea",
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
            appearance_fields: &["TextAreaAppearance.foreground", "TextAreaAppearance.caret"],
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
            part: "selection",
            token: "state.selected.background",
            states: &["selection"],
            appearance_fields: &["TextAreaAppearance.selection_background"],
        },
        ThemePartUsage {
            part: "invalid border",
            token: "form.input.invalid_border",
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
            appearance_fields: &[
                "TextAreaAppearance.foreground",
                "TextAreaAppearance.placeholder",
                "TextAreaAppearance.caret",
            ],
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

        let (background, foreground, border, placeholder, selection_background, caret) = if enabled {
            let background = if state.focused {
                palette.form.input.background
            } else if state.hovered {
                palette.state.hover.background
            } else {
                palette.form.input.background
            };
            let border = if state.invalid {
                palette.form.input.invalid_border
            } else {
                palette.form.input.border
            };

            (
                background,
                palette.form.input.foreground,
                border,
                palette.form.input.placeholder,
                palette.state.selected.background,
                palette.form.input.foreground,
            )
        } else {
            (
                palette.state.disabled.background,
                palette.state.disabled.foreground,
                palette.form.input.border,
                palette.state.disabled.foreground,
                palette.state.selected.background,
                palette.state.disabled.foreground,
            )
        };

        TextAreaAppearance {
            background,
            foreground,
            border,
            placeholder,
            selection_background,
            caret,
            focus_ring: (enabled && state.focus_visible).then_some(palette.focus.ring),
            typography: typography.text.body,
            font_family: typography.font.sans.family.clone(),
            min_height: metrics.control_height(size),
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            radius: metrics.radius(size),
            border_width: metrics.border_width.default,
        }
    }
}
