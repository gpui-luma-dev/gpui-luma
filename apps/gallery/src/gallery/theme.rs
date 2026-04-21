use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use gpui::{Hsla, rgb};
use gpui_luma::controls::{
    button::{ButtonTemplate, ThemedButtonTemplate},
    checkbox::{CheckboxTemplate, ThemedCheckboxTemplate},
    icon_button::{IconButtonTemplate, ThemedIconButtonTemplate},
    popup_menu::{PopupMenuTemplate, ThemedPopupMenuTemplate},
    progress::{ProgressTemplate, ThemedProgressTemplate},
    radio_group::{RadioGroupTemplate, ThemedRadioGroupTemplate},
    scrollbar::{ScrollbarOrientation, ScrollbarTemplate, ThemedScrollbarTemplate},
    slider::{SliderTemplate, ThemedSliderTemplate},
    switch::{SwitchTemplate, ThemedSwitchTemplate},
    tabs_navigation::{TabsNavigationTemplate, ThemedTabsNavigationTemplate},
    toggle_button::{ThemedToggleButtonTemplate, ToggleButtonTemplate},
    toggle_group::{ThemedToggleGroupTemplate, ToggleGroupTemplate},
};
use gpui_luma::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, CheckboxAppearance, CheckboxTheme,
    ContextMenuAppearance, ContextMenuTheme, DefaultButtonFamilyTheme, DefaultCheckboxTheme, DefaultContextMenuTheme,
    DefaultPopupMenuTheme, DefaultProgressTheme, DefaultRadioGroupTheme, DefaultScrollbarTheme, DefaultSliderTheme,
    DefaultSwitchTheme, DefaultTabsNavigationTheme, DefaultToggleGroupTheme, InteractionState, LumaTheme,
    PopupMenuAppearance, PopupMenuTheme, ProgressAppearance, ProgressTheme, RadioGroupItemAppearance, RadioGroupTheme,
    ScrollbarAppearance, ScrollbarTheme, SliderAppearance, SliderTheme, SwitchAppearance, SwitchTheme,
    TabsNavigationItemAppearance, TabsNavigationListAppearance, TabsNavigationTheme, ThemeMode, ThemeTokens,
    ToggleGroupItemAppearance, ToggleGroupListAppearance, ToggleGroupTheme,
};

#[derive(Clone)]
pub(in crate::gallery) struct GalleryThemePack {
    state: Arc<GalleryThemeState>,
    button_family_theme: Arc<dyn ButtonFamilyTheme>,
    checkbox_theme: Arc<dyn CheckboxTheme>,
    context_menu_theme: Arc<dyn ContextMenuTheme>,
    popup_menu_theme: Arc<dyn PopupMenuTheme>,
    progress_theme: Arc<dyn ProgressTheme>,
    radio_group_theme: Arc<dyn RadioGroupTheme>,
    scrollbar_theme: Arc<dyn ScrollbarTheme>,
    slider_theme: Arc<dyn SliderTheme>,
    switch_theme: Arc<dyn SwitchTheme>,
    tabs_navigation_theme: Arc<dyn TabsNavigationTheme>,
    toggle_group_theme: Arc<dyn ToggleGroupTheme>,
}

struct GalleryThemeState {
    theme: LumaTheme,
    mode: AtomicU8,
}

#[derive(Clone, Copy)]
pub(in crate::gallery) struct GalleryChrome {
    pub app_background: Hsla,
    pub sidebar_background: Hsla,
    pub content_background: Hsla,
    pub title_text: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub section_label: Hsla,
    pub border: Hsla,
    pub panel_background: Hsla,
    pub focus_ring: Hsla,
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
            popup_menu_theme: Arc::new(GalleryPopupMenuTheme { state: state.clone() }),
            progress_theme: Arc::new(GalleryProgressTheme { state: state.clone() }),
            radio_group_theme: Arc::new(GalleryRadioGroupTheme { state: state.clone() }),
            scrollbar_theme: Arc::new(GalleryScrollbarTheme { state: state.clone() }),
            slider_theme: Arc::new(GallerySliderTheme { state: state.clone() }),
            switch_theme: Arc::new(GallerySwitchTheme { state: state.clone() }),
            tabs_navigation_theme: Arc::new(GalleryTabsNavigationTheme { state: state.clone() }),
            toggle_group_theme: Arc::new(GalleryToggleGroupTheme { state }),
        }
    }

    pub(in crate::gallery) fn mode(&self) -> ThemeMode {
        u8_to_mode(self.state.mode.load(Ordering::Relaxed))
    }

    pub(in crate::gallery) fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
    }

    pub(in crate::gallery) fn chrome(&self) -> GalleryChrome {
        let tokens = self.state.tokens();
        let palette = &tokens.palette;

        GalleryChrome {
            app_background: palette.app.background,
            sidebar_background: palette.navigation.background,
            content_background: palette.app.background,
            title_text: palette.app.foreground,
            body_text: palette.surface.subtle.foreground,
            muted_text: palette.app.muted_foreground,
            section_label: palette.navigation.muted_foreground,
            border: palette.border.default,
            panel_background: palette.surface.panel.background,
            focus_ring: palette.focus.ring,
        }
    }

    pub(in crate::gallery) fn tokens(&self) -> ThemeTokens {
        self.state.tokens()
    }

    pub(in crate::gallery) fn theme_name(&self) -> &str {
        &self.state.theme.name
    }

    pub(in crate::gallery) fn theme_version(&self) -> u32 {
        self.state.theme.version
    }

    pub(in crate::gallery) fn button_template(&self) -> Arc<dyn ButtonTemplate> {
        Arc::new(ThemedButtonTemplate::new(self.button_family_theme.clone()))
    }

    pub(in crate::gallery) fn icon_button_template(&self) -> Arc<dyn IconButtonTemplate> {
        Arc::new(ThemedIconButtonTemplate::new(self.button_family_theme.clone()))
    }

    pub(in crate::gallery) fn toggle_button_template(&self) -> Arc<dyn ToggleButtonTemplate> {
        Arc::new(ThemedToggleButtonTemplate::new(self.button_family_theme.clone()))
    }

    pub(in crate::gallery) fn toggle_group_template(&self) -> Arc<dyn ToggleGroupTemplate> {
        Arc::new(ThemedToggleGroupTemplate::new(self.toggle_group_theme.clone()))
    }

    pub(in crate::gallery) fn checkbox_template(&self) -> Arc<dyn CheckboxTemplate> {
        Arc::new(ThemedCheckboxTemplate::new(self.checkbox_theme.clone()))
    }

    pub(in crate::gallery) fn border_checkbox_template(&self) -> Arc<dyn CheckboxTemplate> {
        Arc::new(ThemedCheckboxTemplate::new(Arc::new(GalleryCheckboxPresentationTheme {
            state: self.state.clone(),
            presentation: CheckboxPresentation::Border,
        })))
    }

    pub(in crate::gallery) fn filled_checkbox_template(&self) -> Arc<dyn CheckboxTemplate> {
        Arc::new(ThemedCheckboxTemplate::new(Arc::new(GalleryCheckboxPresentationTheme {
            state: self.state.clone(),
            presentation: CheckboxPresentation::Filled,
        })))
    }

    pub(in crate::gallery) fn switch_template(&self) -> Arc<dyn SwitchTemplate> {
        Arc::new(ThemedSwitchTemplate::new(self.switch_theme.clone()))
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

    pub(in crate::gallery) fn popup_menu_template(&self) -> Arc<dyn PopupMenuTemplate> {
        Arc::new(ThemedPopupMenuTemplate::new(self.popup_menu_theme.clone()))
    }

    pub(in crate::gallery) fn context_menu_theme(&self) -> Arc<dyn ContextMenuTheme> {
        self.context_menu_theme.clone()
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
    fn tokens(&self) -> ThemeTokens {
        self.theme.mode(u8_to_mode(self.mode.load(Ordering::Relaxed))).clone()
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

enum CheckboxPresentation {
    Border,
    Filled,
}

struct GalleryCheckboxPresentationTheme {
    state: Arc<GalleryThemeState>,
    presentation: CheckboxPresentation,
}

impl CheckboxTheme for GalleryCheckboxPresentationTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let mut appearance = DefaultCheckboxTheme::new(self.state.tokens()).resolve(checked, state);

        if !state.disabled {
            let mode = u8_to_mode(self.state.mode.load(Ordering::Relaxed));
            match (mode, &self.presentation) {
                (ThemeMode::Light, CheckboxPresentation::Border) => {
                    appearance.control_border = Some(rgb(0x2563eb).into());
                }
                (ThemeMode::Dark, CheckboxPresentation::Border) => {
                    appearance.control_border = Some(rgb(0x60a5fa).into());
                }
                (ThemeMode::Light, CheckboxPresentation::Filled) => {
                    appearance.control_border = Some(rgb(0xbe185d).into());
                    appearance.control_background = Some(rgb(0xfce7f3).into());
                }
                (ThemeMode::Dark, CheckboxPresentation::Filled) => {
                    appearance.control_border = Some(rgb(0xf472b6).into());
                    appearance.control_background = Some(rgb(0x3b1327).into());
                }
            }

            appearance.control_padding_x = 10.0;
            appearance.control_padding_y = 6.0;
        }

        appearance
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

struct GalleryPopupMenuTheme {
    state: Arc<GalleryThemeState>,
}

impl PopupMenuTheme for GalleryPopupMenuTheme {
    fn resolve(&self, state: InteractionState) -> PopupMenuAppearance {
        DefaultPopupMenuTheme::new(self.state.tokens()).resolve(state)
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
