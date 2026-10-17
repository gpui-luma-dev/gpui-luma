use std::sync::Arc;

use gpui_luma::controls::tree_view::{TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::tree_view::tree_view_row_palette;
use crate::look::ShadcnLook;

struct ShadcnTreeViewTheme {
    theme: ShadcnLook,
}

impl TreeViewTheme for ShadcnTreeViewTheme {
    fn drop_layout_motion(&self) -> gpui_luma::theme::stylesheet::ResolvedMotion {
        self.theme.stylesheet().common.drop_layout_motion("Shadcn")
    }
    fn resolve_row(
        &self,
        state: InteractionState,
        selected: bool,
        size: ControlSize,
    ) -> gpui_luma::controls::tree_view::TreeViewPalette {
        let tokens = self.theme.mode_tokens();
        tree_view_row_palette(tokens.as_ref(), selected, state, size)
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn tree_view_template<T>(theme: ShadcnLook) -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
{
    Arc::new(ThemedTreeViewTemplate::new(tree_view_theme(theme.clone())))
}

pub fn tree_view_theme(theme: ShadcnLook) -> Arc<dyn TreeViewTheme> {
    Arc::new(ShadcnTreeViewTheme { theme: theme.clone() })
}
