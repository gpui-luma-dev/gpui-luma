use std::sync::Arc;

use gpui_luma::controls::context_menu::{ContextMenuTheme, ThemedContextMenuTemplate};
use gpui_luma::theme::InteractionState;

use crate::controls::context_menu::context_menu_look;
use crate::look::ShadcnLook;

struct ShadcnContextMenuTheme {
    theme: ShadcnLook,
}

impl ContextMenuTheme for ShadcnContextMenuTheme {
    fn resolve(&self, state: InteractionState) -> gpui_luma::controls::context_menu::ContextMenuLook {
        let tokens = self.theme.mode_tokens();
        context_menu_look(tokens.as_ref(), self.theme.mode(), state)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn context_menu_theme(theme: ShadcnLook) -> Arc<dyn ContextMenuTheme> {
    Arc::new(ShadcnContextMenuTheme { theme: theme.clone() })
}

pub fn context_menu_template(theme: ShadcnLook) -> Arc<dyn gpui_luma::controls::context_menu::ContextMenuTemplate> {
    Arc::new(ThemedContextMenuTemplate::new(context_menu_theme(theme.clone())))
}
