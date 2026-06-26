use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::theme::{LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens};
use crate::controls::textarea::TextAreaState;

#[derive(Clone, Debug)]
pub struct TextAreaPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub focus_ring: Option<Hsla>,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
}

#[derive(Clone, Debug)]
pub struct TextAreaLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub focus_ring: Option<Hsla>,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub border_width: f32,
}

pub trait TextAreaTheme: Send + Sync {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(&self, state: TextAreaState, enabled: bool, scale: &StandardBoxScale) -> TextAreaLook {
        compose_textarea_look(&self.resolve(state, enabled), scale, self.metrics().border_width.default)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTextAreaTheme {
    tokens: ThemeTokens,
}

pub fn default_textarea_theme() -> Arc<dyn TextAreaTheme> {
    static THEME: OnceLock<Arc<dyn TextAreaTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTextAreaTheme::default())).clone()
}

impl DefaultTextAreaTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TextAreaTheme for DefaultTextAreaTheme {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;

        let (background, foreground, border, placeholder, selection_background, selection_foreground, caret) =
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
                    palette.state.selected.background,
                    palette.state.selected.foreground,
                    palette.state.disabled.foreground,
                )
            };

        TextAreaPalette {
            background,
            foreground,
            border,
            placeholder,
            selection_background,
            selection_foreground,
            caret,
            focus_ring: (enabled && state.focus_visible).then_some(palette.focus.ring),
            shadow: None,
            typography: typography.text.body,
            font_family: typography.font.sans.family.clone(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_textarea_look(
    palette: &TextAreaPalette,
    scale: &StandardBoxScale,
    border_width: f32,
) -> TextAreaLook {
    TextAreaLook {
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        placeholder: palette.placeholder,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        focus_ring: palette.focus_ring,
        shadow: palette.shadow.clone(),
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        radius: scale.radius,
        border_width,
    }
}
