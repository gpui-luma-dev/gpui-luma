//! Inspect metadata for `listbox`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

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

#[derive(Clone, Debug)]
pub struct ListBoxInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_gap: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
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

pub fn inspect_listbox_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ListBoxInspectMetrics {
    use luma::theme::ListRowScale;

    use luma_look_shadcn::catalog::SpacingField;
    use crate::metrics::{derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric};

    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let list_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = crate::metrics::control_size_key(size);

    ListBoxInspectMetrics {
        radius: radius_metric(catalog, size, metrics.radius(size)),
        padding_x: derived_metric("listbox padding x", 6.0),
        padding_y: derived_metric(
            format!("{size_key} list padding y = padding_y × 0.5"),
            metrics.padding_y(size) * 0.5,
        ),
        row_gap: derived_metric(format!("{size_key} list row gap = padding_y × 0.25"), metrics.padding_y(size) * 0.25),
        row_min_height: scaffold_control_metric(size_key, "row_min_height", list_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, list_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, list_scale.padding_y),
    }
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
