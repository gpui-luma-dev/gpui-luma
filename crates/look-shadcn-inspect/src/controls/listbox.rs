//! Inspect metadata for `listbox`.

use luma::theme::{InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct ListBoxListInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ListBoxRowInspectPalette {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn inspect_listbox_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
    _focused: bool,
) -> ListBoxListInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "listbox_list_inspect");
    let colors = luma_look_shadcn::tables::resolve_listbox_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| luma_look_shadcn::tables::ListBoxListColorTable::fallback());
    ListBoxListInspectPalette { background: colors.background, border: colors.border, divider: colors.divider }
}

pub fn inspect_listbox_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ListBoxRowInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "listbox_row_inspect");
    let colors =
        luma_look_shadcn::tables::resolve_listbox_row_colors(&resolver, state.disabled, state.focused, state.layer())
            .unwrap_or_else(|_| luma_look_shadcn::tables::ListBoxRowColorTable::fallback());
    ListBoxRowInspectPalette { background: colors.background, label_color: colors.label_color }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_catalog;

    #[test]
    fn listbox_metadata_covers_list_and_row_tables() {
        assert_eq!(
            luma_look_shadcn::stylesheet::resolve_listbox_list_colors_metadata(luma_look_shadcn::embedded_stylesheet())
                .len(),
            2
        );
        assert_eq!(
            luma_look_shadcn::stylesheet::resolve_listbox_row_colors_metadata(luma_look_shadcn::embedded_stylesheet())
                .len(),
            6
        );
    }

    #[test]
    fn inspect_hovered_row_uses_accent_layer() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_listbox_row_color_palette(
            &mode,
            ThemeMode::Light,
            luma::theme::InteractionState { hovered: true, ..Default::default() },
        );
        assert!(palette.background.value.a > 0.0);
    }
}
