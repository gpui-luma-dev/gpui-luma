//! Style Guide tab screen.

mod buttons;
mod checkboxes;
mod high_contrast;
mod matrix_grid;
mod menus;
mod palettes;
mod preview_handlers;
mod radios;
mod sliders;
mod states;
mod switches;
mod table;
mod textareas;
mod textfields;

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, Window, prelude::*};
use luma::controls::tabs::Tabs;
use luma::vstack;
use luma_look_radix::{Look, ScaleFamily, SemanticRole};

use super::section::section;

pub struct PreviewTabs {
    pub buttons: Entity<Tabs>,
    pub checkboxes: Entity<Tabs>,
    pub radios: Entity<Tabs>,
    pub switches: Entity<Tabs>,
    pub textfields: Entity<Tabs>,
    pub textareas: Entity<Tabs>,
    pub sliders: Entity<Tabs>,
}

pub fn page(look: &Arc<Look>, tabs: PreviewTabs, window: &mut Window, cx: &mut App) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();
    let accent = palettes::title_case(look.palette_label(ScaleFamily::Color));
    let gray = palettes::title_case(look.palette_label(ScaleFamily::Gray));

    vstack! {
        gap=28;
        section(
            "Palette",
            format!("Every matrix below paints from {accent} (accent) and {gray} (gray)."),
            fg,
            muted,
            border,
            palettes::indicator(look, muted),
        ),
        section(
            "Button",
            format!("Radix variants on {accent}, across interaction states."),
            fg,
            muted,
            border,
            buttons::tabbed(look, tabs.buttons, fg, muted, border, window, cx),
        ),
        section(
            "Button · High contrast",
            format!("Gray tone ({gray}) with Radix highContrast."),
            fg,
            muted,
            border,
            high_contrast::strip(look, muted, window, cx),
        ),
        section(
            "Checkbox",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs unchecked and checked."
            ),
            fg,
            muted,
            border,
            checkboxes::tabbed(look, tabs.checkboxes, fg, muted, border, window, cx),
        ),
        section(
            "Radio",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs unselected and selected."
            ),
            fg,
            muted,
            border,
            radios::tabbed(look, tabs.radios, fg, muted, border, window, cx),
        ),
        section(
            "Switch",
            format!(
                "Radix variants on {accent}, across interaction states. Each cell pairs off and on."
            ),
            fg,
            muted,
            border,
            switches::tabbed(look, tabs.switches, fg, muted, border, window, cx),
        ),
        section(
            "Text Field",
            format!("Radix variants on {accent}, across interaction states including invalid."),
            fg,
            muted,
            border,
            textfields::tabbed(look, tabs.textfields, fg, muted, border, window, cx),
        ),
        section(
            "Text Area",
            format!("Radix variants on {accent}, across interaction states including invalid."),
            fg,
            muted,
            border,
            textareas::tabbed(look, tabs.textareas, fg, muted, border, window, cx),
        ),
        section(
            "Slider",
            format!("Radix variants on {accent}, across interaction states."),
            fg,
            muted,
            border,
            sliders::tabbed(look, tabs.sliders, fg, muted, border, window, cx),
        ),
        section(
            "Menu",
            format!(
                "Popup menu triggers on {accent}, with a {gray}-locked pair for the neutral slot."
            ),
            fg,
            muted,
            border,
            menus::matrix(look, &menus::RADIX_VARIANTS, fg, muted, window, cx),
        ),
    }
    .w_full()
    .into_any_element()
}
