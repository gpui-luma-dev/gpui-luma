//! SDK default theme factories for the controls app and headless support tests.
use std::sync::Arc;
use gpui::SharedString;
use gpui_luma::controls::{
    button::{ButtonBuilder, ButtonTemplate, DefaultButtonTemplate},
    button_family::DefaultButtonFamilyTheme,
    scrollbar::{DefaultScrollbarTheme, ThemedScrollbarTemplate},
    switch::{DefaultSwitchTheme, SwitchTheme},
};
use gpui_luma::theme::{StandardBoxScale, ThemeMode, ThemeTokens};
use crate::event_log::{EventLogLook, EventLogTheme, EventLogViewBuilder, compose_event_log_look};

#[derive(Clone)]
pub struct NativeTheme {
    pub mode: ThemeMode,
    pub tokens: ThemeTokens,
}
impl NativeTheme {
    pub fn light() -> Self {
        Self::new(ThemeMode::Light)
    }
    pub fn new(mode: ThemeMode) -> Self {
        Self {
            mode,
            tokens: match mode {
                ThemeMode::Light => ThemeTokens::light(),
                ThemeMode::Dark => ThemeTokens::dark(),
            },
        }
    }
    pub fn button_template(&self) -> Arc<dyn ButtonTemplate<()>> {
        Arc::new(DefaultButtonTemplate::new(Arc::new(DefaultButtonFamilyTheme::new(self.tokens.clone()))))
    }
    pub fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        ButtonBuilder::new(id).template(self.button_template())
    }
    pub fn switch_theme(&self) -> Arc<dyn SwitchTheme> {
        Arc::new(DefaultSwitchTheme::new(self.tokens.clone()))
    }
    pub fn scrollbar_template(&self) -> Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate> {
        Arc::new(ThemedScrollbarTemplate::new(Arc::new(DefaultScrollbarTheme::new(self.tokens.clone()))))
    }
    pub fn event_log(&self, id: impl Into<SharedString>) -> EventLogViewBuilder {
        EventLogViewBuilder::with_parts(id, Arc::new(self.clone()), self.scrollbar_template())
    }
}
impl EventLogTheme for NativeTheme {
    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.tokens.metrics
    }
    fn resolve_look(&self, rows: usize, scale: &StandardBoxScale, focused: bool) -> EventLogLook {
        let p = &self.tokens.palette;
        compose_event_log_look(
            p.form.input.background,
            p.app.foreground,
            if focused { p.focus.ring } else { p.border.default },
            p.app.muted_foreground,
            self.tokens.typography.text.caption,
            self.tokens.typography.font.mono.family.clone(),
            rows,
            scale,
            self.tokens.metrics.border_width.default,
        )
    }
}
