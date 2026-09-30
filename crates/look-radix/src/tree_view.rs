//! Radix scale colors for the SDK's interactive tree view.

use std::sync::Arc;

use gpui::{Context, Div, SharedString, div, prelude::*};
use luma::controls::tree_view::{
    TreeNode, TreeViewBuilder, TreeViewPalette, TreeViewTemplate, TreeViewTheme, ThemedTreeViewTemplate,
};
use luma::theme::{ControlSize, InteractionState, MetricTokens};

use crate::{Look, ScaleFamily};

struct RadixTreeViewTheme {
    look: Look,
}

/// Neutral frame for a tree viewport. The application supplies its dimensions.
pub fn tree_view_frame(look: &Look) -> Div {
    div()
        .border_1()
        .border_color(look.resolve_step(ScaleFamily::Gray, 6).hsla())
        .bg(look.resolve_step(ScaleFamily::Gray, 1).hsla())
        .overflow_hidden()
}

impl TreeViewTheme for RadixTreeViewTheme {
    fn resolve_row(&self, state: InteractionState, selected: bool, size: ControlSize) -> TreeViewPalette {
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        let accent = |step| self.look.resolve_step(ScaleFamily::Color, step).hsla();
        let background = if state.disabled {
            None
        } else if state.pressed {
            Some(accent(5))
        } else if selected {
            Some(accent(3))
        } else if state.focused {
            Some(accent(4))
        } else {
            None
        };
        let foreground = if state.disabled {
            gray(11)
        } else if selected {
            accent(12)
        } else {
            gray(12)
        };
        let icon = if selected && !state.disabled {
            foreground
        } else {
            gray(9)
        };
        TreeViewPalette {
            background,
            foreground,
            icon_color: icon,
            chevron_color: icon,
            typography: crate::typography::label_typography(size),
            font_family: crate::typography::font_family(&self.look),
        }
    }

    fn row_outline(&self, state: InteractionState, _size: ControlSize) -> Option<gpui::Hsla> {
        (state.hovered && !state.disabled).then(|| self.look.resolve_step(ScaleFamily::Color, 8).hsla())
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

/// Live theme adapter; rows resolve current colors when rendered.
/// Notify affected views after mutating the look; see [`Look`].
pub fn tree_view_theme(look: &Look) -> Arc<dyn TreeViewTheme> {
    Arc::new(RadixTreeViewTheme { look: look.clone() })
}

/// SDK row shells retain selection, disclosure, and keyboard behavior.
pub fn tree_view_template<T: Send + Sync + 'static>(look: &Look) -> Arc<dyn TreeViewTemplate<T>> {
    Arc::new(ThemedTreeViewTemplate::new(tree_view_theme(look)))
}

/// Look-owned tree builder using Radix colors and typography.
pub struct TreeView<T: Clone + Send + Sync + 'static> {
    look: Option<Look>,
    builder: TreeViewBuilder<T>,
}

impl<T: Clone + Send + Sync + 'static> TreeView<T> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { look: None, builder: TreeViewBuilder::new(id) }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set initial nodes. Duplicate IDs retain the previous items, as in the SDK.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::tree_view::TreeView<T> {
        let look = crate::look::resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        self.builder.template(tree_view_template(&look)).spawn(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::ThemeMode;

    #[test]
    fn rows_follow_mode_and_custom_accent_changes() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let selected = theme.resolve_row(Default::default(), true, ControlSize::Md);
            assert_eq!(selected.background, Some(look.resolve_step(ScaleFamily::Color, 3).hsla()));
            assert_eq!(selected.foreground, look.resolve_step(ScaleFamily::Color, 12).hsla());
            let before = selected.background;
            look.set_accent_seed(gpui::rgb(0xd6409f).into());
            let custom = theme.resolve_row(Default::default(), true, ControlSize::Md);
            assert_ne!(custom.background, before);
            assert_eq!(custom.background, Some(look.resolve_step(ScaleFamily::Color, 3).hsla()));
            look.set_palettes(crate::Accent::Indigo, crate::Gray::Auto);
        }
    }

    #[test]
    fn hover_outlines_without_replacing_the_selection_fill() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let hovered = InteractionState { hovered: true, ..Default::default() };
            assert_eq!(theme.resolve_row(hovered, false, ControlSize::Md).background, None);
            assert_eq!(
                theme.resolve_row(hovered, true, ControlSize::Md).background,
                theme.resolve_row(Default::default(), true, ControlSize::Md).background,
            );
            assert_eq!(
                theme.row_outline(hovered, ControlSize::Md),
                Some(look.resolve_step(ScaleFamily::Color, 8).hsla()),
            );
            assert_eq!(theme.row_outline(Default::default(), ControlSize::Md), None);
            assert_eq!(theme.row_outline(InteractionState { disabled: true, ..hovered }, ControlSize::Md), None);
        }
    }

    #[test]
    fn disabled_overrides_selection_and_interaction() {
        let look = Look::built_in();
        let theme = tree_view_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let state =
                InteractionState { disabled: true, hovered: true, pressed: true, focused: true, invalid: false };
            let row = theme.resolve_row(state, true, ControlSize::Md);
            assert_eq!(row.background, None);
            assert_eq!(row.foreground, look.resolve_step(ScaleFamily::Gray, 11).hsla());
        }
    }
}
