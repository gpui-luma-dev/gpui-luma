use std::sync::Arc;

use luma::controls::toolbar::{ThemedToolbarTemplate, ToolbarLook, ToolbarTemplate, ToolbarTheme, ToolbarVariant};
use luma::theme::ControlSize;

use crate::look::ShadcnLook;

struct ShadcnToolbarTheme {
    theme: ShadcnLook,
}

impl ToolbarTheme for ShadcnToolbarTheme {
    fn resolve(&self, enabled: bool, size: ControlSize, variant: ToolbarVariant) -> ToolbarLook {
        let tokens = self.theme.mode_tokens();
        let metrics = &tokens.metrics;
        let control = metrics.for_size(size);
        let transparent = gpui::hsla(0.0, 0.0, 0.0, 0.0);

        let (background, border) = match variant {
            ToolbarVariant::Outline => {
                let background = if enabled {
                    tokens.palette.muted_background
                } else {
                    tokens.palette.disabled_background
                };
                (background, tokens.palette.border_default)
            }
            ToolbarVariant::Ghost => (transparent, transparent),
        };

        ToolbarLook {
            background,
            border,
            separator: tokens.palette.border_default,
            radius: metrics.radius.md,
            padding_x: 6.0,
            padding_y: 4.0,
            gap: control.gap,
            separator_height: control.height,
        }
    }
}

pub fn toolbar_theme(theme: ShadcnLook) -> Arc<dyn ToolbarTheme> {
    Arc::new(ShadcnToolbarTheme { theme: theme.clone() })
}

pub fn toolbar_template(theme: ShadcnLook) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(toolbar_theme(theme)))
}
