use std::sync::Arc;

use luma::controls::scrollbar::{ScrollbarTheme, ThemedScrollbarTemplate};
use luma::theme::InteractionState;

use crate::controls::scrollbar::scrollbar_look;
use crate::look::ShadcnLook;

pub fn scrollbar_theme(theme: ShadcnLook) -> Arc<dyn ScrollbarTheme> {
    Arc::new(ShadcnScrollbarTheme { theme: theme.clone() })
}

struct ShadcnScrollbarTheme {
    theme: ShadcnLook,
}

impl ScrollbarTheme for ShadcnScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: luma::controls::scrollbar::ScrollbarOrientation,
        size: luma::theme::ControlSize,
        style: luma::controls::scrollbar::ScrollbarStyle,
    ) -> luma::controls::scrollbar::ScrollbarLook {
        let tokens = self.theme.mode_tokens();
        scrollbar_look(tokens.as_ref(), state, orientation, size, style)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn scrollbar_template(theme: ShadcnLook) -> Arc<dyn luma::controls::scrollbar::ScrollbarTemplate> {
    Arc::new(ThemedScrollbarTemplate::new(Arc::new(ShadcnScrollbarTheme { theme: theme.clone() })))
}
