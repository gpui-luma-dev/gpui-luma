use gpui::Hsla;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeUsage {
    pub label: &'static str,
    pub parts: &'static [ThemePartUsage],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePartUsage {
    pub part: &'static str,
    pub token: &'static str,
    pub states: &'static [&'static str],
    pub appearance_fields: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaletteColorToken {
    pub token: &'static str,
    pub color: Hsla,
    pub reserved: bool,
    pub gallery_chrome: bool,
}

pub fn palette_color_tokens(tokens: &super::ThemeTokens) -> Vec<PaletteColorToken> {
    let palette = &tokens.palette;

    vec![
        gallery_chrome_token("app.background", palette.app.background),
        gallery_chrome_token("app.foreground", palette.app.foreground),
        gallery_chrome_token("app.muted_foreground", palette.app.muted_foreground),
        gallery_chrome_token("surface.panel.background", palette.surface.panel.background),
        token("surface.panel.foreground", palette.surface.panel.foreground),
        token("surface.panel.border", palette.surface.panel.border),
        token("surface.floating.background", palette.surface.floating.background),
        token("surface.floating.foreground", palette.surface.floating.foreground),
        token("surface.floating.border", palette.surface.floating.border),
        token("surface.subtle.background", palette.surface.subtle.background),
        gallery_chrome_token("surface.subtle.foreground", palette.surface.subtle.foreground),
        token("action.prominent.background", palette.action.prominent.background),
        token("action.prominent.foreground", palette.action.prominent.foreground),
        token("action.prominent.hover_background", palette.action.prominent.hover_background),
        token("action.prominent.pressed_background", palette.action.prominent.pressed_background),
        token("action.prominent.border", palette.action.prominent.border),
        token("action.subtle.background", palette.action.subtle.background),
        token("action.subtle.foreground", palette.action.subtle.foreground),
        token("action.subtle.hover_background", palette.action.subtle.hover_background),
        token("action.subtle.pressed_background", palette.action.subtle.pressed_background),
        token("action.subtle.border", palette.action.subtle.border),
        token("action.standard.background", palette.action.standard.background),
        token("action.standard.foreground", palette.action.standard.foreground),
        token("action.standard.hover_background", palette.action.standard.hover_background),
        token("action.standard.pressed_background", palette.action.standard.pressed_background),
        token("action.standard.border", palette.action.standard.border),
        token("action.ghost.background", palette.action.ghost.background),
        token("action.ghost.foreground", palette.action.ghost.foreground),
        token("action.ghost.hover_background", palette.action.ghost.hover_background),
        token("action.ghost.pressed_background", palette.action.ghost.pressed_background),
        token("action.ghost.border", palette.action.ghost.border),
        token("state.hover.background", palette.state.hover.background),
        token("state.hover.foreground", palette.state.hover.foreground),
        token("state.pressed.background", palette.state.pressed.background),
        token("state.selected.background", palette.state.selected.background),
        token("state.selected.foreground", palette.state.selected.foreground),
        token("state.disabled.background", palette.state.disabled.background),
        token("state.disabled.foreground", palette.state.disabled.foreground),
        token("form.input.background", palette.form.input.background),
        reserved_token("form.input.foreground", palette.form.input.foreground),
        token("form.input.border", palette.form.input.border),
        token("form.input.invalid_border", palette.form.input.invalid_border),
        reserved_token("form.input.placeholder", palette.form.input.placeholder),
        token("focus.ring", palette.focus.ring),
        gallery_chrome_token("border.default", palette.border.default),
        reserved_token("border.strong", palette.border.strong),
        token("navigation.background", palette.navigation.background),
        token("navigation.foreground", palette.navigation.foreground),
        token("navigation.muted_foreground", palette.navigation.muted_foreground),
        token("navigation.hover_background", palette.navigation.hover_background),
        token("navigation.selected_background", palette.navigation.selected_background),
        token("navigation.selected_foreground", palette.navigation.selected_foreground),
        token("navigation.border", palette.navigation.border),
        reserved_token("data.accent_1", palette.data.accent_1),
        reserved_token("data.accent_2", palette.data.accent_2),
        reserved_token("data.accent_3", palette.data.accent_3),
        reserved_token("data.accent_4", palette.data.accent_4),
        reserved_token("data.accent_5", palette.data.accent_5),
    ]
}

pub fn resolve_palette_color(tokens: &super::ThemeTokens, token_name: &str) -> Option<Hsla> {
    palette_color_tokens(tokens)
        .into_iter()
        .find(|token| token.token == token_name)
        .map(|token| token.color)
}

fn token(token: &'static str, color: Hsla) -> PaletteColorToken {
    PaletteColorToken { token, color, reserved: false, gallery_chrome: false }
}

fn gallery_chrome_token(token: &'static str, color: Hsla) -> PaletteColorToken {
    PaletteColorToken { token, color, reserved: false, gallery_chrome: true }
}

fn reserved_token(token: &'static str, color: Hsla) -> PaletteColorToken {
    PaletteColorToken { token, color, reserved: true, gallery_chrome: false }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::palette_color_tokens;
    use crate::theme::ThemeTokens;

    #[test]
    fn palette_token_names_are_unique() {
        let tokens = ThemeTokens::default();
        let palette_tokens = palette_color_tokens(&tokens);
        let mut names = HashSet::new();

        for token in &palette_tokens {
            assert!(names.insert(token.token), "duplicate palette token in registry: {}", token.token);
        }
    }

    #[test]
    fn registry_includes_action_standard_tokens() {
        let tokens = ThemeTokens::default();
        let palette_tokens = palette_color_tokens(&tokens);

        for token_name in [
            "action.subtle.background",
            "action.subtle.foreground",
            "action.subtle.hover_background",
            "action.subtle.pressed_background",
            "action.standard.background",
            "action.standard.foreground",
            "action.standard.hover_background",
            "action.standard.pressed_background",
        ] {
            assert!(
                palette_tokens.iter().any(|token| token.token == token_name),
                "missing palette token in registry: {}",
                token_name
            );
        }
    }
}
