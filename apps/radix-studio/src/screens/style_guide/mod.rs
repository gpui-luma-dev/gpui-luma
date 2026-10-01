//! Style Guide tab screen.

mod state;
pub use state::State;

mod avatars;
mod badges;
mod buttons;
mod cards;
mod checkboxes;
mod context_menus;
mod high_contrast;
mod matrix_grid;
mod menus;
mod palettes;
mod preview_handlers;
mod progress;
mod radios;
mod sliders;
mod states;
mod switches;
mod table;
mod tabs;
mod textareas;
mod textfields;

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, IntoElement, Window, div, prelude::*, px};
use luma::controls::tabs::Tabs;
use luma::controls::tree_view::TreeView;
use luma::vstack;
use luma_look_radix::{Look, ScaleFamily, SemanticRole};

use super::section::section;

pub use tabs::TabsExamples;

#[derive(Clone)]
pub struct PreviewTabs {
    pub avatars: Entity<Tabs>,
    pub badges: Entity<Tabs>,
    pub examples: TabsExamples,
    pub buttons: Entity<Tabs>,
    pub checkboxes: Entity<Tabs>,
    pub radios: Entity<Tabs>,
    pub switches: Entity<Tabs>,
    pub textfields: Entity<Tabs>,
    pub textareas: Entity<Tabs>,
    pub sliders: Entity<Tabs>,
    pub progress: Entity<Tabs>,
}

pub fn page(
    look: &Arc<Look>,
    tabs: PreviewTabs,
    tree: TreeView<()>,
    disabled_tree: TreeView<()>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
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
            "Avatar",
            "Image, icon, or initials; Solid and Soft, with nine sizes and five radius options.",
            fg, muted, border,
            avatars::tabbed(look, tabs.avatars, muted, border, cx),
        ),
        section(
            "Badge",
            "Solid, Soft, Surface, and Outline; accent and gray, each paired with high contrast.",
            fg, muted, border,
            badges::tabbed(look, tabs.badges, muted, border, cx),
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
            "Card",
            "Surface, Classic, and Ghost containers in sizes 1–3.",
            fg, muted, border,
            cards::matrix(look, muted),
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
            "Context Menu",
            "Template preview · Solid and Soft, in accent and gray.",
            fg, muted, border,
            context_menus::preview(look, fg),
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
        section(
            "Progress",
            "Surface and Soft; sizes 1–3, high contrast, and all five radius options.",
            fg, muted, border,
            progress::tabbed(look, tabs.progress, fg, muted, border, window, cx),
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
            "Slider",
            format!("Radix variants on {accent}, across interaction states."),
            fg,
            muted,
            border,
            sliders::tabbed(look, tabs.sliders, fg, muted, border, window, cx),
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
            "Tabs",
            "Underline tabs across colors and sizes. Select tabs or use arrow keys; the Colors sample tab is disabled.",
            fg,
            muted,
            border,
            tabs.examples.render(look, muted, border, cx),
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
            "Text Field",
            format!("Radix variants on {accent}, across interaction states including invalid."),
            fg,
            muted,
            border,
            textfields::tabbed(look, tabs.textfields, fg, muted, border, window, cx),
        ),
        section(
            "Tree View",
            "Select rows and expand Grid. Use arrow keys to navigate. Selection uses accent 3/12; hover outlines use accent 8; pressed uses accent 5.",
            fg,
            muted,
            border,
            div().flex().flex_wrap().gap(px(24.0)).children([
                vstack! {
                    gap=8;
                    div().text_sm().text_color(muted).child("Interactive"),
                    super::shared::tree_view::panel(tree, look),
                }.w(px(320.0)).into_any_element(),
                vstack! {
                    gap=8;
                    div().text_sm().text_color(muted).child("Disabled"),
                    super::shared::tree_view::panel(disabled_tree, look),
                }.w(px(320.0)).into_any_element(),
            ]).into_any_element(),
        ),
    }
    .w_full()
    .into_any_element()
}
