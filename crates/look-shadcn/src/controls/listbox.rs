//! List box — input surface tokens; accent row hover and keyboard focus.
//!
//! | Part           | Token              |
//! |----------------|--------------------|
//! | List bg        | `background`       |
//! | Disabled list  | `muted`            |
//! | Border         | `input`            |
//! | Divider        | `border`           |
//! | Row hover      | `accent` (layer)   |
//! | Focused row    | `accent`           |
//! | Disabled label | `muted-foreground` |

use luma::controls::listbox::{ListBoxListLook, ListBoxRowPalette};
use luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_listbox_list_color_rule, find_listbox_row_color_rule,
    resolve_listbox_list_color_rule, resolve_listbox_row_color_rule,
};

#[derive(Clone, Debug)]
pub struct ListBoxListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

impl ListBoxListColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_listbox_list_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<ListBoxListColorTable> {
    resolve_listbox_list_colors_with_stylesheet(resolver, embedded_stylesheet(), enabled)
}

pub fn resolve_listbox_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ListBoxListColorTable> {
    let rule = find_listbox_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching listbox list color rule"))?;
    let colors = resolve_listbox_list_color_rule(resolver, rule)?;
    Ok(ListBoxListColorTable { background: colors.background, border: colors.border, divider: colors.divider })
}

#[derive(Clone, Debug)]
pub struct ListBoxRowColorTable {
    pub label_color: ResolvedColor,
    pub background: ResolvedColor,
}

impl ListBoxRowColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), background: ResolvedColor::transparent() }
    }
}

pub fn resolve_listbox_row_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ListBoxRowColorTable> {
    resolve_listbox_row_colors_with_stylesheet(resolver, embedded_stylesheet(), disabled, focused, layer)
}

pub fn resolve_listbox_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    focused: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ListBoxRowColorTable> {
    let rule = find_listbox_row_color_rule(stylesheet, disabled, focused, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching listbox row color rule"))?;
    let colors = resolve_listbox_row_color_rule(resolver, rule, layer)?;
    Ok(ListBoxRowColorTable { label_color: colors.label_color, background: colors.background })
}

pub fn listbox_list_look(mode: &ShadcnModeTokens, enabled: bool, _focused: bool, size: ControlSize) -> ListBoxListLook {
    let ctx = LookContext::new(mode, ThemeMode::Light, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "listbox_list");
    let colors = resolve_listbox_list_colors(&resolver, enabled).unwrap_or_else(|_| ListBoxListColorTable::fallback());

    ListBoxListLook {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        divider: colors.divider.hsla(),
        radius: metrics.radius(size),
        padding_x: 6.0,
        padding_y: metrics.padding_y(size) * 0.5,
        row_gap: metrics.padding_y(size) * 0.25,
    }
}

pub fn listbox_row_palette(
    mode: &ShadcnModeTokens,
    _selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> ListBoxRowPalette {
    let ctx = LookContext::new(mode, ThemeMode::Light, state);
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "listbox_row");
    let colors = resolve_listbox_row_colors(&resolver, state.disabled, state.focused, state.layer())
        .unwrap_or_else(|_| ListBoxRowColorTable::fallback());

    let mut label_typography = typography.text.label;
    super::apply_button_metrics_typography(&mut label_typography, mode, size);

    ListBoxRowPalette { background: colors.background.hsla(), label_color: colors.label_color.hsla(), label_typography }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use luma::theme::{ControlSize, InteractionLayer, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::LookResolver;
    use super::{listbox_list_look, listbox_row_palette, resolve_listbox_row_colors};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn listbox_uses_input_border_and_accent_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let list = listbox_list_look(&mode, true, false, ControlSize::Md);
        let row = listbox_row_palette(
            &mode,
            false,
            luma::theme::InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );

        assert_eq!(list.border, catalog.color("input").expect("input"));
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let expected = resolve_listbox_row_colors(&resolver, false, false, InteractionLayer::Hovered)
            .expect("row colors")
            .background
            .hsla();
        assert_eq!(row.background, expected);
    }
}
