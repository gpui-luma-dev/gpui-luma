use std::sync::Arc;

use luma::controls::textfield::{TextFieldState, TextFieldTheme, ThemedTextFieldTemplate};
use luma::theme::{ControlSize, StandardBoxScale};

use crate::look::ShadcnLook;

struct ShadcnTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Outline,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn textfield_template(theme: ShadcnLook) -> Arc<dyn luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnTextFieldTheme { theme: theme.clone() })))
}

pub fn textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnTextFieldTheme { theme: theme.clone() })
}

struct ShadcnInputTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnInputTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Input,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn input_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnInputTextFieldTheme { theme: theme.clone() })
}

pub fn input_textfield_template(theme: ShadcnLook) -> Arc<dyn luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnInputTextFieldTheme { theme: theme.clone() })))
}

struct ShadcnSurfaceTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnSurfaceTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Surface,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn surface_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnSurfaceTextFieldTheme { theme: theme.clone() })
}

struct ShadcnPrimaryTextFieldTheme {
    theme: ShadcnLook,
}

impl TextFieldTheme for ShadcnPrimaryTextFieldTheme {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> luma::controls::textfield::TextFieldPalette {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_palette(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
        )
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> luma::controls::textfield::TextFieldLook {
        let tokens = self.theme.mode_tokens();
        crate::controls::textfield::textfield_look(
            tokens.as_ref(),
            self.theme.mode(),
            crate::controls::textfield::ShadcnTextFieldStyle::Primary,
            state,
            enabled,
            size,
            scale,
        )
    }
}

pub fn primary_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    Arc::new(ShadcnPrimaryTextFieldTheme { theme: theme.clone() })
}

pub fn primary_textfield_template(theme: ShadcnLook) -> Arc<dyn luma::controls::textfield::TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(Arc::new(ShadcnPrimaryTextFieldTheme { theme: theme.clone() })))
}
