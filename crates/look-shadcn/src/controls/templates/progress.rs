use std::sync::Arc;

use luma::controls::progress::{ProgressTheme, ThemedLinearProgressTemplate, ThemedProgressTemplate};

use crate::controls::progress::progress_look;
use crate::look::ShadcnLook;

struct ShadcnProgressTheme {
    theme: ShadcnLook,
}

impl ProgressTheme for ShadcnProgressTheme {
    fn resolve(&self, enabled: bool, size: luma::theme::ControlSize) -> luma::controls::progress::ProgressLook {
        let tokens = self.theme.mode_tokens();
        progress_look(tokens.as_ref(), enabled, size)
    }
}

pub fn progress_template(theme: ShadcnLook) -> Arc<dyn luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedProgressTemplate::new(progress_theme(theme)))
}

pub fn linear_progress_template(theme: ShadcnLook) -> Arc<dyn luma::controls::progress::ProgressTemplate> {
    Arc::new(ThemedLinearProgressTemplate::new(progress_theme(theme)))
}

pub fn progress_theme(theme: ShadcnLook) -> Arc<dyn ProgressTheme> {
    Arc::new(ShadcnProgressTheme { theme: theme.clone() })
}
