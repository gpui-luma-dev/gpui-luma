use std::sync::Arc;

use gpui_luma::controls::table::{TableTheme, table_template_with_theme};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::table::{table_look, table_row_palette};
use crate::look::ShadcnLook;

struct ShadcnTableTheme {
    theme: ShadcnLook,
}

impl TableTheme for ShadcnTableTheme {
    fn resolve_drag(&self, size: ControlSize) -> gpui_luma::controls::table::TableDragLook {
        let tokens = self.theme.mode_tokens();
        let palette = &tokens.palette;
        gpui_luma::controls::table::TableDragLook {
            background: self.theme.token_color("popover").unwrap_or(palette.panel_background),
            foreground: self.theme.token_color("popover-foreground").unwrap_or(palette.body_text),
            valid_marker: self.theme.token_color("ring").unwrap_or(palette.focus_ring),
            invalid_marker: self.theme.token_color("destructive").unwrap_or(palette.destructive_background),
            radius: tokens.metrics.radius(size),
        }
    }

    fn resolve_look(&self, enabled: bool, focused: bool, size: ControlSize) -> gpui_luma::controls::table::TableLook {
        let tokens = self.theme.mode_tokens();
        table_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::table::TableRowPalette {
        let tokens = self.theme.mode_tokens();
        table_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn table_theme(theme: ShadcnLook) -> Arc<dyn TableTheme> {
    Arc::new(ShadcnTableTheme { theme: theme.clone() })
}

pub fn table_template(theme: ShadcnLook) -> Arc<dyn gpui_luma::controls::table::TableTemplate> {
    table_template_with_theme(table_theme(theme))
}
