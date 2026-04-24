use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ControlSize, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};
use crate::controls::textfield::TextFieldState;

#[derive(Clone, Copy, Debug)]
pub struct TextFieldAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub caret: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon_size: f32,
}

pub trait TextFieldTheme: Send + Sync {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextFieldTheme {
    tokens: ThemeTokens,
}

pub fn default_textfield_theme() -> Arc<dyn TextFieldTheme> {
    static THEME: OnceLock<Arc<dyn TextFieldTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextFieldTheme::default())).clone()
}

pub const TEXTFIELD_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "TextField",
    parts: &[
        ThemePartUsage {
            part: "background",
            token: "form.input.background",
            states: &["default"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "hover background",
            token: "state.hover.background",
            states: &["hovered"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "foreground",
            token: "form.input.foreground",
            states: &["default", "focused"],
            appearance_fields: &["TextFieldAppearance.foreground", "TextFieldAppearance.caret"],
        },
        ThemePartUsage {
            part: "border",
            token: "form.input.border",
            states: &["default", "hovered", "disabled"],
            appearance_fields: &["TextFieldAppearance.border"],
        },
        ThemePartUsage {
            part: "placeholder and icon",
            token: "form.input.placeholder",
            states: &["empty"],
            appearance_fields: &["TextFieldAppearance.placeholder", "TextFieldAppearance.icon"],
        },
        ThemePartUsage {
            part: "selection",
            token: "state.selected.background",
            states: &["selection"],
            appearance_fields: &["TextFieldAppearance.selection_background"],
        },
        ThemePartUsage {
            part: "invalid border",
            token: "form.input.invalid_border",
            states: &["invalid"],
            appearance_fields: &["TextFieldAppearance.border"],
        },
        ThemePartUsage {
            part: "disabled fill",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &[
                "TextFieldAppearance.foreground",
                "TextFieldAppearance.placeholder",
                "TextFieldAppearance.icon",
                "TextFieldAppearance.caret",
            ],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focus visible"],
            appearance_fields: &["TextFieldAppearance.focus_ring"],
        },
    ],
};

impl DefaultTextFieldTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextFieldTheme for DefaultTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let (background, foreground, border, placeholder, icon, selection_background, caret) = if enabled {
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
                palette.state.disabled.foreground,
                palette.state.selected.background,
                palette.state.disabled.foreground,
            )
        };

        TextFieldAppearance {
            background,
            foreground,
            border,
            placeholder,
            icon,
            selection_background,
            caret,
            focus_ring: (enabled && state.focus_visible).then_some(palette.focus.ring),
            typography: typography.text.body,
            min_height: metrics.control_height(size),
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            gap: metrics.gap(size),
            radius: metrics.radius(size),
            border_width: metrics.border_width.default,
            icon_size: typography.text.body.size + 2.0,
        }
    }
}
