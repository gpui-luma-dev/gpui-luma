use std::sync::Arc;

pub(crate) use gpui_luma_look_radix::Look;
use gpui_luma_look_radix::{CustomColors, SemanticRole};
use gpui_luma::theme::{LumaChrome, LumaTextStyle, LumaTypography, ThemeMode};
use gpui::{Styled, px};
use luma_app_common::{built_in_look, built_in_theme, built_in_themes};

fn available_theme_names() -> Vec<String> {
    built_in_themes().iter().map(|theme| theme.id.to_string()).collect()
}

/// Startup theme: `default` or a built-in CSS theme ID (e.g. `retro-arcade`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorVizThemeChoice {
    Default,
    Named(String),
}

impl ColorVizThemeChoice {
    pub fn from_args() -> Self {
        match std::env::args().nth(1).as_deref() {
            None => Self::Default,
            Some(arg) => Self::parse(arg).unwrap_or_else(|usage| {
                eprintln!("{usage}");
                std::process::exit(1);
            }),
        }
    }

    fn parse(arg: &str) -> Result<Self, String> {
        let stem = arg.trim().to_ascii_lowercase();
        if stem == "default" {
            return Ok(Self::Default);
        }

        if built_in_theme(&stem).is_some() {
            return Ok(Self::Named(stem));
        }

        let available = available_theme_names();
        let hint = if available.is_empty() {
            "no embedded built-in themes found".to_string()
        } else {
            format!("available: {}", available.join(", "))
        };

        Err(format!("unknown color-viz theme {arg:?} ({hint})\n{}", Self::usage_line()))
    }

    fn usage_line() -> String {
        let program = std::env::args().next().unwrap_or_else(|| "luma-color-viz".into());
        let mut options = vec!["default".to_string()];
        options.extend(available_theme_names());
        format!("usage: {program} [{}]", options.join("|"))
    }

    pub fn radix_look(self) -> Arc<Look> {
        let look = Look::built_in();
        // Existing named CSS themes seed Radix palettes in each mode.
        if let Self::Named(stem) = self {
            match built_in_look(&stem) {
                Ok(source) => {
                    for mode in [ThemeMode::Light, ThemeMode::Dark] {
                        source.set_mode(mode);
                        look.set_mode(mode);
                        let chrome = source.chrome();
                        let colors = CustomColors {
                            accent: gpui_luma::color::gpui_bridge::from_hsla(
                                source.token_color("primary").unwrap_or(chrome.body_text),
                            ),
                            gray: gpui_luma::color::gpui_bridge::from_hsla(chrome.muted_text),
                            background: gpui_luma::color::gpui_bridge::from_hsla(chrome.app_background),
                        };
                        if let Err(error) = look.set_custom_colors(colors) {
                            eprintln!("failed to seed Radix theme {stem}: {error}");
                        }
                    }
                }
                Err(error) => eprintln!("failed to load theme {stem}: {error}"),
            }
        }
        look.set_mode(ThemeMode::Dark);
        Arc::new(look)
    }
}

/// Typography and shell colors for this application, resolved from Radix roles.
pub(crate) trait ColorVizLookExt {
    fn chrome(&self) -> LumaChrome;
    fn typography_scale(&self, size: TextSize) -> LumaTextStyle;
    fn scrollbar_template(&self) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate>;
}

#[derive(Clone, Copy)]
pub(crate) enum TextSize {
    Xs,
    Sm,
    Base,
}

impl ColorVizLookExt for Look {
    fn chrome(&self) -> LumaChrome {
        let color = |role| self.resolve_role(role).hsla();
        LumaChrome {
            app_background: color(SemanticRole::Background),
            content_background: color(SemanticRole::Background),
            panel_background: color(SemanticRole::Surface),
            border: color(SemanticRole::Border),
            title_text: color(SemanticRole::Foreground),
            body_text: color(SemanticRole::Foreground),
            muted_text: color(SemanticRole::MutedForeground),
        }
    }

    fn typography_scale(&self, size: TextSize) -> LumaTextStyle {
        let scale = LumaTypography::default().text.scale;
        match size {
            TextSize::Xs => scale.xs,
            TextSize::Sm => scale.sm,
            TextSize::Base => scale.md,
        }
    }

    fn scrollbar_template(&self) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate> {
        Arc::new(gpui_luma::controls::scrollbar::ThemedScrollbarTemplate::new(Arc::new(ColorVizScrollbarTheme(
            self.clone(),
        ))))
    }
}

pub(crate) trait TypographyExt: Styled + Sized {
    fn typography_style(self, style: LumaTextStyle) -> Self {
        self.text_size(px(style.size)).line_height(px(style.line_height)).font_weight(style.weight)
    }
}
impl<T: Styled> TypographyExt for T {}

pub(crate) fn icon_button(
    id: impl Into<gpui::SharedString>,
    icon: lucide_svg_static::Icon,
) -> gpui_luma_look_radix::Button {
    gpui_luma_look_radix::Button::new(id)
        .role(gpui_luma::controls::button_family::ButtonFamilyRole::Icon)
        .icon(icon)
}

struct ColorVizScrollbarTheme(Look);
impl gpui_luma::controls::scrollbar::ScrollbarTheme for ColorVizScrollbarTheme {
    fn resolve(
        &self,
        state: gpui_luma::theme::InteractionState,
        orientation: gpui_luma::controls::scrollbar::ScrollbarOrientation,
        size: gpui_luma::theme::ControlSize,
        style: gpui_luma::controls::scrollbar::ScrollbarStyle,
    ) -> gpui_luma::controls::scrollbar::ScrollbarLook {
        use gpui_luma::controls::scrollbar::{DefaultScrollbarTheme};
        let mut resolved = DefaultScrollbarTheme::default().resolve(state, orientation, size, style);
        resolved.track_background = gpui::transparent_black();
        resolved.thumb_background = self
            .0
            .resolve_step(
                gpui_luma_look_radix::ScaleFamily::Gray,
                if state.pressed {
                    10
                } else if state.hovered {
                    9
                } else {
                    7
                },
            )
            .hsla();
        resolved
    }
    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.0.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_startup_theme_seeds_radix_in_both_modes() {
        let theme = built_in_themes().first().expect("embedded themes");
        let choice = ColorVizThemeChoice::parse(theme.id).expect("existing theme argument");
        let look = choice.radix_look();
        assert_eq!(look.mode(), ThemeMode::Dark);
        let source = built_in_look(theme.id).expect("source theme");
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            source.set_mode(mode);
            assert_eq!(
                look.custom_inputs().expect("Radix palette inputs").background,
                gpui_luma::color::gpui_bridge::from_hsla(source.chrome().app_background)
            );
        }
        assert!(ColorVizThemeChoice::parse("unknown-color-viz-theme").is_err());
    }
}
