use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens};
use crate::controls::textfield::TextFieldState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    #[default]
    Standard,
}

#[derive(Clone, Debug)]
pub struct TextFieldPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct TextFieldLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
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
    fn resolve(&self, variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        compose_textfield_look(&self.resolve(variant, state, enabled), scale, self.metrics().border_width.default)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextFieldTheme {
    tokens: ThemeTokens,
}

pub fn default_textfield_theme() -> Arc<dyn TextFieldTheme> {
    static THEME: OnceLock<Arc<dyn TextFieldTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextFieldTheme::default())).clone()
}

impl DefaultTextFieldTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextFieldTheme for DefaultTextFieldTheme {
    fn resolve(&self, _variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;

        let (background, foreground, border, placeholder, icon, selection_background, selection_foreground, caret) =
            if enabled {
                let background = palette.form.input.background;
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
                    palette.state.selected.foreground,
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
                    palette.state.selected.foreground,
                    palette.state.disabled.foreground,
                )
            };

        TextFieldPalette {
            background,
            foreground,
            border,
            placeholder,
            icon,
            selection_background,
            selection_foreground,
            caret,
            focus_ring: (enabled && state.focus_visible).then_some(palette.focus.ring),
            typography: typography.text.body,
            font_family: typography.font.sans.family.clone().into(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_textfield_look(
    palette: &TextFieldPalette,
    scale: &StandardBoxScale,
    border_width: f32,
) -> TextFieldLook {
    TextFieldLook {
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        placeholder: palette.placeholder,
        icon: palette.icon,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        focus_ring: palette.focus_ring,
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        gap: scale.gap,
        radius: scale.radius,
        border_width,
        icon_size: palette.typography.size + 2.0,
    }
}
