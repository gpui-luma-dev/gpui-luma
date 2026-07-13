use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, FontWeight, IntoElement, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::floating_menu::{
    FloatingMenuLook, FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuState, FloatingMenuStepDirection,
    FloatingMenuTheme, render_floating_menu,
};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::state::MenuPath;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_floating_menu_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

#[derive(Clone)]
pub(in crate::gallery) struct FloatingMenuPane {
    menu_theme: Arc<dyn FloatingMenuTheme>,
    inspector: Entity<ColorInspectorShell>,
}

impl FloatingMenuPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "floating-menu-inspector-tree",
            look.clone(),
            build_floating_menu_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "floating-menu-inspector",
                "floating-menu-inspector-split",
                "floating-menu-inspector-detail",
                build_floating_menu_inspect_tree,
                cx,
            )
        });
        Self { menu_theme: look.floating_menu_theme(), inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();
        let menu_look = self.menu_theme.resolve();

        gallery_pane_with_inspector(
            "floating-menu",
            "Floating Menu",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .justify_center()
                        .gap(px(16.0))
                        .child(render_state_sample("Standard", &menu_look, chrome.muted_text, &default_items(), None))
                        .child(render_state_sample(
                            "Hover / active item",
                            &menu_look,
                            chrome.muted_text,
                            &default_items(),
                            Some(MenuPath::Root(1)),
                        ))
                        .child(render_state_sample(
                            "Disabled item",
                            &menu_look,
                            chrome.muted_text,
                            &disabled_items(),
                            None,
                        ))
                        .child(render_state_sample(
                            "Submenu affordance",
                            &menu_look,
                            chrome.muted_text,
                            &submenu_items(),
                            None,
                        )),
                )
                .child(div().text_sm().text_color(chrome.muted_text).child(floating_menu_state_machine_snapshot()))
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

fn render_state_sample(
    label: &'static str,
    look: &FloatingMenuLook,
    label_color: gpui::Hsla,
    items: &[MenuItem],
    active_path: Option<MenuPath>,
) -> AnyElement {
    let id = SharedString::from(format!("floating-menu-sample-{}", sample_id(label)));
    let root_count = items.len();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(7.0))
        .child(render_floating_menu(
            &id,
            items,
            None,
            active_path,
            look.clone(),
            noop_hovers(root_count),
            noop_clicks(root_count),
        ))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}

fn floating_menu_state_machine_snapshot() -> String {
    let items = submenu_items().into_iter().collect::<Vec<_>>();
    let mut state = FloatingMenuState::default();
    let _ = state.open_with(Some(MenuPath::Root(0)));
    let _ = state.step(&items, FloatingMenuStepDirection::Next);
    let _ = state.open_active_submenu(&items);

    format!("State machine demo: active_path={:?}, open_submenu={:?}", state.active_path(), state.open_submenu())
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
    (0..count).map(|_| Box::new(noop_hover) as FloatingMenuHoverHandler).collect()
}

fn noop_clicks(count: usize) -> Vec<FloatingMenuClickHandler> {
    (0..count).map(|_| Box::new(noop_click) as FloatingMenuClickHandler).collect()
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
