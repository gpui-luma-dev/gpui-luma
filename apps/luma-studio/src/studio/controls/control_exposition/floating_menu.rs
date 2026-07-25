use std::sync::Arc;

use gpui::{App, ClickEvent, Context, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::floating_menu::{
    FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuLook, render_floating_menu,
};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::state::MenuPath;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const EVENT_SPECS: &[EventReferenceSpec] = &[EventReferenceSpec {
    event: "(template)",
    trigger: "render_floating_menu item clicks",
    notes: "Floating menu is a template primitive; parent controls emit Select/OpenChanged (PopupMenu, ContextMenu).",
}];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "FloatingMenuLook",
        surface: "Type",
        notes: "Resolved menu chrome tokens — background, item height, typography.",
    },
    PublicInterfaceSpec {
        symbol: "render_floating_menu(...)",
        surface: "Factory",
        notes: "Renders a menu item list with hover/click handler slots.",
    },
    PublicInterfaceSpec {
        symbol: "FloatingMenuState",
        surface: "Type",
        notes: "Keyboard navigation state machine for active path and open submenu.",
    },
    PublicInterfaceSpec {
        symbol: "look.floating_menu_theme()",
        surface: "Look",
        notes: "ShadcnLook floating menu theme used by popup and context menus.",
    },
];

pub struct FloatingMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
}

impl FloatingMenuControlExposition {
    pub fn new(_cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("floating-menu").expect("floating-menu catalog entry");
        Self { look, entry }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for FloatingMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let menu_look = self.look.floating_menu_theme().resolve();
            let preview = div()
                .w_full()
                .flex()
                .flex_wrap()
                .items_start()
                .justify_center()
                .gap(px(16.0))
                .child(render_menu_sample("Standard", &menu_look, &default_items(), None))
                .child(render_menu_sample("Hover / active item", &menu_look, &default_items(), Some(MenuPath::Root(1))))
                .child(render_menu_sample("Disabled item", &menu_look, &disabled_items(), None))
                .child(render_menu_sample("Submenu affordance", &menu_look, &submenu_items(), None));

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn render_menu_sample(
    label: &'static str,
    look: &FloatingMenuLook,
    items: &[MenuItem],
    active_path: Option<MenuPath>,
) -> gpui::AnyElement {
    let id = SharedString::from(format!("controls-doc-floating-menu-{}", sample_id(label)));
    let root_count = items.len();
    let chrome = look.background;

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(render_floating_menu(
            &id,
            items,
            None,
            active_path,
            look.clone(),
            noop_hovers(root_count),
            noop_clicks(root_count),
        ))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(chrome).opacity(0.72).child(label))
        .into_any_element()
}

fn default_items() -> [MenuItem; 3] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
    ]
}

fn disabled_items() -> [MenuItem; 3] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download).enabled(false),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2),
    ]
}

fn submenu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("inspect").label("Inspect"),
    ]
}

fn sample_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn noop_hovers(count: usize) -> Vec<FloatingMenuHoverHandler> {
    (0..count)
        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as FloatingMenuHoverHandler)
        .collect()
}

fn noop_clicks(count: usize) -> Vec<FloatingMenuClickHandler> {
    (0..count)
        .map(|_| Box::new(|_: &ClickEvent, _: &mut Window, _: &mut App| {}) as FloatingMenuClickHandler)
        .collect()
}
