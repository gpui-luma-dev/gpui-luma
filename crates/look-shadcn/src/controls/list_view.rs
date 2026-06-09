//! List view — input surface + accent whisper hover, muted selected rows.
//!
//! | Row state   | Token / effect        |
//! |-------------|-----------------------|
//! | Default     | transparent           |
//! | Hover       | `accent` at 40% alpha |
//! | Selected    | `muted`               |
//! | Keyboard active | `muted`           |

use gpui::hsla;

use gpui_luma::controls::list_view::{ListViewAppearance, ListViewRowPalette};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, ThemeMode};

use crate::color::with_alpha;
use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};

use gpui_luma_look_shadcn_macros::declare_look_table;

const ROW_HOVER_ACCENT_ALPHA: f32 = 0.4;

#[derive(Clone, Debug)]
pub struct ListViewSurfaceColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

impl ListViewSurfaceColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            header_background: ResolvedColor::transparent(),
            header_label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

declare_look_table! {
    name: resolve_list_view_surface_colors,
    inputs: {
        enabled: bool,
    },
    output: ListViewSurfaceColorTable { background, border, header_background, header_label_color },
    matrix: [
        [true]  => "background" | "input" | "muted" | "muted-foreground",
        [false] => "muted" | "input" | "muted" | "muted-foreground",
    ]
}

#[derive(Clone, Debug)]
pub struct ListViewRowColorTable {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

impl ListViewRowColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            label_color: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

declare_look_table! {
    name: resolve_list_view_row_colors,
    inputs: {
        selected: bool,
        focused: bool,
        disabled: bool,
        layer: InteractionLayer,
    },
    output: ListViewRowColorTable { background, label_color, divider },
    matrix: [
        [true] | [false] | [true] | [_] => "transparent" | "muted-foreground" | "border",
        [false] | [false] | [true] | [_] => "transparent" | "muted-foreground" | "border",
        [false] | [true] | [true] | [_] => "muted" | "muted-foreground" | "border",
        [true] | [true] | [true] | [_] => "muted" | "muted-foreground" | "border",

        [true] | [_] | [false] | [_] => "muted" | "foreground" | "border",
        [false] | [true] | [false] | [_] => "muted" | "foreground" | "border",

        [false] | [false] | [false] | [InteractionLayer::Default] => "transparent" | "foreground" | "border",
        [false] | [false] | [false] | [InteractionLayer::Hovered] => "@accent_whisper_40" | "foreground" | "border",
        [false] | [false] | [false] | [InteractionLayer::Pressed] => "@accent_whisper_pressed_40" | "foreground" | "border",
        [false] | [false] | [false] | [InteractionLayer::Disabled] => "transparent" | "foreground" | "border",
    ]
}

pub fn list_view_appearance(
    mode: &ShadcnModeTokens,
    enabled: bool,
    focused: bool,
    size: ControlSize,
) -> ListViewAppearance {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, InteractionState::default());
    if mode.catalog.tokens.is_empty() {
        list_view_appearance_from_palette(&ctx, enabled, focused, size)
    } else {
        list_view_appearance_from_catalog(&ctx, enabled, focused, size)
            .unwrap_or_else(|err| panic!("list view appearance properties: {err}"))
    }
}

pub fn list_view_row_palette(
    mode: &ShadcnModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> ListViewRowPalette {
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    if mode.catalog.tokens.is_empty() {
        list_view_row_from_palette(&ctx, selected, size)
    } else {
        list_view_row_from_catalog(&ctx, selected, size).unwrap_or_else(|err| panic!("list view row properties: {err}"))
    }
}

pub fn list_view_appearance_from_palette(
    ctx: &AppearanceContext,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> ListViewAppearance {
    let palette = ctx.palette();
    let metrics = ctx.metrics();

    ListViewAppearance {
        background: if enabled {
            palette.app_background
        } else {
            palette.disabled_background
        },
        border: palette.input_background,
        header_background: if enabled {
            palette.muted_background
        } else {
            palette.disabled_background
        },
        header_label_color: if enabled {
            palette.app_muted_foreground
        } else {
            palette.disabled_foreground
        },
        header_typography: ctx.typography().text.caption,
        radius: metrics.radius(size),
        padding_x: 0.0,
        padding_y: metrics.padding_y(size) * 0.5,
    }
}

pub fn list_view_appearance_from_catalog(
    ctx: &AppearanceContext,
    enabled: bool,
    _focused: bool,
    size: ControlSize,
) -> anyhow::Result<ListViewAppearance> {
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "list_view");
    let colors =
        resolve_list_view_surface_colors(&resolver, enabled).unwrap_or_else(|_| ListViewSurfaceColorTable::fallback());

    Ok(ListViewAppearance {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        header_background: colors.header_background.hsla(),
        header_label_color: colors.header_label_color.hsla(),
        header_typography: ctx.typography().text.caption,
        radius: metrics.radius(size),
        padding_x: 0.0,
        padding_y: metrics.padding_y(size) * 0.5,
    })
}

pub fn list_view_row_from_palette(ctx: &AppearanceContext, selected: bool, _size: ControlSize) -> ListViewRowPalette {
    let state = ctx.state;
    let palette = ctx.palette();
    let typography = ctx.typography();
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);

    let selected_background = palette.muted_background;
    let hover_background = with_alpha(palette.accent_background, ROW_HOVER_ACCENT_ALPHA);

    let background = if state.disabled && !state.focused {
        transparent
    } else if selected || state.focused {
        selected_background
    } else {
        match state.layer() {
            InteractionLayer::Disabled => transparent,
            InteractionLayer::Pressed => crate::resolve::resolve_whisper_pressed(hover_background, ctx.theme_mode),
            InteractionLayer::Hovered => hover_background,
            InteractionLayer::Default => transparent,
        }
    };

    let label_color = if state.disabled {
        palette.disabled_foreground
    } else {
        palette.app_foreground
    };

    ListViewRowPalette {
        background,
        label_color,
        divider: palette.input_background,
        adorner: None,
        label_typography: typography.text.label,
    }
}

fn list_view_row_from_catalog(
    ctx: &AppearanceContext,
    selected: bool,
    _size: ControlSize,
) -> anyhow::Result<ListViewRowPalette> {
    let state = ctx.state;
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "list_view_row");
    let colors = resolve_list_view_row_colors(&resolver, selected, state.focused, state.disabled, state.layer())
        .unwrap_or_else(|_| ListViewRowColorTable::fallback());

    Ok(ListViewRowPalette {
        background: colors.background.hsla(),
        label_color: colors.label_color.hsla(),
        divider: colors.divider.hsla(),
        adorner: None,
        label_typography: typography.text.label,
    })
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
    use super::{
        ROW_HOVER_ACCENT_ALPHA, list_view_row_palette, resolve_list_view_row_colors_metadata,
        resolve_list_view_surface_colors_metadata,
    };

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
    fn list_view_row_hover_uses_accent_whisper_and_selected_uses_muted() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hover = list_view_row_palette(
            &mode,
            false,
            InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );
        let selected = list_view_row_palette(
            &mode,
            true,
            InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );

        let accent = catalog.color("accent").expect("accent");
        assert_eq!(hover.background, with_alpha(accent, ROW_HOVER_ACCENT_ALPHA));
        assert_eq!(selected.background, resolve_color(&catalog, "muted").expect("muted"));
    }

}
