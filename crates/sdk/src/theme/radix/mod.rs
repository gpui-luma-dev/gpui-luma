mod action;
mod autocomplete;
mod button;
mod catalog;
mod checkbox;
mod controls;
mod color;
mod context_menu;
mod control_group;
mod css;
mod floating_menu;
mod focus;
mod listbox;
mod mode;
mod navigation_sidebar;
mod palette;
pub mod prelude;
mod popup_menu;
mod progress;
mod radio;
mod resolve;
mod scrollbar;
mod selection_panel;
mod selector;
mod selector_items_panel;
mod slider;
mod switch;
mod tabs_navigation;
mod templates;
mod textfield;
mod textarea;
mod usage;

pub use controls::{
    RadixButtonStyleExt, RadixCheckboxStyleExt, RadixSwitchStyleExt, RadixTextFieldExt, RadixThemeControlExt,
};
pub use usage::all_radix_theme_usages;

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

pub(crate) use button::button_appearance as resolve_button_appearance;
pub use button::RadixButtonStyle;
pub use catalog::{CssTokenCatalog, CssTokenMap, parse_css_catalog};
pub use mode::RadixModeTokens;
pub use palette::{RadixActionRole, RadixPalette};

use crate::controls::button_family::{ButtonFamilyAppearance, ButtonFamilyRole};
use crate::theme::pack::LumaChrome;
use crate::theme::{ControlSize, InteractionState, LumaTheme, ThemeMode};

#[derive(Clone)]
pub struct RadixTheme {
    state: Arc<RadixThemeState>,
}

struct RadixThemeState {
    catalog: CssTokenCatalog,
    light: RadixModeTokens,
    dark: RadixModeTokens,
    mode: AtomicU8,
}

impl RadixTheme {
    pub fn native() -> Self {
        Self::from_theme(LumaTheme::native())
    }

    /// Loads a theme file. Legacy Luma-shaped TOML is converted once at the boundary.
    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        Ok(Self::from_theme(LumaTheme::from_toml_str(source)?))
    }

    /// Loads Radix palette tokens from tweakcn-style CSS (`:root` / `.dark` custom properties).
    pub fn from_css_str(source: &str) -> anyhow::Result<Self> {
        let catalog = css::parse_css_catalog(source)?;
        Ok(Self {
            state: Arc::new(RadixThemeState {
                light: RadixModeTokens::from_catalog(catalog.light_map())?,
                dark: RadixModeTokens::from_catalog(catalog.dark_map())?,
                catalog,
                mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
            }),
        })
    }

    pub fn from_css_path(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|err| anyhow::anyhow!("read radix theme css {}: {err}", path.as_ref().display()))?;
        Self::from_css_str(&source)
    }

    pub fn from_theme(theme: LumaTheme) -> Self {
        Self {
            state: Arc::new(RadixThemeState {
                catalog: CssTokenCatalog { light: Default::default(), dark: Default::default() },
                light: RadixModeTokens::from_luma_tokens(theme.mode(ThemeMode::Light)),
                dark: RadixModeTokens::from_luma_tokens(theme.mode(ThemeMode::Dark)),
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

    pub fn mode_tokens(&self) -> &RadixModeTokens {
        match self.mode() {
            ThemeMode::Light => &self.state.light,
            ThemeMode::Dark => &self.state.dark,
        }
    }

    pub fn light_tokens(&self) -> &RadixModeTokens {
        &self.state.light
    }

    pub fn dark_tokens(&self) -> &RadixModeTokens {
        &self.state.dark
    }

    pub fn catalog(&self) -> &CssTokenCatalog {
        &self.state.catalog
    }

    /// `true` when the theme was loaded from tweakcn/shadcn CSS (non-empty catalog).
    pub fn has_css_catalog(&self) -> bool {
        !self.state.light.catalog.tokens.is_empty() || !self.state.dark.catalog.tokens.is_empty()
    }

    pub fn token(&self, name: &str) -> Option<&str> {
        self.mode_tokens().catalog.get(name)
    }

    pub fn token_color(&self, name: &str) -> anyhow::Result<gpui::Hsla> {
        self.mode_tokens().catalog.color(name)
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

    pub fn resolve_primary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        resolve_button_appearance(self.mode_tokens(), RadixButtonStyle::Primary, role, size, state)
    }

    pub fn resolve_secondary_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        resolve_button_appearance(self.mode_tokens(), RadixButtonStyle::Secondary, role, size, state)
    }

    pub fn resolve_outline_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        resolve_button_appearance(self.mode_tokens(), RadixButtonStyle::Outline, role, size, state)
    }

    pub fn resolve_ghost_button(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        resolve_button_appearance(self.mode_tokens(), RadixButtonStyle::Ghost, role, size, state)
    }

    pub fn switch_template(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> Arc<dyn crate::controls::command::button::ButtonTemplate<bool>> {
        templates::switch_template(Arc::clone(self), style)
    }

    pub fn checkbox_template(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> Arc<dyn crate::controls::command::button::ButtonTemplate<bool>> {
        templates::checkbox_template(Arc::clone(self), style)
    }

    pub fn radio_button_template(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> Arc<dyn crate::controls::command::button::ButtonTemplate<bool>> {
        templates::radio_button_template(Arc::clone(self), style)
    }

    pub fn button_template(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> Arc<dyn crate::controls::command::button::ButtonTemplate<()>> {
        templates::button_template(Arc::clone(self), style)
    }

    pub fn slider_template(self: &Arc<Self>) -> Arc<dyn crate::controls::slider::SliderTemplate> {
        templates::slider_template(Arc::clone(self))
    }

    pub fn scrollbar_template(self: &Arc<Self>) -> Arc<dyn crate::controls::scrollbar::ScrollbarTemplate> {
        templates::scrollbar_template(Arc::clone(self))
    }

    pub fn floating_menu_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::floating_menu::FloatingMenuTheme> {
        templates::floating_menu_theme(Arc::clone(self))
    }

    pub fn popup_menu_template(self: &Arc<Self>) -> Arc<dyn crate::controls::popup_menu::PopupMenuTemplate> {
        templates::popup_menu_template(Arc::clone(self))
    }

    pub fn context_menu_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::context_menu::ContextMenuTheme> {
        templates::context_menu_theme(Arc::clone(self))
    }

    pub fn context_menu_template(self: &Arc<Self>) -> Arc<dyn crate::controls::context_menu::ContextMenuTemplate> {
        templates::context_menu_template(Arc::clone(self))
    }

    pub fn selector_template(self: &Arc<Self>) -> Arc<dyn crate::controls::selector::SelectorTemplate> {
        templates::selector_template(Arc::clone(self))
    }

    pub fn textfield_template(self: &Arc<Self>) -> Arc<dyn crate::controls::textfield::TextFieldTemplate> {
        templates::textfield_template(Arc::clone(self))
    }

    pub fn textfield_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::textfield::TextFieldTheme> {
        templates::textfield_theme(Arc::clone(self))
    }

    pub fn textarea_template(self: &Arc<Self>) -> Arc<dyn crate::controls::textarea::TextAreaTemplate> {
        templates::textarea_template(Arc::clone(self))
    }

    pub fn textarea_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::textarea::TextAreaTheme> {
        templates::textarea_theme(Arc::clone(self))
    }

    pub fn autocomplete_textbox_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn crate::controls::autocomplete::AutocompleteTextBoxTheme> {
        templates::autocomplete_textbox_theme(Arc::clone(self))
    }

    pub fn selector_items_panel_appearance(
        &self,
        size: crate::theme::ControlSize,
    ) -> crate::controls::selector_panel::SelectorItemsPanelAppearance {
        selector_items_panel::selector_items_panel_appearance(self.mode_tokens(), self.mode(), size)
    }

    pub fn selection_panel_appearance(
        &self,
        size: crate::theme::ControlSize,
    ) -> crate::controls::selection_panel::SelectionPanelAppearance {
        selection_panel::selection_panel_appearance(self.mode_tokens(), self.mode(), size)
    }

    pub fn selection_panel_appearance_provider(
        self: &Arc<Self>,
    ) -> crate::controls::selection_panel::SelectionPanelAppearanceProvider {
        templates::selection_panel_appearance_provider(Arc::clone(self))
    }

    pub fn tabs_navigation_template(
        self: &Arc<Self>,
    ) -> Arc<dyn crate::controls::tabs_navigation::TabsNavigationTemplate> {
        templates::tabs_navigation_template(Arc::clone(self))
    }

    pub fn tabs_navigation_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::tabs_navigation::TabsNavigationTheme> {
        templates::tabs_navigation_theme(Arc::clone(self))
    }

    pub fn navigation_sidebar_template(
        self: &Arc<Self>,
    ) -> Arc<dyn crate::controls::navigation_sidebar::NavigationSidebarTemplate> {
        templates::navigation_sidebar_template(Arc::clone(self))
    }

    pub fn navigation_sidebar_theme(
        self: &Arc<Self>,
    ) -> Arc<dyn crate::controls::navigation_sidebar::NavigationSidebarTheme> {
        templates::navigation_sidebar_theme(Arc::clone(self))
    }

    pub fn control_group_template<T>(self: &Arc<Self>) -> crate::controls::control_group::ControlGroupTemplate<T>
    where
        T: crate::controls::control_group::ControlGroupItemLike + Clone + Send + Sync + 'static,
    {
        templates::control_group_template(Arc::clone(self))
    }

    pub fn control_group_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::control_group::ControlGroupTheme> {
        templates::control_group_theme(Arc::clone(self))
    }

    pub fn listbox_template(
        self: &Arc<Self>,
    ) -> crate::controls::control_group::ControlGroupTemplate<crate::controls::listbox::ListBoxItem> {
        templates::listbox_template(Arc::clone(self))
    }

    pub fn listbox_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::listbox::ListBoxTheme> {
        templates::listbox_theme(Arc::clone(self))
    }

    pub fn radio_group_template<T>(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> crate::controls::control_group::ControlGroupTemplate<T>
    where
        T: crate::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            Arc::clone(self),
            style,
            crate::controls::radio_group::RadioGroupLayout::Vertical,
        )
    }

    pub fn radio_group_horizontal_template<T>(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> crate::controls::control_group::ControlGroupTemplate<T>
    where
        T: crate::controls::control_group::ControlGroupItemLike + 'static,
    {
        templates::radio_group_template(
            Arc::clone(self),
            style,
            crate::controls::radio_group::RadioGroupLayout::Horizontal,
        )
    }

    pub fn progress_template(self: &Arc<Self>) -> Arc<dyn crate::controls::progress::ProgressTemplate> {
        templates::progress_template(Arc::clone(self))
    }

    pub fn progress_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::progress::ProgressTheme> {
        templates::progress_theme(Arc::clone(self))
    }

    pub fn toggle_template(
        self: &Arc<Self>,
        style: RadixButtonStyle,
    ) -> Arc<dyn crate::controls::command::button::ButtonTemplate<bool>> {
        templates::toggle_template(Arc::clone(self), style)
    }

    pub fn button_family_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::button_family::ButtonFamilyTheme> {
        templates::button_family_theme(Arc::clone(self))
    }

    pub fn checkbox_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::checkbox::CheckboxTheme> {
        templates::checkbox_theme(Arc::clone(self))
    }

    pub fn switch_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::switch::SwitchTheme> {
        templates::switch_theme(Arc::clone(self))
    }

    pub fn radio_button_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::radio_button::RadioButtonTheme> {
        templates::radio_button_theme(Arc::clone(self))
    }

    pub fn slider_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::slider::SliderTheme> {
        templates::slider_theme(Arc::clone(self))
    }

    pub fn scrollbar_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::scrollbar::ScrollbarTheme> {
        templates::scrollbar_theme(Arc::clone(self))
    }

    pub fn selector_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::selector::SelectorTheme> {
        templates::selector_theme(Arc::clone(self))
    }

    pub fn popup_menu_theme(self: &Arc<Self>) -> Arc<dyn crate::controls::popup_menu::PopupMenuTheme> {
        templates::popup_menu_theme(Arc::clone(self))
    }
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
