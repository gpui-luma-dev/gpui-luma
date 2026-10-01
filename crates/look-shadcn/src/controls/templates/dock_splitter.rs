use std::sync::Arc;

use gpui_luma::controls::dock_splitter::DockSplitterTheme;

use crate::look::ShadcnLook;

pub fn dock_splitter_theme(theme: ShadcnLook) -> Arc<dyn DockSplitterTheme> {
    Arc::new(ShadcnDockSplitterTheme { theme: theme.clone() })
}

struct ShadcnDockSplitterTheme {
    theme: ShadcnLook,
}

impl DockSplitterTheme for ShadcnDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> gpui_luma::controls::dock_splitter::DockSplitterLook {
        let tokens = self.theme.mode_tokens();
        let border = tokens.palette.border_default;
        let disabled = tokens.palette.disabled_foreground;

        gpui_luma::controls::dock_splitter::DockSplitterLook {
            line_color: if enabled { border } else { disabled },
            hover_color: if enabled { border } else { disabled },
            thumb_color: if enabled {
                tokens.palette.primary.background
            } else {
                disabled
            },
            hit_target_px: 8.0,
            visible_line_px: 1.0,
        }
    }
}
