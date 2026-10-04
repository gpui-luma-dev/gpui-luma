//! Table — input surface + accent whisper hover, muted selected rows.
//!
//! | Row state   | Token / effect        |
//! |-------------|-----------------------|
//! | Default     | transparent           |
//! | Hover       | `accent` at 40% alpha |
//! | Selected    | `muted`               |
//! | Keyboard active | `muted`           |

use gpui_luma::controls::table::{TableLook, TableRowPalette};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, find_table_row_color_rule, find_table_surface_color_rule, resolve_table_row_color_rule,
    resolve_table_surface_color_rule, resolve_stylesheet_metric,
};

#[derive(Clone, Debug)]
pub struct TableSurfaceColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

impl TableSurfaceColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            header_background: ResolvedColor::transparent(),
            header_label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_table_surface_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<TableSurfaceColorTable> {
    resolve_table_surface_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_table_surface_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<TableSurfaceColorTable> {
    let rule = find_table_surface_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching table surface color rule"))?;
    let colors = resolve_table_surface_color_rule(resolver, rule)?;
    Ok(TableSurfaceColorTable {
        background: colors.background,
        border: colors.border,
        header_background: colors.header_background,
        header_label_color: colors.header_label_color,
    })
}

#[derive(Clone, Debug)]
pub struct TableRowColorTable {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

impl TableRowColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            label_color: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_table_row_colors(
    resolver: &LookResolver<'_>,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TableRowColorTable> {
    resolve_table_row_colors_with_stylesheet(resolver, resolver.stylesheet(), selected, focused, disabled, layer)
}

pub fn resolve_table_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TableRowColorTable> {
    let rule = find_table_row_color_rule(stylesheet, selected, focused, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching table row color rule"))?;
    let colors = resolve_table_row_color_rule(resolver, rule, layer)?;
    Ok(TableRowColorTable { background: colors.background, label_color: colors.label_color, divider: colors.divider })
}

pub fn table_look(mode: &ShadcnModeTokens, enabled: bool, _focused: bool, size: ControlSize) -> TableLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "table").with_stylesheet(mode.stylesheet());
    let colors =
        resolve_table_surface_colors(&resolver, enabled).unwrap_or_else(|_| TableSurfaceColorTable::fallback());
    let surface_metrics = mode.stylesheet().table.surface.metrics_for_size(size);
    let radius = surface_metrics
        .and_then(|rule| resolve_stylesheet_metric(&rule.radius, metrics, size))
        .unwrap_or(metrics.radius(size));
    let radius = surface_metrics
        .and_then(|rule| rule.max_radius.as_deref())
        .and_then(|max| resolve_stylesheet_metric(max, metrics, size))
        .map_or(radius, |max| radius.min(max));

    let mut look = TableLook {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        header_background: colors.header_background.hsla(),
        header_label_color: colors.header_label_color.hsla(),
        header_typography: ctx.typography().text.caption,
        radius,
        padding_x: 0.0,
        padding_y: metrics.padding_y(size) * 0.5,
    };
    let geometry = mode.stylesheet().common.table.resolve_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        gpui_luma::theme::stylesheet::TableGeometry { padding_x: look.padding_x, padding_y: look.padding_y },
    );
    look.padding_x = geometry.padding_x.value_px;
    look.padding_y = geometry.padding_y.value_px;

    look
}

pub fn table_row_palette(
    mode: &ShadcnModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> TableRowPalette {
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "table_row").with_stylesheet(mode.stylesheet());
    let colors = resolve_table_row_colors(&resolver, selected, state.focused, state.disabled, state.layer())
        .unwrap_or_else(|_| TableRowColorTable::fallback());

    let mut label_typography = typography.text.label;
    super::apply_button_metrics_typography(&mut label_typography, mode, size);

    TableRowPalette {
        background: colors.background.hsla(),
        label_color: colors.label_color.hsla(),
        divider: colors.divider.hsla(),
        label_typography,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use gpui_luma::theme::ThemeMode;

    use gpui_luma::theme::{ControlSize, InteractionState};

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;
    use crate::resolve::resolve_color;
    use super::table_row_palette;

    const ROW_HOVER_ACCENT_ALPHA: f32 = 0.4;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn table_row_hover_uses_accent_whisper_and_selected_uses_muted() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hover =
            table_row_palette(&mode, false, InteractionState { hovered: true, ..Default::default() }, ControlSize::Md);
        let selected =
            table_row_palette(&mode, true, InteractionState { hovered: true, ..Default::default() }, ControlSize::Md);

        let accent = catalog.color("accent").expect("accent");
        assert_eq!(hover.background, with_alpha(accent, ROW_HOVER_ACCENT_ALPHA));
        assert_eq!(selected.background, resolve_color(&catalog, "muted").expect("muted"));
    }
}
