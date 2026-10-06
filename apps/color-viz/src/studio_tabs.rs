//! Studio navigation uses SDK tabs with a look-backed selected chip.
use std::sync::Arc;
use gpui_luma::controls::tabs::{TabsItemLook, TabsListLook, TabsTemplate, TabsTheme, ThemedTabsTemplate};
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;

struct StudioTabsTheme {
    look: Arc<ShadcnLook>,
    segmented: bool,
}

impl TabsTheme for StudioTabsTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        let mut list = self.look.tabs_theme().resolve_list(enabled, size);
        list.background = None;
        list.border = None;
        list.padding = if self.segmented { 3.0 } else { 0.0 };
        list.radius = 5.0;
        list.gap = if self.segmented { 2.0 } else { 8.0 };
        list
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let mut item = self.look.tabs_theme().resolve_item(active, state, size);
        let chrome = self.look.chrome();
        if !self.segmented {
            item.label_typography.size = 14.0;
            item.label_typography.line_height = 20.0;
        }
        item.indicator = None;
        item.indicator_height = 0.0;
        item.background = (active || state.hovered || state.focused).then_some(chrome.border);
        item.label_color = if state.disabled {
            chrome.muted_text
        } else if active {
            chrome.body_text
        } else {
            chrome.muted_text
        };
        item.radius = 4.0;
        item.padding_x = 14.0;
        item.height = if self.segmented { 26.0 } else { 32.0 };
        item
    }

    fn font_family(&self) -> gpui::SharedString {
        self.look.mode_tokens().typography.font.sans.family.clone().into()
    }
}

pub(crate) fn template(look: Arc<ShadcnLook>, segmented: bool) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(Arc::new(StudioTabsTheme { look, segmented })))
}
