use std::sync::Arc;

use luma::controls::table::{TableTheme, table_template_with_theme};
use luma::theme::{ControlSize, InteractionState};

use crate::controls::table::{table_look, table_row_palette};
use crate::look::ShadcnLook;

struct ShadcnTableTheme {
    theme: ShadcnLook,
}

impl TableTheme for ShadcnTableTheme {
    fn resolve_look(&self, enabled: bool, focused: bool, size: ControlSize) -> luma::controls::table::TableLook {
        let tokens = self.theme.mode_tokens();
        table_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> luma::controls::table::TableRowPalette {
        let tokens = self.theme.mode_tokens();
        table_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn table_theme(theme: ShadcnLook) -> Arc<dyn TableTheme> {
    Arc::new(ShadcnTableTheme { theme: theme.clone() })
}

pub fn table_template(theme: ShadcnLook) -> Arc<dyn luma::controls::table::TableTemplate> {
    table_template_with_theme(table_theme(theme))
}
