use std::sync::Arc;

use luma::controls::control_group::ControlGroupTemplate;
use luma::controls::listbox::{ListBoxTheme, listbox_template_with_theme};
use luma::theme::{ControlSize, InteractionState};

use crate::controls::listbox::{listbox_list_look, listbox_row_palette};
use crate::look::ShadcnLook;

struct ShadcnListBoxTheme {
    theme: ShadcnLook,
}

impl ListBoxTheme for ShadcnListBoxTheme {
    fn resolve_list(
        &self,
        enabled: bool,
        focused: bool,
        size: ControlSize,
    ) -> luma::controls::listbox::ListBoxListLook {
        let tokens = self.theme.mode_tokens();
        listbox_list_look(tokens.as_ref(), enabled, focused, size)
    }

    fn resolve_row(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> luma::controls::listbox::ListBoxRowPalette {
        let tokens = self.theme.mode_tokens();
        listbox_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn listbox_theme(theme: ShadcnLook) -> Arc<dyn ListBoxTheme> {
    Arc::new(ShadcnListBoxTheme { theme: theme.clone() })
}

pub fn listbox_template(theme: ShadcnLook) -> ControlGroupTemplate<luma::controls::listbox::ListBoxItem> {
    listbox_template_with_theme(listbox_theme(theme))
}
