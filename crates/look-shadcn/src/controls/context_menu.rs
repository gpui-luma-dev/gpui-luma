//! Context menu property mappings:
//!
//! | Part   | Token                              |
//! |--------|------------------------------------|
//! | Target | ghost (accent-foreground on hover) |
//! | Menu   | floating menu surface              |

use gpui_luma::controls::context_menu::ContextMenuLook;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use super::floating_menu::floating_menu_look;
use crate::focus::focus_ring_color;
use crate::resolve::{resolve_color, resolve_ghost_trigger_background, resolve_ghost_trigger_foreground};
use crate::mode::ShadcnModeTokens;

pub fn context_menu_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, state: InteractionState) -> ContextMenuLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let size = ControlSize::Md;
    let layer = state.layer();

    ContextMenuLook {
        target_background: resolve_ghost_trigger_background(catalog, layer)
            .unwrap_or_else(|err| panic!("context menu properties: {err}")),
        target_foreground: resolve_ghost_trigger_foreground(catalog, layer, state.disabled)
            .unwrap_or_else(|err| panic!("context menu properties: {err}")),
        target_border: resolve_color(catalog, "border").unwrap_or_else(|err| panic!("context menu properties: {err}")),
        focus_ring: state
            .focused
            .then(|| focus_ring_color(catalog))
            .transpose()
            .unwrap_or_else(|err| panic!("context menu properties: {err}")),
        target_typography: typography.text.label,
        target_radius: metrics.radius(size),
        target_padding_x: metrics.padding_x(size),
        target_padding_y: metrics.padding_y(size),
        target_min_width: 200.0,
        floating_menu: floating_menu_look(ctx.tokens, ctx.theme_mode, size),
    }
}
