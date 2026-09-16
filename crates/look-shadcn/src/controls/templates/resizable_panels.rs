use std::sync::Arc;

use luma::controls::resizable_panels::ResizablePanelsTheme;
use luma::theme::InteractionState;

use crate::controls::resizable_panels::resizable_panels_look;
use crate::look::ShadcnLook;

pub fn resizable_panels_theme(theme: ShadcnLook) -> Arc<dyn ResizablePanelsTheme> {
    Arc::new(ShadcnResizablePanelsTheme { theme: theme.clone() })
}

struct ShadcnResizablePanelsTheme {
    theme: ShadcnLook,
}

impl ResizablePanelsTheme for ShadcnResizablePanelsTheme {
    fn resolve(&self, state: InteractionState) -> luma::controls::resizable_panels::ResizablePanelsLook {
        let tokens = self.theme.mode_tokens();
        resizable_panels_look(tokens.as_ref(), self.theme.mode(), state)
    }
}
