use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use gpui::{BoxShadow, Hsla, SharedString};
use gpui_luma::controls::button_family::{ButtonFamilyAppearance, ButtonFamilyRole};
use gpui_luma::theme::pack::LumaChrome;
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, LumaTheme, ThemeMode};

use crate::catalog::{CssTokenCatalog, CssTokenMap, parse_css_catalog};
use crate::controls::ShadcnButtonStyle;
use crate::controls::{button_appearance, selection_panel_appearance, selector_items_panel_appearance, templates};
use crate::mode::ShadcnModeTokens;

use crate::shadow::parse_shadow_token;
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnToken};

const GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
];

/// Runtime shadcn appearance resolver backed by parsed CSS token catalogs.
#[derive(Clone)]
pub struct ShadcnLook {
    state: Arc<ShadcnLookState>,
}

struct ShadcnLookState {
    catalog: CssTokenCatalog,
    light: ShadcnModeTokens,
    dark: ShadcnModeTokens,
    mode: AtomicU8,
}

impl ShadcnLook {
    pub fn native() -> Self {
        Self::from_theme(LumaTheme::native())
    }

    /// Loads a theme file. Legacy Luma-shaped TOML is converted once at the boundary.
    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        Ok(Self::from_theme(LumaTheme::from_toml_str(source)?))
    }

    /// Loads shadcn palette tokens from tweakcn-style CSS (`:root` / `.dark` custom properties).
    pub fn from_css_str(source: &str) -> anyhow::Result<Self> {
        let catalog = parse_css_catalog(source)?;
        Ok(Self {
            state: Arc::new(ShadcnLookState {
                light: ShadcnModeTokens::from_catalog(catalog.light_map(), ThemeMode::Light)?,
                dark: ShadcnModeTokens::from_catalog(catalog.dark_map(), ThemeMode::Dark)?,
                catalog,
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
            }),
        })
    }

    pub fn from_css_path(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|err| anyhow::anyhow!("read shadcn theme css {}: {err}", path.as_ref().display()))?;
        Self::from_css_str(&source)
    }

    pub fn from_theme(theme: LumaTheme) -> Self {
        Self {
            state: Arc::new(ShadcnLookState {
                catalog: CssTokenCatalog { light: Default::default(), dark: Default::default() },
                light: ShadcnModeTokens::from_luma_tokens(theme.mode(ThemeMode::Light), ThemeMode::Light),
                dark: ShadcnModeTokens::from_luma_tokens(theme.mode(ThemeMode::Dark), ThemeMode::Dark),
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
            }),
        }
    }

    pub fn mode(&self) -> ThemeMode {
        u8_to_mode(self.state.mode.load(Ordering::Relaxed))
    }

    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    pub fn mode_tokens(&self) -> &ShadcnModeTokens {
        match self.mode() {
            ThemeMode::Light => &self.state.light,
            ThemeMode::Dark => &self.state.dark,
        }
    }

    pub fn catalog(&self) -> &CssTokenCatalog {
        &self.state.catalog
    }

    /// `true` when the look was loaded from tweakcn/shadcn CSS (non-empty catalog).
    pub fn has_css_catalog(&self) -> bool {
        !self.state.light.catalog.tokens.is_empty() || !self.state.dark.catalog.tokens.is_empty()
    }

    pub fn token(&self, name: &str) -> Option<&str> {
        self.mode_tokens().catalog.get(name)
    }

    /// Resolves colors directly from the loaded palette.
    pub fn color(&self, token: ShadcnToken) -> Hsla {
        self.mode_tokens().palette.token_color(token)
    }

    /// Dynamically resolves a token color for a specific interaction state.
    ///
    /// Explicit CSS overrides (e.g. `--primary-hover`) take precedence; otherwise
    /// mode-aware Oklch lightness shifts are applied.
    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> Hsla {
        self.mode_tokens().resolve_color_state(token, layer)
    }

    /// Resolves fonts to their loaded families.
    pub fn font(&self, role: ShadcnFont) -> SharedString {
        let key = match role {
            ShadcnFont::Sans => "font-sans",
            ShadcnFont::Serif => "font-serif",
            ShadcnFont::Mono => "font-mono",
        };
        self.token(key).map(first_font_family).unwrap_or_else(|| match role {
            ShadcnFont::Sans => SharedString::from("sans-serif"),
            ShadcnFont::Serif => SharedString::from("serif"),
            ShadcnFont::Mono => SharedString::from("monospace"),
        })
    }

    /// Resolves border-radius to raw pixels based on the current theme radius.
    pub fn radius(&self, role: ShadcnRadius) -> f32 {
        let base_radius = self.parse_pixel_token("radius").unwrap_or(8.0);
        match role {
            ShadcnRadius::None => 0.0,
            ShadcnRadius::Sm => (base_radius - 4.0).max(0.0),
            ShadcnRadius::Md => (base_radius - 2.0).max(0.0),
            ShadcnRadius::Lg => base_radius,
            ShadcnRadius::Xl => base_radius + 4.0,
        }
    }

    /// Resolves custom shadow structures for GPUI box-shadow arrays.
    pub fn shadow(&self, role: ShadcnShadow) -> Vec<BoxShadow> {
        let shadow_key = match role {
            ShadcnShadow::None => return vec![],
            ShadcnShadow::TwoXs => "shadow-2xs",
            ShadcnShadow::Xs => "shadow-xs",
            ShadcnShadow::Sm => "shadow-sm",
            ShadcnShadow::Default => "shadow",
            ShadcnShadow::Md => "shadow-md",
            ShadcnShadow::Lg => "shadow-lg",
            ShadcnShadow::Xl => "shadow-xl",
            ShadcnShadow::TwoXl => "shadow-2xl",
        };

        self.parse_shadow_token(shadow_key).unwrap_or_default()
    }

    pub fn parse_pixel_token(&self, name: &str) -> Option<f32> {
        self.token(name).and_then(parse_length_px)
    }

    pub fn parse_shadow_token(&self, token_key: &str) -> anyhow::Result<Vec<BoxShadow>> {
        parse_shadow_token(&self.mode_tokens().catalog, token_key)
    }

    pub fn light_tokens(&self) -> &ShadcnModeTokens {
        &self.state.light
    }

    pub fn dark_tokens(&self) -> &ShadcnModeTokens {
        &self.state.dark
    }

    pub fn token_color(&self, name: &str) -> anyhow::Result<Hsla> {
        self.mode_tokens().catalog.color(name)
    }

    /// Returns a copy of this look with global color overrides applied to both mode catalogs.
    pub fn with_color_overrides(&self, overrides: &HashMap<String, Hsla>) -> Self {
        if overrides.is_empty() {
            return self.clone();
        }

        let light_catalog = apply_color_overrides_to_catalog(self.state.light.catalog.clone(), overrides);
        let dark_catalog = apply_color_overrides_to_catalog(self.state.dark.catalog.clone(), overrides);

        let Ok(light) = ShadcnModeTokens::from_catalog(light_catalog, ThemeMode::Light) else {
            return self.clone();
        };
        let Ok(dark) = ShadcnModeTokens::from_catalog(dark_catalog, ThemeMode::Dark) else {
            return self.clone();
        };

        Self {
            state: Arc::new(ShadcnLookState {
                catalog: self.state.catalog.clone(),
                light,
                dark,
                mode: AtomicU8::new(self.state.mode.load(Ordering::Relaxed)),
            }),
        }
    }

    pub fn chrome(&self) -> LumaChrome {
        let palette = &self.mode_tokens().palette;

        LumaChrome {
            app_background: palette.app_background,
            content_background: palette.app_background,
            title_text: palette.app_foreground,
            body_text: palette.body_text,
            muted_text: palette.app_muted_foreground,
            border: palette.border_default,
            panel_background: palette.panel_background,
        }
    }

    pub fn paging_toolbar_chrome(self: &Arc<Self>) -> gpui_luma::controls::list_view::PagingToolbarChrome {
        let look = Arc::clone(self);
        Arc::new(move || look.chrome())
    }

    pub fn resolve_primary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        button_appearance(self.mode_tokens(), ShadcnButtonStyle::Primary, role, size, state)
    }

    pub fn resolve_secondary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        button_appearance(self.mode_tokens(), ShadcnButtonStyle::Secondary, role, size, state)
    }

    pub fn resolve_outline_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        button_appearance(self.mode_tokens(), ShadcnButtonStyle::Outline, role, size, state)
    }

    pub fn resolve_ghost_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        button_appearance(self.mode_tokens(), ShadcnButtonStyle::Ghost, role, size, state)
    }

    pub fn switch_template(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn gpui_luma::controls::command::button::ButtonTemplate<bool>> {
        templates::switch_template(Arc::clone(self), style)
    }

    pub fn checkbox_template(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn gpui_luma::controls::command::button::ButtonTemplate<bool>> {
        templates::checkbox_template(Arc::clone(self), style)
    }

    pub fn radio_button_template(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn gpui_luma::controls::command::button::ButtonTemplate<bool>> {
        templates::radio_button_template(Arc::clone(self), style)
    }

    pub fn button_template(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn gpui_luma::controls::command::button::ButtonTemplate<()>> {
        templates::button_template(Arc::clone(self), style)
    }

    pub fn slider_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::slider::SliderTemplate> {
        templates::slider_template(Arc::clone(self))
    }

    pub fn scrollbar_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate> {
        templates::scrollbar_template(Arc::clone(self))
    }

    pub fn floating_menu_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::floating_menu::FloatingMenuTheme> {
        templates::floating_menu_theme(Arc::clone(self))
    }

    pub fn popup_menu_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::popup_menu::PopupMenuTemplate> {
        templates::popup_menu_template(Arc::clone(self))
    }

    pub fn context_menu_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::context_menu::ContextMenuTheme> {
        templates::context_menu_theme(Arc::clone(self))
    }

    pub fn context_menu_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::context_menu::ContextMenuTemplate> {
        templates::context_menu_template(Arc::clone(self))
    }

    pub fn selector_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::selector::SelectorTemplate> {
        templates::selector_template(Arc::clone(self))
    }

    pub fn textfield_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTemplate> {
        templates::textfield_template(Arc::clone(self))
    }

    pub fn textfield_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::textfield::TextFieldTheme> {
        templates::textfield_theme(Arc::clone(self))
    }

    pub fn textarea_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::textarea::TextAreaTemplate> {
        templates::textarea_template(Arc::clone(self))
    }

    pub fn textarea_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::textarea::TextAreaTheme> {
        templates::textarea_theme(Arc::clone(self))
    }

    pub fn autocomplete_textbox_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::autocomplete::AutocompleteTextBoxTheme> {
        templates::autocomplete_textbox_theme(Arc::clone(self))
    }

    pub fn selector_items_panel_appearance(
        &self,
        size: ControlSize,
    ) -> gpui_luma::controls::selector_panel::SelectorItemsPanelAppearance {
        selector_items_panel_appearance(self.mode_tokens(), self.mode(), size)
    }

    pub fn selection_panel_appearance(
        &self,
        size: ControlSize,
    ) -> gpui_luma::controls::selection_panel::SelectionPanelAppearance {
        selection_panel_appearance(self.mode_tokens(), self.mode(), size)
    }

    pub fn selection_panel_appearance_provider(
        self: &Arc<Self>,
    ) -> gpui_luma::controls::selection_panel::SelectionPanelAppearanceProvider {
        templates::selection_panel_appearance_provider(Arc::clone(self))
    }

    pub fn tabs_navigation_template(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::tabs_navigation::TabsNavigationTemplate> {
        templates::tabs_navigation_template(Arc::clone(self))
    }

    pub fn tabs_navigation_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::tabs_navigation::TabsNavigationTheme> {
        templates::tabs_navigation_theme(Arc::clone(self))
    }

    pub fn accordion_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::accordion::AccordionTemplate> {
        templates::accordion_template(Arc::clone(self))
    }

    pub fn accordion_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::accordion::AccordionTheme> {
        templates::accordion_theme(Arc::clone(self))
    }

    pub fn accordion(
        self: &Arc<Self>,
        id: impl Into<SharedString>,
    ) -> gpui_luma::controls::accordion::AccordionBuilder {
        gpui_luma::controls::accordion::new(id).template(self.accordion_template())
    }

    pub fn tree_view_template<T>(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::tree_view::TreeViewTemplate<T>>
    where
        T: Send + Sync + 'static,
    {
        templates::tree_view_template(Arc::clone(self))
    }

    pub fn tree_view_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::tree_view::TreeViewTheme> {
        templates::tree_view_theme(Arc::clone(self))
    }

    pub fn tree_view<T>(
        self: &Arc<Self>,
        id: impl Into<SharedString>,
    ) -> gpui_luma::controls::tree_view::TreeViewBuilder<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        gpui_luma::controls::tree_view::new(id).template(self.tree_view_template::<T>())
    }

    pub fn resizable_panels_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::resizable_panels::ResizablePanelsTheme> {
        templates::resizable_panels_theme(Arc::clone(self))
    }

    pub fn resizable_panels(
        self: &Arc<Self>,
        id: impl Into<SharedString>,
    ) -> gpui_luma::controls::resizable_panels::ResizablePanelsBuilder {
        gpui_luma::controls::resizable_panels::ResizablePanels::new(id).theme(self.resizable_panels_theme())
    }

    pub fn navigation_sidebar_template(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::navigation_sidebar::NavigationSidebarTemplate> {
        templates::navigation_sidebar_template(Arc::clone(self))
    }

    pub fn navigation_sidebar_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn gpui_luma::controls::navigation_sidebar::NavigationSidebarTheme> {
        templates::navigation_sidebar_theme(Arc::clone(self))
    }

    pub fn control_group_template<T>(self: &Arc<Self>) -> gpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: gpui_luma::controls::control_group::ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        templates::control_group_template(Arc::clone(self))
    }

    pub fn control_group_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::control_group::ControlGroupTheme> {
        templates::control_group_theme(Arc::clone(self))
    }

    pub fn listbox_template(
        self: &Arc<Self>,
    ) -> gpui_luma::controls::control_group::ControlGroupTemplate<gpui_luma::controls::listbox::ListBoxItem> {
        templates::listbox_template(Arc::clone(self))
    }

    pub fn listbox_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::listbox::ListBoxTheme> {
        templates::listbox_theme(Arc::clone(self))
    }

    pub fn list_view_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::list_view::ListViewTemplate> {
        templates::list_view_template(Arc::clone(self))
    }

    pub fn list_view_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::list_view::ListViewTheme> {
        templates::list_view_theme(Arc::clone(self))
    }

    pub fn radio_group_template<T>(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> gpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: gpui_luma::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            Arc::clone(self),
            style,
            gpui_luma::controls::radio_group::RadioGroupLayout::Vertical,
        )
    }

    pub fn radio_group_horizontal_template<T>(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> gpui_luma::controls::control_group::ControlGroupTemplate<T>
    where
        T: gpui_luma::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            Arc::clone(self),
            style,
            gpui_luma::controls::radio_group::RadioGroupLayout::Horizontal,
        )
    }

    pub fn progress_template(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::progress::ProgressTemplate> {
        templates::progress_template(Arc::clone(self))
    }

    pub fn progress_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::progress::ProgressTheme> {
        templates::progress_theme(Arc::clone(self))
    }

    pub fn toggle_template(
        self: &Arc<Self>,
        style: ShadcnButtonStyle,
    ) -> Arc<dyn gpui_luma::controls::command::button::ButtonTemplate<bool>> {
        templates::toggle_template(Arc::clone(self), style)
    }

    pub fn button_family_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::button_family::ButtonFamilyTheme> {
        templates::button_family_theme(Arc::clone(self))
    }

    pub fn checkbox_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::checkbox::CheckboxTheme> {
        templates::checkbox_theme(Arc::clone(self))
    }

    pub fn switch_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::switch::SwitchTheme> {
        templates::switch_theme(Arc::clone(self))
    }

    pub fn radio_button_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::radio_button::RadioButtonTheme> {
        templates::radio_button_theme(Arc::clone(self))
    }

    pub fn slider_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::slider::SliderTheme> {
        templates::slider_theme(Arc::clone(self))
    }

    pub fn scrollbar_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTheme> {
        templates::scrollbar_theme(Arc::clone(self))
    }

    pub fn selector_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::selector::SelectorTheme> {
        templates::selector_theme(Arc::clone(self))
    }

    pub fn popup_menu_theme(self: &Arc<Self>) -> Arc<dyn gpui_luma::controls::popup_menu::PopupMenuTheme> {
        templates::popup_menu_theme(Arc::clone(self))
    }
}

fn apply_color_overrides_to_catalog(mut catalog: CssTokenMap, overrides: &HashMap<String, Hsla>) -> CssTokenMap {
    for (token, color) in overrides {
        let key = token.strip_prefix("--").unwrap_or(token.as_str()).to_string();
        catalog.tokens.insert(key, hsla_to_css_value(*color));
    }
    catalog
}

fn hsla_to_css_value(color: Hsla) -> String {
    format!("hsl({} {}% {}%)", (color.h * 360.0).round(), (color.s * 100.0).round(), (color.l * 100.0).round())
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn u8_to_mode(value: u8) -> ThemeMode {
    match value {
        0 => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

fn normalize_font_family(family: &str) -> SharedString {
    if family.eq_ignore_ascii_case("Rajdhani") {
        return SharedString::from("Rajdhani Variable");
    }
    SharedString::from(family.to_string())
}

fn first_font_family(raw: &str) -> SharedString {
    for part in raw.split(',') {
        let candidate = part.trim().trim_matches(['\'', '"']);
        if candidate.is_empty() {
            continue;
        }
        if GENERIC_FAMILIES.iter().any(|generic| candidate.eq_ignore_ascii_case(generic)) {
            continue;
        }
        return normalize_font_family(candidate);
    }

    normalize_font_family(raw.split(',').next().unwrap_or(raw).trim().trim_matches(['\'', '"']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_look_resolves_primary_color() {
        let look = ShadcnLook::native();
        let primary = look.color(ShadcnToken::Primary);
        assert!(primary.a > 0.0);
    }

    #[test]
    fn radius_scales_from_base_token() {
        let look = ShadcnLook::native();
        assert!(look.radius(ShadcnRadius::Lg) >= look.radius(ShadcnRadius::Sm));
    }
}
