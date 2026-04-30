use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use gpui::{Hsla, Styled};

use crate::controls::{
    checkbox::ThemedCheckboxTemplate,
    command::button::{ButtonTemplate, DefaultButtonTemplate},
    navigation_sidebar::{NavigationSidebarTemplate, ThemedNavigationSidebarTemplate},
    popup_menu::{PopupMenuTemplate, ThemedPopupMenuTemplate},
    progress::{ProgressTemplate, ThemedProgressTemplate},
    radio_button::ThemedRadioButtonTemplate,
    scrollbar::{ScrollbarOrientation, ScrollbarTemplate, ThemedScrollbarTemplate},
    slider::{SliderTemplate, ThemedSliderTemplate},
    switch::ThemedSwitchTemplate,
    tabs_navigation::{TabsNavigationTemplate, ThemedTabsNavigationTemplate},
    textarea::{TextAreaTemplate, ThemedTextAreaTemplate},
    textfield::{TextFieldTemplate, ThemedTextFieldTemplate},
};
use crate::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, CheckboxAppearance, CheckboxTheme,
    ContextMenuAppearance, ContextMenuTheme, DefaultButtonFamilyTheme, DefaultCheckboxTheme, DefaultContextMenuTheme,
    DefaultFloatingMenuTheme, DefaultPopupMenuTheme, DefaultProgressTheme, DefaultRadioButtonTheme,
    DefaultScrollbarTheme, DefaultSliderTheme, DefaultSwitchTheme, DefaultTabsNavigationTheme, DefaultTextAreaTheme,
    DefaultTextFieldTheme, InteractionState, LumaTheme, NavigationSidebarTheme, PopupMenuAppearance, PopupMenuTheme,
    ProgressAppearance, ProgressTheme, RadioButtonAppearance, RadioButtonTheme, ScrollbarAppearance, ScrollbarTheme,
    SliderAppearance, SliderTheme, SwitchAppearance, SwitchTheme, TabsNavigationItemAppearance,
    TabsNavigationListAppearance, TabsNavigationTheme, TextAreaAppearance, TextAreaTheme, TextFieldAppearance,
    TextFieldTheme, ThemeMode, ThemeTokens,
};

#[derive(Clone)]
pub struct LumaThemePack {
    state: Arc<LumaThemeState>,
    live_theme: Arc<LumaLiveTheme>,
}

struct LumaThemeState {
    theme: LumaTheme,
    mode: AtomicU8,
}

#[derive(Clone)]
struct LumaLiveTheme {
    state: Arc<LumaThemeState>,
}

#[derive(Clone, Copy)]
pub struct LumaChrome {
    pub app_background: Hsla,
    pub content_background: Hsla,
    pub title_text: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub border: Hsla,
    pub panel_background: Hsla,
}

impl LumaThemePack {
    pub fn new() -> Self {
        let state =
            Arc::new(LumaThemeState { theme: LumaTheme::native(), mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)) });

        Self { state: state.clone(), live_theme: Arc::new(LumaLiveTheme { state }) }
    }

    pub fn chrome(&self) -> LumaChrome {
        let tokens = self.state.tokens();
        let palette = &tokens.palette;

        LumaChrome {
            app_background: palette.app.background,
            content_background: palette.app.background,
            title_text: palette.app.foreground,
            body_text: palette.surface.subtle.foreground,
            muted_text: palette.app.muted_foreground,
            border: palette.border.default,
            panel_background: palette.surface.panel.background,
        }
    }

    pub fn tokens(&self) -> ThemeTokens {
        self.state.tokens()
    }

    pub fn mode(&self) -> ThemeMode {
        self.state.current_mode()
    }

    pub fn toggle_mode(&self) -> ThemeMode {
        let next = match self.mode() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.state.set_mode(next);
        next
    }

    pub fn theme_name(&self) -> &str {
        &self.state.theme.name
    }

    pub fn theme_version(&self) -> u32 {
        self.state.theme.version
    }

    pub fn button_family_theme(&self) -> Arc<dyn ButtonFamilyTheme> {
        self.live_theme.clone()
    }

    pub fn button_template(&self) -> Arc<dyn ButtonTemplate<()>> {
        Arc::new(DefaultButtonTemplate::new(self.live_theme.clone()))
    }

    pub fn toggle_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        let button_family_theme = self.live_theme.clone();
        Arc::new(DefaultButtonTemplate::new(self.live_theme.clone()).with_modifier(move |element, model| {
            let variant = match model.kind {
                crate::controls::button_family::ButtonKind::Standard => ButtonVariant::Standard,
                crate::controls::button_family::ButtonKind::Ghost => ButtonVariant::Ghost,
                crate::controls::button_family::ButtonKind::Prominent => ButtonVariant::Prominent,
            };
            let appearance = ButtonFamilyTheme::resolve(
                button_family_theme.as_ref(),
                variant,
                ButtonFamilyRole::Toggle { selected: model.data },
                model.size,
                model.state,
            );
            element.bg(appearance.background).text_color(appearance.foreground).border_color(appearance.border)
        }))
    }

    pub fn checkbox_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedCheckboxTemplate::new(self.live_theme.clone()))
    }

    pub fn switch_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedSwitchTemplate::new(self.live_theme.clone()))
    }

    pub fn radio_button_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedRadioButtonTemplate::new(self.live_theme.clone()))
    }

    pub fn slider_template(&self) -> Arc<dyn SliderTemplate> {
        Arc::new(ThemedSliderTemplate::new(self.live_theme.clone()))
    }

    pub fn scrollbar_template(&self) -> Arc<dyn ScrollbarTemplate> {
        Arc::new(ThemedScrollbarTemplate::new(self.live_theme.clone()))
    }

    pub fn textfield_template(&self) -> Arc<dyn TextFieldTemplate> {
        Arc::new(ThemedTextFieldTemplate::new(self.live_theme.clone()))
    }

    pub fn textfield_theme(&self) -> Arc<dyn TextFieldTheme> {
        self.live_theme.clone()
    }

    pub fn textarea_template(&self) -> Arc<dyn TextAreaTemplate> {
        Arc::new(ThemedTextAreaTemplate::new(self.live_theme.clone()))
    }

    pub fn textarea_theme(&self) -> Arc<dyn TextAreaTheme> {
        self.live_theme.clone()
    }

    pub fn popup_menu_template(&self) -> Arc<dyn PopupMenuTemplate> {
        Arc::new(ThemedPopupMenuTemplate::new(self.live_theme.clone()))
    }

    pub fn context_menu_theme(&self) -> Arc<dyn ContextMenuTheme> {
        self.live_theme.clone()
    }

    pub fn navigation_sidebar_theme(&self) -> Arc<dyn NavigationSidebarTheme> {
        self.live_theme.clone()
    }

    pub fn navigation_sidebar_template(&self) -> Arc<dyn NavigationSidebarTemplate> {
        Arc::new(ThemedNavigationSidebarTemplate::new_with_floating_menu_theme(
            self.live_theme.clone(),
            Arc::new(DefaultFloatingMenuTheme::new(self.state.tokens())),
        ))
    }

    pub fn tabs_navigation_template(&self) -> Arc<dyn TabsNavigationTemplate> {
        Arc::new(ThemedTabsNavigationTemplate::new(self.live_theme.clone()))
    }

    pub fn progress_template(&self) -> Arc<dyn ProgressTemplate> {
        Arc::new(ThemedProgressTemplate::new(self.live_theme.clone()))
    }
}

impl Default for LumaThemePack {
    fn default() -> Self {
        Self::new()
    }
}

impl LumaThemeState {
    fn current_mode(&self) -> ThemeMode {
        u8_to_mode(self.mode.load(Ordering::Relaxed))
    }

    fn set_mode(&self, mode: ThemeMode) {
        self.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    fn tokens(&self) -> ThemeTokens {
        self.theme.mode(self.current_mode()).clone()
    }
}

impl ButtonFamilyTheme for LumaLiveTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: crate::theme::ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        DefaultButtonFamilyTheme::new(self.state.tokens()).resolve(variant, role, size, state)
    }
}

impl CheckboxTheme for LumaLiveTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        DefaultCheckboxTheme::new(self.state.tokens()).resolve(checked, state)
    }
}

impl SwitchTheme for LumaLiveTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance {
        DefaultSwitchTheme::new(self.state.tokens()).resolve(on, state)
    }
}

impl RadioButtonTheme for LumaLiveTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> RadioButtonAppearance {
        DefaultRadioButtonTheme::new(self.state.tokens()).resolve(checked, state)
    }
}

impl SliderTheme for LumaLiveTheme {
    fn resolve(&self, state: InteractionState) -> SliderAppearance {
        DefaultSliderTheme::new(self.state.tokens()).resolve(state)
    }
}

impl ScrollbarTheme for LumaLiveTheme {
    fn resolve(&self, state: InteractionState, orientation: ScrollbarOrientation) -> ScrollbarAppearance {
        DefaultScrollbarTheme::new(self.state.tokens()).resolve(state, orientation)
    }
}

impl TextFieldTheme for LumaLiveTheme {
    fn resolve(&self, state: crate::controls::textfield::TextFieldState, enabled: bool) -> TextFieldAppearance {
        DefaultTextFieldTheme::new(self.state.tokens()).resolve(state, enabled)
    }
}

impl TextAreaTheme for LumaLiveTheme {
    fn resolve(&self, state: crate::controls::textarea::TextAreaState, enabled: bool) -> TextAreaAppearance {
        DefaultTextAreaTheme::new(self.state.tokens()).resolve(state, enabled)
    }
}

impl PopupMenuTheme for LumaLiveTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance {
        DefaultPopupMenuTheme::new(self.state.tokens()).resolve(state)
    }
}

impl NavigationSidebarTheme for LumaLiveTheme {
    fn resolve_container(&self) -> crate::theme::NavigationSidebarContainerAppearance {
        crate::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_container()
    }

    fn resolve_section(&self) -> crate::theme::NavigationSidebarSectionAppearance {
        crate::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_section()
    }

    fn resolve_branch(
        &self,
        state: InteractionState,
        size: crate::theme::ControlSize,
    ) -> crate::theme::NavigationSidebarItemAppearance {
        crate::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_branch(state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: crate::theme::ControlSize,
    ) -> crate::theme::NavigationSidebarItemAppearance {
        crate::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_item(selected, state, size)
    }
}

impl ContextMenuTheme for LumaLiveTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        DefaultContextMenuTheme::new(self.state.tokens()).resolve(state)
    }
}

impl ProgressTheme for LumaLiveTheme {
    fn resolve(&self, enabled: bool) -> ProgressAppearance {
        DefaultProgressTheme::new(self.state.tokens()).resolve(enabled)
    }
}

impl TabsNavigationTheme for LumaLiveTheme {
    fn resolve_list(&self, enabled: bool) -> TabsNavigationListAppearance {
        DefaultTabsNavigationTheme::new(self.state.tokens()).resolve_list(enabled)
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance {
        DefaultTabsNavigationTheme::new(self.state.tokens()).resolve_item(active, state)
    }
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn u8_to_mode(mode: u8) -> ThemeMode {
    match mode {
        1 => ThemeMode::Dark,
        _ => ThemeMode::Light,
    }
}
