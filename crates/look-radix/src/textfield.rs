//! Radix text-field theme adapter.
//!
//! Variants match Radix Themes Text Field: `classic` | `surface` | `soft`.

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, SharedString, point, px};
use luma::controls::textfield::{
    TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTheme, TextFieldVariant, compose_textfield_look,
};
use luma::theme::{ControlSize, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::RadixLook;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;

/// Radix Themes text-field / text-area visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadixTextFieldVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

impl RadixTextFieldVariant {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Surface, Self::Soft];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Surface => "surface",
            Self::Soft => "soft",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Surface => "Surface",
            Self::Soft => "Soft",
        }
    }
}

struct RadixTextFieldTheme {
    look: RadixLook,
    variant: RadixTextFieldVariant,
}

impl TextFieldTheme for RadixTextFieldTheme {
    fn resolve(&self, _variant: TextFieldVariant, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        resolve_text_chrome_palette(&self.look, self.variant, state.hovered, state.focused, state.invalid, enabled)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        variant: TextFieldVariant,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        let mut palette = self.resolve(variant, state, enabled);
        palette.typography = typography_for_size(size);
        let mut look = compose_textfield_look(&palette, scale, self.metrics().border_width.default);
        if matches!(self.variant, RadixTextFieldVariant::Soft | RadixTextFieldVariant::Classic) {
            // Soft has no rim; Classic draws its edge via shadow-1.
            look.border_width = 0.0;
        }
        look
    }
}

pub fn textfield_theme(look: Arc<RadixLook>) -> Arc<dyn TextFieldTheme> {
    textfield_theme_with(look, RadixTextFieldVariant::default())
}

pub fn textfield_theme_with(look: Arc<RadixLook>, variant: RadixTextFieldVariant) -> Arc<dyn TextFieldTheme> {
    Arc::new(RadixTextFieldTheme { look: look.as_ref().clone(), variant })
}

pub(crate) fn resolve_text_chrome_palette(
    look: &RadixLook,
    variant: RadixTextFieldVariant,
    hovered: bool,
    focused: bool,
    invalid: bool,
    enabled: bool,
) -> TextFieldPalette {
    let foreground = look.resolve_role(SemanticRole::Foreground).hsla();
    let placeholder = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let selection_background = look.resolve_role(SemanticRole::Soft).hsla();
    let selection_foreground = look.resolve_role(SemanticRole::SoftForeground).hsla();
    let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
    let accent = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
    let surface = look.resolve_role(SemanticRole::Surface).hsla();

    if !enabled {
        return TextFieldPalette {
            background: match variant {
                RadixTextFieldVariant::Soft => alpha(gray(3), 0.55),
                RadixTextFieldVariant::Classic | RadixTextFieldVariant::Surface => surface,
            },
            foreground: placeholder,
            border: match variant {
                RadixTextFieldVariant::Soft => gpui::transparent_black(),
                RadixTextFieldVariant::Classic | RadixTextFieldVariant::Surface => gray(6),
            },
            placeholder,
            icon: placeholder,
            selection_background,
            selection_foreground,
            caret: placeholder,
            shadow: None,
            typography: typography_for_size(ControlSize::Md),
            font_family: SharedString::from("System UI"),
        };
    }

    // Radix: surface = inset gray rim; classic = raised shadow-1; soft = accent tint, no border.
    let (background, border, shadow) = if invalid {
        let bg = match variant {
            RadixTextFieldVariant::Soft => accent(3),
            RadixTextFieldVariant::Classic | RadixTextFieldVariant::Surface => surface,
        };
        (bg, look.resolve_role(SemanticRole::Destructive).hsla(), None)
    } else if focused {
        let bg = match variant {
            RadixTextFieldVariant::Soft => accent(3),
            RadixTextFieldVariant::Classic | RadixTextFieldVariant::Surface => surface,
        };
        let focus = look.resolve_role(SemanticRole::Focus).hsla();
        (bg, focus, None)
    } else {
        match variant {
            RadixTextFieldVariant::Surface => {
                let border = if hovered { gray(8) } else { gray(7) };
                (surface, border, None)
            }
            RadixTextFieldVariant::Classic => {
                // Classic draws its edge via `--shadow-1`, not a gray stroke.
                (surface, gpui::transparent_black(), Some(classic_field_shadow(gray(5), alpha(gray(2), 0.45))))
            }
            RadixTextFieldVariant::Soft => {
                let bg = if hovered { accent(4) } else { accent(3) };
                (bg, gpui::transparent_black(), None)
            }
        }
    };

    TextFieldPalette {
        background,
        foreground,
        border,
        placeholder,
        icon: placeholder,
        selection_background,
        selection_foreground,
        caret: foreground,
        shadow,
        typography: typography_for_size(ControlSize::Md),
        font_family: SharedString::from("System UI"),
    }
}

fn alpha(color: gpui::Hsla, a: f32) -> gpui::Hsla {
    gpui::Hsla { a, ..color }
}

/// Radix `--shadow-1` approximation: hairline rim + soft top fade.
fn classic_field_shadow(edge: gpui::Hsla, fade: gpui::Hsla) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: edge,
            inset: true,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.5)),
            blur_radius: px(2.0),
            spread_radius: px(0.0),
            color: fade,
            inset: true,
        },
    ]
}

fn typography_for_size(size: ControlSize) -> LumaTextStyle {
    match size {
        ControlSize::Sm => LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
        ControlSize::Md => LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        ControlSize::Lg => LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::NORMAL },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_border_uses_focus_role() {
        let look = Arc::new(RadixLook::built_in());
        let theme = textfield_theme(look.clone());
        let palette = theme.resolve(
            TextFieldVariant::Standard,
            TextFieldState { focused: true, focus_visible: true, ..Default::default() },
            true,
        );
        let expected = look.resolve_role(SemanticRole::Focus).hsla();
        assert_eq!(palette.border.l, expected.l);
    }

    #[test]
    fn soft_uses_accent_tint_background() {
        let look = Arc::new(RadixLook::built_in());
        let theme = textfield_theme_with(Arc::clone(&look), RadixTextFieldVariant::Soft);
        let palette = theme.resolve(TextFieldVariant::Standard, TextFieldState::default(), true);
        assert_eq!(palette.background, look.resolve_step(ScaleFamily::Color, 3).hsla());
    }

    #[test]
    fn classic_uses_shadow_not_gray_border() {
        let look = Arc::new(RadixLook::built_in());
        let classic = textfield_theme_with(Arc::clone(&look), RadixTextFieldVariant::Classic).resolve(
            TextFieldVariant::Standard,
            TextFieldState::default(),
            true,
        );
        let surface = textfield_theme_with(Arc::clone(&look), RadixTextFieldVariant::Surface).resolve(
            TextFieldVariant::Standard,
            TextFieldState::default(),
            true,
        );

        assert_eq!(classic.background, surface.background);
        assert!(classic.shadow.is_some(), "classic carries shadow-1");
        assert!(surface.shadow.is_none(), "surface stays flat");
        assert_ne!(classic.border.a, surface.border.a);
    }
}
