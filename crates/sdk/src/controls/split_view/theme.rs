use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitViewLook {
    pub separator: Hsla,
    pub separator_hover: Hsla,
}

pub trait SplitViewTheme: Send + Sync {
    fn resolve(&self, hovered: bool, enabled: bool) -> SplitViewLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSplitViewTheme {
    tokens: ThemeTokens,
}

pub fn default_split_view_theme() -> Arc<dyn SplitViewTheme> {
    static THEME: OnceLock<Arc<dyn SplitViewTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultSplitViewTheme::default())).clone()
}

impl DefaultSplitViewTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl SplitViewTheme for DefaultSplitViewTheme {
    fn resolve(&self, _hovered: bool, enabled: bool) -> SplitViewLook {
        let palette = &self.tokens.palette;
        let separator = if enabled {
            palette.border.default
        } else {
            palette.state.disabled.background
        };
        let separator_hover = if enabled {
            palette.border.strong
        } else {
            palette.state.disabled.foreground
        };

        SplitViewLook { separator, separator_hover }
    }
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::{DefaultSplitViewTheme, SplitViewTheme};

    #[test]
    fn default_theme_uses_stronger_border_when_hovered_path_is_resolved() {
        let theme = DefaultSplitViewTheme::default();
        let idle = theme.resolve(false, true);
        let hover = theme.resolve(true, true);

        assert_ne!(idle.separator, hover.separator_hover);
        assert_eq!(hover.separator_hover, theme.tokens.palette.border.strong);
    }

    #[test]
    fn disabled_theme_mutes_separator_colors() {
        let theme = DefaultSplitViewTheme::default();
        let enabled = theme.resolve(false, true);
        let disabled = theme.resolve(false, false);

        assert_ne!(enabled.separator, disabled.separator);
        assert_eq!(disabled.separator, theme.tokens.palette.state.disabled.background);
    }

    #[test]
    fn disabled_theme_does_not_panic_with_default_tokens() {
        let theme = DefaultSplitViewTheme::default();
        let appearance = theme.resolve(false, false);
        assert!(appearance.separator.a >= 0.0);
        assert!(appearance.separator_hover.a >= 0.0);
        let _ = hsla(0.0, 0.0, 0.0, appearance.separator.a);
    }
}
