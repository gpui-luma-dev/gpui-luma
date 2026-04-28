use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use gpui::{Hsla, Styled};
use gpui_luma::controls::{
    checkbox::ThemedCheckboxTemplate,
    command::button::{ButtonTemplate, DefaultButtonTemplate},
    navigation_sidebar::{NavigationSidebarTemplate, ThemedNavigationSidebarTemplate},
    popup_menu::{PopupMenuTemplate, ThemedPopupMenuTemplate},
    progress::{ProgressTemplate, ThemedProgressTemplate},
    radio_button::ThemedRadioButtonTemplate,
    radio_group::{RadioGroupTemplate, ThemedRadioGroupTemplate},
    scrollbar::{ScrollbarOrientation, ScrollbarTemplate, ThemedScrollbarTemplate},
    slider::{SliderTemplate, ThemedSliderTemplate},
    switch::ThemedSwitchTemplate,
    tabs_navigation::{TabsNavigationTemplate, ThemedTabsNavigationTemplate},
    textarea::{TextAreaTemplate, ThemedTextAreaTemplate},
    textfield::{TextFieldTemplate, ThemedTextFieldTemplate},
    toggle_group::{ThemedToggleGroupTemplate, ToggleGroupTemplate},
};
use gpui_luma::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, CheckboxAppearance, CheckboxTheme,
    ContextMenuAppearance, ContextMenuTheme, DefaultButtonFamilyTheme, DefaultCheckboxTheme, DefaultContextMenuTheme,
    DefaultFloatingMenuTheme, DefaultPopupMenuTheme, DefaultProgressTheme, DefaultRadioButtonTheme,
    DefaultRadioGroupTheme, DefaultScrollbarTheme, DefaultSliderTheme, DefaultSwitchTheme, DefaultTabsNavigationTheme,
    DefaultTextAreaTheme, DefaultTextFieldTheme, DefaultToggleGroupTheme, InteractionState, LumaTheme,
    NavigationSidebarTheme, PopupMenuAppearance, PopupMenuTheme, ProgressAppearance, ProgressTheme,
    RadioButtonAppearance, RadioButtonTheme, RadioGroupItemAppearance, RadioGroupTheme, ScrollbarAppearance,
    ScrollbarTheme, SliderAppearance, SliderTheme, SwitchAppearance, SwitchTheme, TabsNavigationItemAppearance,
    TabsNavigationListAppearance, TabsNavigationTheme, TextAreaAppearance, TextAreaTheme, TextFieldAppearance,
    TextFieldTheme, ThemeMode, ThemeTokens, ToggleGroupItemAppearance, ToggleGroupListAppearance, ToggleGroupTheme,
};

#[derive(Clone)]
pub(in crate::gallery) struct GalleryThemePack {
    state: Arc<GalleryThemeState>,
    button_family_theme: Arc<dyn ButtonFamilyTheme>,
    checkbox_theme: Arc<dyn CheckboxTheme>,
    context_menu_theme: Arc<dyn ContextMenuTheme>,
    navigation_sidebar_theme: Arc<dyn NavigationSidebarTheme>,
    popup_menu_theme: Arc<dyn PopupMenuTheme>,
    progress_theme: Arc<dyn ProgressTheme>,
    radio_button_theme: Arc<dyn RadioButtonTheme>,
    radio_group_theme: Arc<dyn RadioGroupTheme>,
    scrollbar_theme: Arc<dyn ScrollbarTheme>,
    slider_theme: Arc<dyn SliderTheme>,
    switch_theme: Arc<dyn SwitchTheme>,
    tabs_navigation_theme: Arc<dyn TabsNavigationTheme>,
    textarea_theme: Arc<dyn TextAreaTheme>,
    textfield_theme: Arc<dyn TextFieldTheme>,
    toggle_group_theme: Arc<dyn ToggleGroupTheme>,
}

struct GalleryThemeState {
    theme: LumaTheme,
    mode: AtomicU8,
}

#[derive(Clone, Copy)]
pub(in crate::gallery) struct GalleryChrome {
    pub app_background: Hsla,
    pub content_background: Hsla,
    pub title_text: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub border: Hsla,
    pub panel_background: Hsla,
}

impl GalleryThemePack {
    pub(in crate::gallery) fn new() -> Self {
        let state = Arc::new(GalleryThemeState {
            theme: LumaTheme::native(),
            mode: AtomicU8::new(mode_to_u8(ThemeMode::Light)),
        });

        Self {
            state: state.clone(),
            button_family_theme: Arc::new(GalleryButtonFamilyTheme { state: state.clone() }),
            checkbox_theme: Arc::new(GalleryCheckboxTheme { state: state.clone() }),
            context_menu_theme: Arc::new(GalleryContextMenuTheme { state: state.clone() }),
            navigation_sidebar_theme: Arc::new(GalleryNavigationSidebarTheme { state: state.clone() }),
            popup_menu_theme: Arc::new(GalleryPopupMenuTheme { state: state.clone() }),
            progress_theme: Arc::new(GalleryProgressTheme { state: state.clone() }),
            radio_button_theme: Arc::new(GalleryRadioButtonTheme { state: state.clone() }),
            radio_group_theme: Arc::new(GalleryRadioGroupTheme { state: state.clone() }),
            scrollbar_theme: Arc::new(GalleryScrollbarTheme { state: state.clone() }),
            slider_theme: Arc::new(GallerySliderTheme { state: state.clone() }),
            switch_theme: Arc::new(GallerySwitchTheme { state: state.clone() }),
            tabs_navigation_theme: Arc::new(GalleryTabsNavigationTheme { state: state.clone() }),
            textarea_theme: Arc::new(GalleryTextAreaTheme { state: state.clone() }),
            textfield_theme: Arc::new(GalleryTextFieldTheme { state: state.clone() }),
            toggle_group_theme: Arc::new(GalleryToggleGroupTheme { state }),
        }
    }

    pub(in crate::gallery) fn chrome(&self) -> GalleryChrome {
        let tokens = self.state.tokens();
        let palette = &tokens.palette;

        GalleryChrome {
            app_background: palette.app.background,
            content_background: palette.app.background,
            title_text: palette.app.foreground,
            body_text: palette.surface.subtle.foreground,
            muted_text: palette.app.muted_foreground,
            border: palette.border.default,
            panel_background: palette.surface.panel.background,
        }
    }

    pub(in crate::gallery) fn tokens(&self) -> ThemeTokens {
        self.state.tokens()
    }

    pub(in crate::gallery) fn mode(&self) -> ThemeMode {
        self.state.current_mode()
    }

    pub(in crate::gallery) fn toggle_mode(&self) -> ThemeMode {
        let next = match self.mode() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.state.set_mode(next);
        next
    }

    pub(in crate::gallery) fn theme_name(&self) -> &str {
        &self.state.theme.name
    }

    pub(in crate::gallery) fn theme_version(&self) -> u32 {
        self.state.theme.version
    }

    pub(in crate::gallery) fn button_family_theme(&self) -> Arc<dyn ButtonFamilyTheme> {
        self.button_family_theme.clone()
    }

    pub(in crate::gallery) fn button_template(&self) -> Arc<dyn ButtonTemplate<()>> {
        Arc::new(DefaultButtonTemplate::new(self.button_family_theme.clone()))
    }

    pub(in crate::gallery) fn toggle_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        let button_family_theme = self.button_family_theme.clone();
        Arc::new(DefaultButtonTemplate::new(self.button_family_theme.clone()).with_modifier(move |element, model| {
            let variant = match model.kind {
                gpui_luma::controls::button_family::ButtonKind::Standard => ButtonVariant::Standard,
                gpui_luma::controls::button_family::ButtonKind::Ghost => ButtonVariant::Ghost,
                gpui_luma::controls::button_family::ButtonKind::Prominent => ButtonVariant::Prominent,
            };
            let appearance = button_family_theme.resolve(
                variant,
                ButtonFamilyRole::Toggle { selected: model.data },
                model.size,
                model.state,
            );
            element.bg(appearance.background).text_color(appearance.foreground).border_color(appearance.border)
        }))
    }

    pub(in crate::gallery) fn toggle_group_template(&self) -> Arc<dyn ToggleGroupTemplate> {
        Arc::new(ThemedToggleGroupTemplate::new(self.toggle_group_theme.clone()))
    }

    pub(in crate::gallery) fn checkbox_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedCheckboxTemplate::new(self.checkbox_theme.clone()))
    }

    pub(in crate::gallery) fn switch_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedSwitchTemplate::new(self.switch_theme.clone()))
    }

    pub(in crate::gallery) fn radio_button_template(&self) -> Arc<dyn ButtonTemplate<bool>> {
        Arc::new(ThemedRadioButtonTemplate::new(self.radio_button_theme.clone()))
    }

    pub(in crate::gallery) fn radio_group_template(&self) -> Arc<dyn RadioGroupTemplate> {
        Arc::new(ThemedRadioGroupTemplate::new(self.radio_group_theme.clone()))
    }

    pub(in crate::gallery) fn slider_template(&self) -> Arc<dyn SliderTemplate> {
        Arc::new(ThemedSliderTemplate::new(self.slider_theme.clone()))
    }

    pub(in crate::gallery) fn scrollbar_template(&self) -> Arc<dyn ScrollbarTemplate> {
        Arc::new(ThemedScrollbarTemplate::new(self.scrollbar_theme.clone()))
    }

    pub(in crate::gallery) fn textfield_template(&self) -> Arc<dyn TextFieldTemplate> {
        Arc::new(ThemedTextFieldTemplate::new(self.textfield_theme.clone()))
    }

    pub(in crate::gallery) fn textfield_theme(&self) -> Arc<dyn TextFieldTheme> {
        self.textfield_theme.clone()
    }

    pub(in crate::gallery) fn textarea_template(&self) -> Arc<dyn TextAreaTemplate> {
        Arc::new(ThemedTextAreaTemplate::new(self.textarea_theme.clone()))
    }

    pub(in crate::gallery) fn textarea_theme(&self) -> Arc<dyn TextAreaTheme> {
        self.textarea_theme.clone()
    }

    pub(in crate::gallery) fn popup_menu_template(&self) -> Arc<dyn PopupMenuTemplate> {
        Arc::new(ThemedPopupMenuTemplate::new(self.popup_menu_theme.clone()))
    }

    pub(in crate::gallery) fn context_menu_theme(&self) -> Arc<dyn ContextMenuTheme> {
        self.context_menu_theme.clone()
    }

    pub(in crate::gallery) fn navigation_sidebar_theme(&self) -> Arc<dyn NavigationSidebarTheme> {
        self.navigation_sidebar_theme.clone()
    }

    pub(in crate::gallery) fn navigation_sidebar_template(&self) -> Arc<dyn NavigationSidebarTemplate> {
        Arc::new(ThemedNavigationSidebarTemplate::new_with_floating_menu_theme(
            self.navigation_sidebar_theme.clone(),
            Arc::new(DefaultFloatingMenuTheme::new(self.state.tokens())),
        ))
    }

    pub(in crate::gallery) fn tabs_navigation_template(&self) -> Arc<dyn TabsNavigationTemplate> {
        Arc::new(ThemedTabsNavigationTemplate::new(self.tabs_navigation_theme.clone()))
    }

    pub(in crate::gallery) fn progress_template(&self) -> Arc<dyn ProgressTemplate> {
        Arc::new(ThemedProgressTemplate::new(self.progress_theme.clone()))
    }
}

impl Default for GalleryThemePack {
    fn default() -> Self {
        Self::new()
    }
}

impl GalleryThemeState {
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

struct GalleryButtonFamilyTheme {
    state: Arc<GalleryThemeState>,
}

impl ButtonFamilyTheme for GalleryButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: gpui_luma::theme::ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        DefaultButtonFamilyTheme::new(self.state.tokens()).resolve(variant, role, size, state)
    }
}

struct GalleryCheckboxTheme {
    state: Arc<GalleryThemeState>,
}

impl CheckboxTheme for GalleryCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        DefaultCheckboxTheme::new(self.state.tokens()).resolve(checked, state)
    }
}

struct GallerySwitchTheme {
    state: Arc<GalleryThemeState>,
}

impl SwitchTheme for GallerySwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState) -> SwitchAppearance {
        DefaultSwitchTheme::new(self.state.tokens()).resolve(on, state)
    }
}

struct GalleryRadioButtonTheme {
    state: Arc<GalleryThemeState>,
}

impl RadioButtonTheme for GalleryRadioButtonTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> RadioButtonAppearance {
        DefaultRadioButtonTheme::new(self.state.tokens()).resolve(checked, state)
    }
}

struct GalleryRadioGroupTheme {
    state: Arc<GalleryThemeState>,
}

impl RadioGroupTheme for GalleryRadioGroupTheme {
    fn resolve_item(&self, selected: bool, state: InteractionState) -> RadioGroupItemAppearance {
        DefaultRadioGroupTheme::new(self.state.tokens()).resolve_item(selected, state)
    }
}

struct GallerySliderTheme {
    state: Arc<GalleryThemeState>,
}

impl SliderTheme for GallerySliderTheme {
    fn resolve(&self, state: InteractionState) -> SliderAppearance {
        DefaultSliderTheme::new(self.state.tokens()).resolve(state)
    }
}

struct GalleryScrollbarTheme {
    state: Arc<GalleryThemeState>,
}

impl ScrollbarTheme for GalleryScrollbarTheme {
    fn resolve(&self, state: InteractionState, orientation: ScrollbarOrientation) -> ScrollbarAppearance {
        DefaultScrollbarTheme::new(self.state.tokens()).resolve(state, orientation)
    }
}

struct GalleryTextFieldTheme {
    state: Arc<GalleryThemeState>,
}

impl TextFieldTheme for GalleryTextFieldTheme {
    fn resolve(&self, state: gpui_luma::controls::textfield::TextFieldState, enabled: bool) -> TextFieldAppearance {
        DefaultTextFieldTheme::new(self.state.tokens()).resolve(state, enabled)
    }
}

struct GalleryTextAreaTheme {
    state: Arc<GalleryThemeState>,
}

impl TextAreaTheme for GalleryTextAreaTheme {
    fn resolve(&self, state: gpui_luma::controls::textarea::TextAreaState, enabled: bool) -> TextAreaAppearance {
        DefaultTextAreaTheme::new(self.state.tokens()).resolve(state, enabled)
    }
}

struct GalleryPopupMenuTheme {
    state: Arc<GalleryThemeState>,
}

impl PopupMenuTheme for GalleryPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance {
        DefaultPopupMenuTheme::new(self.state.tokens()).resolve(state)
    }
}

struct GalleryNavigationSidebarTheme {
    state: Arc<GalleryThemeState>,
}

impl NavigationSidebarTheme for GalleryNavigationSidebarTheme {
    fn resolve_container(&self) -> gpui_luma::theme::NavigationSidebarContainerAppearance {
        gpui_luma::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_container()
    }

    fn resolve_section(&self) -> gpui_luma::theme::NavigationSidebarSectionAppearance {
        gpui_luma::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_section()
    }

    fn resolve_branch(
        &self,
        state: InteractionState,
        size: gpui_luma::theme::ControlSize,
    ) -> gpui_luma::theme::NavigationSidebarItemAppearance {
        gpui_luma::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_branch(state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: gpui_luma::theme::ControlSize,
    ) -> gpui_luma::theme::NavigationSidebarItemAppearance {
        gpui_luma::theme::DefaultNavigationSidebarTheme::new(self.state.tokens()).resolve_item(selected, state, size)
    }
}

struct GalleryContextMenuTheme {
    state: Arc<GalleryThemeState>,
}

impl ContextMenuTheme for GalleryContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> ContextMenuAppearance {
        DefaultContextMenuTheme::new(self.state.tokens()).resolve(state)
    }
}

struct GalleryToggleGroupTheme {
    state: Arc<GalleryThemeState>,
}

impl ToggleGroupTheme for GalleryToggleGroupTheme {
    fn resolve_list(&self, enabled: bool, size: gpui_luma::theme::ControlSize) -> ToggleGroupListAppearance {
        DefaultToggleGroupTheme::new(self.state.tokens()).resolve_list(enabled, size)
    }

    fn resolve_item(
        &self,
        variant: ButtonVariant,
        selected: bool,
        state: InteractionState,
        size: gpui_luma::theme::ControlSize,
    ) -> ToggleGroupItemAppearance {
        DefaultToggleGroupTheme::new(self.state.tokens()).resolve_item(variant, selected, state, size)
    }
}

struct GalleryProgressTheme {
    state: Arc<GalleryThemeState>,
}

impl ProgressTheme for GalleryProgressTheme {
    fn resolve(&self, enabled: bool) -> ProgressAppearance {
        DefaultProgressTheme::new(self.state.tokens()).resolve(enabled)
    }
}

struct GalleryTabsNavigationTheme {
    state: Arc<GalleryThemeState>,
}

impl TabsNavigationTheme for GalleryTabsNavigationTheme {
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
