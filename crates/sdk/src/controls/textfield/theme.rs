use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{ControlSize, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage};
use crate::controls::textfield::TextFieldState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    #[default]
    Standard,
    Ghost,
}

#[derive(Clone, Debug)]
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
    pub font_family: SharedString,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon_size: f32,
}

pub trait TextFieldTheme: Send + Sync {
    fn resolve(&self, variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextFieldTheme {
    tokens: ThemeTokens,
}

pub fn default_textfield_theme() -> Arc<dyn TextFieldTheme> {
    if let Some(live) = crate::theme::pack::active_live_theme() {
        return live;
    }
    static THEME: OnceLock<Arc<dyn TextFieldTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextFieldTheme::default())).clone()
}

pub const TEXTFIELD_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "TextField",
    parts: &[
        ThemePartUsage {
            part: "standard.background",
            token: "form.input.background",
            states: &["standard default", "standard focused"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "standard.hover_background",
            token: "state.hover.background",
            states: &["standard hovered"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "standard.foreground",
            token: "form.input.foreground",
            states: &["standard default", "standard focused"],
            appearance_fields: &["TextFieldAppearance.foreground", "TextFieldAppearance.caret"],
        },
        ThemePartUsage {
            part: "standard.border",
            token: "form.input.border",
            states: &["standard default", "standard hovered", "standard disabled"],
            appearance_fields: &["TextFieldAppearance.border"],
        },
        ThemePartUsage {
            part: "standard.placeholder_and_icon",
            token: "form.input.placeholder",
            states: &["standard empty"],
            appearance_fields: &["TextFieldAppearance.placeholder", "TextFieldAppearance.icon"],
        },
        ThemePartUsage {
            part: "standard.invalid_border",
            token: "form.input.invalid_border",
            states: &["standard invalid"],
            appearance_fields: &["TextFieldAppearance.border"],
        },
        ThemePartUsage {
            part: "ghost.background",
            token: "action.ghost.background",
            states: &["ghost focused"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "ghost.hover_background",
            token: "action.ghost.hover_background",
            states: &["ghost hovered"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "ghost.foreground",
            token: "action.ghost.foreground",
            states: &["ghost default", "ghost focused"],
            appearance_fields: &["TextFieldAppearance.foreground", "TextFieldAppearance.caret"],
        },
        ThemePartUsage {
            part: "ghost.border",
            token: "action.ghost.border",
            states: &["ghost default", "ghost hovered", "ghost focused", "ghost invalid", "ghost disabled"],
            appearance_fields: &["TextFieldAppearance.border"],
        },
        ThemePartUsage {
            part: "ghost.placeholder_and_icon",
            token: "action.ghost.foreground",
            states: &["ghost empty"],
            appearance_fields: &["TextFieldAppearance.placeholder", "TextFieldAppearance.icon"],
        },
        ThemePartUsage {
            part: "ghost.invalid_border",
            token: "form.input.invalid_border",
            states: &[],
            appearance_fields: &[],
        },
        ThemePartUsage {
            part: "selection",
            token: "state.selected.background",
            states: &["selection"],
            appearance_fields: &["TextFieldAppearance.selection_background"],
        },
        ThemePartUsage {
            part: "disabled.background",
            token: "state.disabled.background",
            states: &["disabled"],
            appearance_fields: &["TextFieldAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled.foreground",
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
            part: "focus.ring",
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
    fn resolve(&self, variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size = ControlSize::Md;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };

        let (background, foreground, border, placeholder, icon, selection_background, caret) = match (variant, enabled)
        {
            (TextFieldVariant::Standard, true) => {
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
            }
            (TextFieldVariant::Ghost, true) => {
                let background = if state.focused {
                    palette.action.ghost.background
                } else if state.hovered {
                    palette.action.ghost.hover_background
                } else {
                    transparent
                };
                let placeholder_color = palette.action.ghost.foreground.opacity(0.65);

                (
                    background,
                    palette.action.ghost.foreground,
                    palette.action.ghost.border,
                    placeholder_color,
                    placeholder_color,
                    palette.state.selected.background,
                    palette.action.ghost.foreground,
                )
            }
            (TextFieldVariant::Standard, false) => (
                palette.state.disabled.background,
                palette.state.disabled.foreground,
                palette.form.input.border,
                palette.state.disabled.foreground,
                palette.state.disabled.foreground,
                palette.state.selected.background,
                palette.state.disabled.foreground,
            ),
            (TextFieldVariant::Ghost, false) => (
                transparent,
                palette.state.disabled.foreground,
                palette.action.ghost.border,
                palette.state.disabled.foreground,
                palette.state.disabled.foreground,
                palette.state.selected.background,
                palette.state.disabled.foreground,
            ),
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
            font_family: typography.font.sans.family.clone().into(),
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
