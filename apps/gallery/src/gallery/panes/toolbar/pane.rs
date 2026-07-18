use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FocusHandle, Focusable, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::control_group::ControlGroupArrowPolicy;
use gpui_luma::controls::toolbar::{ToolbarControl, ToolbarItem};
use gpui_luma::focus::EscapeFocus;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_toolbar_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{InspectorToggleRegistry, gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
struct CommandButton {
    label: &'static str,
    entity: Entity<Button<()>>,
}

#[derive(Clone)]
struct ToggleButton {
    label: &'static str,
    entity: Entity<Button<bool>>,
}

#[derive(Clone)]
struct ToolbarMenu {
    label: &'static str,
    entity: Entity<PopupMenu>,
}

#[derive(Clone)]
struct ToolbarSelector {
    label: &'static str,
    entity: Entity<Selector>,
}

#[derive(Clone)]
pub(in crate::gallery) struct ToolbarPane {
    toolbar: ToolbarControl,
    command_buttons: Vec<CommandButton>,
    toggle_buttons: Vec<ToggleButton>,
    menus: Vec<ToolbarMenu>,
    selectors: Vec<ToolbarSelector>,
    search_field: TextField,
    inspector: Entity<ColorInspectorShell>,
    status: String,
}

impl ToolbarPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("toolbar-inspector-tree", look.clone(), build_toolbar_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "toolbar-inspector",
                "toolbar-inspector-split",
                "toolbar-inspector-detail",
                build_toolbar_inspect_tree,
                cx,
            )
        });

        let assistant_menu = icon_menu(
            &look,
            "toolbar-assistant",
            "Assistant",
            LucideIcon::WandSparkles,
            [
                MenuItem::new("rewrite").label("Rewrite"),
                MenuItem::new("summarize").label("Summarize"),
                MenuItem::new("continue").label("Continue writing"),
            ],
            cx,
        );
        let style_selector = look
            .selector("toolbar-style")
            .label("Paragraph")
            .tab_stop(false)
            .items([
                SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                SelectorItem::new("heading").label("Heading").icon(LucideIcon::Type),
                SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
            ])
            .selected_id("paragraph")
            .spawn(cx);
        let color_menu = look
            .popup_menu("toolbar-color")
            .label("A")
            .ghost()
            .tab_stop(false)
            .items([
                MenuItem::new("default").label("Default color"),
                MenuItem::new("muted").label("Muted"),
                MenuItem::new("accent").label("Accent"),
            ])
            .spawn(cx);
        let alignment_menu = icon_menu(
            &look,
            "toolbar-align",
            "Alignment",
            LucideIcon::TextAlignStart,
            [
                MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
            ],
            cx,
        );
        let more_menu = icon_menu(
            &look,
            "toolbar-more",
            "More",
            LucideIcon::Ellipsis,
            [
                MenuItem::new("clear").label("Clear formatting"),
                MenuItem::new("divider").label("Insert divider"),
                MenuItem::new("comment").label("Add comment"),
            ],
            cx,
        );

        let bold = icon_toggle(&look, "toolbar-bold", LucideIcon::Bold, cx);
        let italic = icon_toggle(&look, "toolbar-italic", LucideIcon::Italic, cx);
        let underline = icon_toggle(&look, "toolbar-underline", LucideIcon::Underline, cx);

        let link = icon_button(&look, "toolbar-link", LucideIcon::Link, cx);
        let image = icon_button(&look, "toolbar-image", LucideIcon::Image, cx);
        let video = icon_button(&look, "toolbar-video", LucideIcon::Video, cx);
        let code = icon_button(&look, "toolbar-code", LucideIcon::Code, cx);

        let search_field = look
            .textfield("toolbar-search")
            .placeholder("Search...")
            .tab_stop(false)
            .spawn(cx);

        let toolbar = look
            .toolbar("toolbar-demo")
            .item(menu_item("assistant", assistant_menu.clone(), cx))
            .separator("assistant-separator")
            .item(selector_item("style", style_selector.clone(), cx))
            .separator("style-separator")
            .item(toggle_item("bold", bold.clone(), cx))
            .item(toggle_item("italic", italic.clone(), cx))
            .item(toggle_item("underline", underline.clone(), cx))
            .item(menu_item("color", color_menu.clone(), cx))
            .separator("format-separator")
            .item(menu_item("alignment", alignment_menu.clone(), cx))
            .separator("insert-separator")
            .item(button_item("link", link.clone(), cx))
            .item(button_item("image", image.clone(), cx))
            .item(button_item("video", video.clone(), cx))
            .item(menu_item("more", more_menu.clone(), cx))
            .separator("code-separator")
            .item(textfield_item("search", search_field.clone(), cx))
            .item(button_item("code", code.clone(), cx))
            .spawn(cx);

        Self {
            toolbar,
            command_buttons: vec![
                CommandButton { label: "Link", entity: link },
                CommandButton { label: "Image", entity: image },
                CommandButton { label: "Video", entity: video },
                CommandButton { label: "Code", entity: code },
            ],
            toggle_buttons: vec![
                ToggleButton { label: "Bold", entity: bold },
                ToggleButton { label: "Italic", entity: italic },
                ToggleButton { label: "Underline", entity: underline },
            ],
            menus: vec![
                ToolbarMenu { label: "Assistant", entity: assistant_menu },
                ToolbarMenu { label: "Color", entity: color_menu },
                ToolbarMenu { label: "Alignment", entity: alignment_menu },
                ToolbarMenu { label: "More", entity: more_menu },
            ],
            selectors: vec![ToolbarSelector { label: "Style", entity: style_selector }],
            search_field,
            inspector,
            status: "Paragraph editing toolbar".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        for command in self.command_buttons.iter().cloned() {
            subscriptions.push(cx.subscribe(&command.entity, move |app, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    app.panes.toolbar.status = format!("{} action", command.label);
                    cx.notify();
                }
            }));
        }

        for toggle in self.toggle_buttons.iter().cloned() {
            subscriptions.push(cx.subscribe(&toggle.entity, move |app, button, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    button.update(cx, |button, cx| {
                        let next = !*button.data();
                        button.set_data(next, cx);
                        app.panes.toolbar.status = format!("{}: {}", toggle.label, if next { "on" } else { "off" });
                    });
                    cx.notify();
                }
            }));
        }

        for menu in self.menus.iter().cloned() {
            subscriptions.push(cx.subscribe(&menu.entity, move |app, _, event: &PopupMenuEvent, cx| {
                let PopupMenuEvent::Select { label, .. } = event;
                app.panes.toolbar.status = format!("{}: {label}", menu.label);
                cx.notify();
            }));
        }

        for selector in self.selectors.iter().cloned() {
            subscriptions.push(cx.subscribe(&selector.entity, move |app, _, event: &SelectorEvent, cx| {
                let SelectorEvent::Change { label, .. } = event;
                app.panes.toolbar.status = format!("{}: {label}", selector.label);
                cx.notify();
            }));
        }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();
        gallery_pane_with_inspector(
            "toolbar",
            "Toolbar",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .child(self.toolbar.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.muted_text)
                        .child(self.status.clone()),
                )
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.toolbar.update(cx, |_, cx| cx.notify());
        for command in &self.command_buttons {
            command.entity.update(cx, |_, cx| cx.notify());
        }
        for toggle in &self.toggle_buttons {
            toggle.entity.update(cx, |_, cx| cx.notify());
        }
        for menu in &self.menus {
            menu.entity.update(cx, |_, cx| cx.notify());
        }
        for selector in &self.selectors {
            selector.entity.update(cx, |_, cx| cx.notify());
        }
        self.search_field.update(cx, |_, cx| cx.notify());
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

fn icon_button(
    look: &Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    icon: LucideIcon,
    cx: &mut Context<GalleryApp>,
) -> Entity<Button<()>> {
    look.ghost_icon_button(id, icon).tab_stop(false).spawn(cx)
}

fn icon_toggle(
    look: &Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    icon: LucideIcon,
    cx: &mut Context<GalleryApp>,
) -> Entity<Button<bool>> {
    look.ghost_toggle(id).content(move |_, _| lucide_glyph(icon)).tab_stop(false).spawn(cx)
}

fn icon_menu(
    look: &Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    label: impl Into<gpui::SharedString>,
    icon: LucideIcon,
    items: impl IntoIterator<Item = MenuItem>,
    cx: &mut Context<GalleryApp>,
) -> Entity<PopupMenu> {
    look.popup_menu(id).label(label).ghost().trigger_icon(icon).tab_stop(false).items(items).spawn(cx)
}

fn button_item(
    id: impl Into<gpui::SharedString>,
    entity: Entity<Button<()>>,
    cx: &mut Context<GalleryApp>,
) -> ToolbarItem {
    let focus = focus_handle_for(&entity, cx);
    ToolbarItem::button(id, move |_, _, _| entity.clone()).focus_handle(focus)
}

fn toggle_item(
    id: impl Into<gpui::SharedString>,
    entity: Entity<Button<bool>>,
    cx: &mut Context<GalleryApp>,
) -> ToolbarItem {
    let focus = focus_handle_for(&entity, cx);
    ToolbarItem::toggle(id, move |_, _, _| entity.clone()).focus_handle(focus)
}

fn menu_item(
    id: impl Into<gpui::SharedString>,
    entity: Entity<PopupMenu>,
    cx: &mut Context<GalleryApp>,
) -> ToolbarItem {
    let focus = focus_handle_for(&entity, cx);
    ToolbarItem::menu(id, move |_, _, _| entity.clone()).focus_handle(focus)
}

fn selector_item(
    id: impl Into<gpui::SharedString>,
    entity: Entity<Selector>,
    cx: &mut Context<GalleryApp>,
) -> ToolbarItem {
    let focus = focus_handle_for(&entity, cx);
    ToolbarItem::hosted(id, move |_, _, _| entity.clone()).focus_handle(focus)
}

fn textfield_item(
    id: impl Into<gpui::SharedString>,
    entity: TextField,
    cx: &mut Context<GalleryApp>,
) -> ToolbarItem {
    let focus = focus_handle_for(&entity, cx);
    ToolbarItem::hosted(id, move |_, _, _| {
        // Absorb Escape at the toolbar item so it does not bubble to the pane focus scope.
        div()
            .child(entity.clone())
            .on_action(|_: &EscapeFocus, _window, cx| {
                cx.stop_propagation();
            })
    })
    .focus_handle(focus)
    .arrow_policy(ControlGroupArrowPolicy::ChildOwnsHorizontalWhenFocused)
}

fn focus_handle_for<T: Focusable>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) -> FocusHandle {
    entity.read(cx).focus_handle(cx)
}
