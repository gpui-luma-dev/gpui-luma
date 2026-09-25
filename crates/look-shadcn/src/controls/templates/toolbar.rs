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
        let metrics = crate::tables::metrics::resolve_toolbar_metrics(&tokens, self.theme.mode(), size);
        let colors = crate::tables::resolve_toolbar_colors(&tokens, self.theme.mode(), enabled, variant);

        ToolbarLook {
            background: colors.background.hsla(),
            border: colors.border.hsla(),
            separator: colors.separator.hsla(),
            radius: metrics.radius.value_px,
            padding_x: metrics.padding_x.value_px,
            padding_y: metrics.padding_y.value_px,
            gap: metrics.gap.value_px,
            separator_height: metrics.separator_height.value_px,
        }
    }
}

pub fn toolbar_theme(theme: ShadcnLook) -> Arc<dyn ToolbarTheme> {
    Arc::new(ShadcnToolbarTheme { theme: theme.clone() })
}

pub fn toolbar_template(theme: ShadcnLook) -> Arc<dyn ToolbarTemplate> {
    Arc::new(ThemedToolbarTemplate::new(toolbar_theme(theme)))
}
