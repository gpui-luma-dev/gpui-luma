use gpui::Hsla;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeUsage {
    pub component: &'static str,
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

pub fn all_theme_usages() -> &'static [&'static ThemeUsage] {
    THEME_USAGES
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
        token("action.ghost.background", palette.action.ghost.background),
        token("action.ghost.foreground", palette.action.ghost.foreground),
        token("action.ghost.hover_background", palette.action.ghost.hover_background),
        token("action.ghost.pressed_background", palette.action.ghost.pressed_background),
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

const THEME_USAGES: &[&ThemeUsage] = &[
    &super::button_family::BUTTON_THEME_USAGE,
    &super::navigation_sidebar::NAVIGATION_SIDEBAR_THEME_USAGE,
    &super::button_family::ICON_BUTTON_THEME_USAGE,
    &super::button_family::TOGGLE_THEME_USAGE,
    &super::choice_group::CHOICE_GROUP_THEME_USAGE,
    &super::checkbox::CHECKBOX_THEME_USAGE,
    &super::radio_button::RADIO_BUTTON_THEME_USAGE,
    &super::switch::SWITCH_THEME_USAGE,
    &super::slider::SLIDER_THEME_USAGE,
    &super::scrollbar::SCROLLBAR_THEME_USAGE,
    &super::textarea::TEXTAREA_THEME_USAGE,
    &super::textfield::TEXTFIELD_THEME_USAGE,
    &super::floating_menu::FLOATING_MENU_THEME_USAGE,
    &super::popup_menu::POPUP_MENU_THEME_USAGE,
    &super::context_menu::CONTEXT_MENU_THEME_USAGE,
    &super::tabs_navigation::TABS_NAVIGATION_THEME_USAGE,
    &super::progress::PROGRESS_THEME_USAGE,
];

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
    use super::{all_theme_usages, resolve_palette_color};
    use crate::theme::ThemeTokens;

    #[test]
    fn usage_metadata_tokens_are_known_palette_tokens() {
        let tokens = ThemeTokens::default();

        for usage in all_theme_usages() {
            for part in usage.parts {
                assert!(
                    resolve_palette_color(&tokens, part.token).is_some(),
                    "{} uses unknown palette token {}",
                    usage.component,
                    part.token
                );
            }
        }
    }
}
