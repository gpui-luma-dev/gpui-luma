use std::sync::Arc;

use luma::controls::sidebar::{SidebarPanelTemplate, SidebarTheme, ThemedSidebarPanelTemplate};
use luma::theme::{ControlSize, InteractionState};

use super::floating_menu::floating_menu_theme;
use crate::controls::sidebar::{sidebar_branch_look, sidebar_container_look, sidebar_item_look, sidebar_section_look};
use crate::look::ShadcnLook;

struct ShadcnSidebarTheme {
    theme: ShadcnLook,
}

impl SidebarTheme for ShadcnSidebarTheme {
    fn foreground(&self) -> gpui::Hsla {
        sidebar_container_look(self.theme.mode_tokens().as_ref()).foreground
    }

    fn resolve_section(&self) -> luma::controls::sidebar::SidebarSectionLook {
        sidebar_section_look(&self.theme)
    }

    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> luma::controls::sidebar::SidebarItemLook {
        sidebar_branch_look(&self.theme, state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> luma::controls::sidebar::SidebarItemLook {
        sidebar_item_look(&self.theme, selected, state, size)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn sidebar_panel_template(theme: ShadcnLook) -> Arc<dyn SidebarPanelTemplate> {
    let menu_theme = floating_menu_theme(theme.clone());
    Arc::new(ThemedSidebarPanelTemplate::new_with_floating_menu_theme(
        Arc::new(ShadcnSidebarTheme { theme: theme.clone() }),
        menu_theme,
    ))
}

pub fn sidebar_theme(theme: ShadcnLook) -> Arc<dyn SidebarTheme> {
    Arc::new(ShadcnSidebarTheme { theme: theme.clone() })
}
