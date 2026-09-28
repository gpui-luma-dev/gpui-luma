//! Inspect metadata for `tabs`.

use luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct TabsItemInspectPalette {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsListInspectPalette {
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct TabsInspectMetrics {
    pub list_radius: crate::ResolvedMetric,
    pub list_gap: crate::ResolvedMetric,
    pub list_padding: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub indicator_height: crate::ResolvedMetric,
}

pub fn inspect_tabs_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    active: bool,
    state: InteractionState,
) -> TabsItemInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_item_inspect");
    let colors = crate::tables::resolve_tabs_item_colors(&resolver, active, state.layer(), state.focused)
        .unwrap_or_else(|_| crate::tables::TabsItemColorTable::fallback());

    TabsItemInspectPalette { label_color: colors.label_color, indicator: colors.indicator }
}

pub fn inspect_tabs_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> TabsListInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "tabs_list_inspect");
    let colors = crate::tables::resolve_tabs_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| crate::tables::TabsListColorTable::fallback());
    let background = if enabled {
        None
    } else {
        Some(colors.disabled_background)
    };

    TabsListInspectPalette { background }
}

pub fn inspect_tabs_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TabsInspectMetrics {
    let table = crate::tables::metrics::resolve_tabs_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::TabsMetricTable> for TabsInspectMetrics {
    fn from(table: crate::tables::metrics::TabsMetricTable) -> Self {
        Self {
            list_radius: table.list_radius,
            list_gap: table.list_gap,
            list_padding: table.list_padding,
            item_padding_x: table.item_padding_x,
            item_height: table.item_height,
            item_radius: table.item_radius,
            indicator_height: table.indicator_height,
        }
    }
}
