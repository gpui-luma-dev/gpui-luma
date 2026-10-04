use std::sync::Arc;

use gpui_luma::controls::tabs::{TabsTemplate, TabsTheme, ThemedTabsTemplate};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::tabs::{tabs_item_look_with_stylesheet, tabs_list_look_with_stylesheet};
use crate::look::ShadcnLook;

struct ShadcnTabsTheme {
    theme: ShadcnLook,
}

impl TabsTheme for ShadcnTabsTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> gpui_luma::controls::tabs::TabsListLook {
        let tokens = self.theme.mode_tokens();
        tabs_list_look_with_stylesheet(tokens.as_ref(), &self.theme.stylesheet(), enabled, size)
    }

    fn resolve_item(
        &self,
        active: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::tabs::TabsItemLook {
        let tokens = self.theme.mode_tokens();
        tabs_item_look_with_stylesheet(tokens.as_ref(), &self.theme.stylesheet(), active, state, size)
    }

    fn font_family(&self) -> gpui::SharedString {
        self.theme.mode_tokens().typography.font.sans.family.clone().into()
    }
}

pub fn tabs_template(theme: ShadcnLook) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(tabs_theme(theme.clone())))
}

pub fn tabs_theme(theme: ShadcnLook) -> Arc<dyn TabsTheme> {
    Arc::new(ShadcnTabsTheme { theme: theme.clone() })
}
