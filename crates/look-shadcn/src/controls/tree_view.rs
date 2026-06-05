//! Tree view row tokens (shadcn / tweakcn sidebar branch styling):
//!
//! | Part           | Token                |
//! |----------------|----------------------|
//! | Row label      | `sidebar-foreground` |
//! | Row hover bg   | `sidebar-accent`     |
//! | Row pressed bg | `accent` (pressed)   |
//! | Disabled label | `muted-foreground`   |
//! | Icon / chevron | `sidebar-foreground` |

use gpui_luma::controls::tree_view::TreeViewPalette;
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::appearance_context::AppearanceContext;
use crate::mode::ShadcnModeTokens;
use super::navigation_sidebar::navigation_sidebar_branch_appearance;

pub(crate) fn tree_view_row_palette(
    mode: &ShadcnModeTokens,
    _selected: bool,
    state: InteractionState,
) -> TreeViewPalette {
    let appearance = navigation_sidebar_branch_appearance(mode, state, ControlSize::Md);
    let ctx = AppearanceContext::new(mode, ThemeMode::Light, state);
    let typography = ctx.typography();

    TreeViewPalette {
        background: appearance.background,
        foreground: appearance.foreground,
        icon_color: appearance.icon_color,
        chevron_color: appearance.icon_color,
        adorner: None,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}
