use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::control_group::ControlGroupFocusStrategy;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::selector::SelectorItem;
use gpui_luma::controls::toolbar::{ToolbarControl, ToolbarEvent, ToolbarValue};
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_toolbar_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{InspectorToggleRegistry, gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToolbarPane {
    toolbar: ToolbarControl,
    roving_focus_button: Entity<Button<()>>,
    sequential_focus_button: Entity<Button<()>>,
    inspector: Entity<ColorInspectorShell>,
    status: String,
    focus_mode: &'static str,
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

        let roving_focus_button = look.ghost_button("toolbar-focus-roving").label("Roving focus").spawn(cx);
        let sequential_focus_button = look.ghost_button("toolbar-focus-sequential").label("Sequential focus").spawn(cx);

        let toolbar = look
            .toolbar("toolbar-demo")
            .roving_item_focus()
            .item(look.toolbar_menu(
                "assistant",
                "Assistant",
                LucideIcon::WandSparkles,
                [
                    MenuItem::new("rewrite").label("Rewrite"),
                    MenuItem::new("summarize").label("Summarize"),
                    MenuItem::new("continue").label("Continue writing"),
                ],
                cx,
            ))
            .separator("assistant-separator")
            .item(look.toolbar_selector(
                "style",
                "Paragraph",
                [
                    SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                    SelectorItem::new("heading").label("Heading").icon(LucideIcon::Type),
                    SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
                ],
                "paragraph",
                cx,
            ))
            .separator("style-separator")
            .item(look.toolbar_toggle("bold", LucideIcon::Bold, cx).on_click(|cx| {
                cx.notify();
            }))
            .item(look.toolbar_toggle("italic", LucideIcon::Italic, cx))
            .item(look.toolbar_toggle("underline", LucideIcon::Underline, cx))
            .item(look.toolbar_label_menu(
                "color",
                "A",
                [
                    MenuItem::new("default").label("Default color"),
                    MenuItem::new("muted").label("Muted"),
                    MenuItem::new("accent").label("Accent"),
                ],
                cx,
            ))
            .separator("format-separator")
            .item(look.toolbar_menu(
                "alignment",
                "Alignment",
                LucideIcon::TextAlignStart,
                [
                    MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                    MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                    MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                    MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
                ],
                cx,
            ))
            .separator("insert-separator")
            .item(look.toolbar_button("link", LucideIcon::Link, cx))
            .item(look.toolbar_button("image", LucideIcon::Image, cx))
            .item(look.toolbar_button("video", LucideIcon::Video, cx))
            .item(look.toolbar_menu(
                "more",
                "More",
                LucideIcon::Ellipsis,
                [
                    MenuItem::new("clear").label("Clear formatting"),
                    MenuItem::new("divider").label("Insert divider"),
                    MenuItem::new("comment").label("Add comment"),
                ],
                cx,
            ))
            .separator("code-separator")
            .item(look.toolbar_textfield("search").placeholder("Search...").spawn(cx).on_change(|_value, cx| {
                cx.notify();
            }))
            .item(look.toolbar_button("code", LucideIcon::Code, cx))
            .spawn(cx);

        Self {
            toolbar,
            roving_focus_button,
            sequential_focus_button,
            inspector,
            status: "Paragraph editing toolbar".to_string(),
            focus_mode: "roving",
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.toolbar, |app, _, event: &ToolbarEvent, cx| {
            app.panes.toolbar.status = match event {
                ToolbarEvent::Click { id } => format!("{id} clicked"),
                ToolbarEvent::Change { id, value } => match value {
                    ToolbarValue::Bool(on) => format!("{id}: {}", if *on { "on" } else { "off" }),
                    ToolbarValue::String(value) => format!("{id}: {value}"),
                },
            };
            cx.notify();
        }));

        subscriptions.push(cx.subscribe(&self.roving_focus_button, |app, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.toolbar.toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_focus_strategy(ControlGroupFocusStrategy::RovingItemFocus, cx);
                });
                app.panes.toolbar.focus_mode = "roving";
                app.panes.toolbar.status = "Focus strategy: roving".to_string();
                cx.notify();
            }
        }));

        subscriptions.push(cx.subscribe(&self.sequential_focus_button, |app, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                app.panes.toolbar.toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_focus_strategy(ControlGroupFocusStrategy::ActiveDescendant, cx);
                });
                app.panes.toolbar.focus_mode = "sequential";
                app.panes.toolbar.status = "Focus strategy: sequential".to_string();
                cx.notify();
            }
        }));
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
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(self.roving_focus_button.clone())
                        .child(self.sequential_focus_button.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child(format!("mode: {}", self.focus_mode)),
                        ),
                )
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
        self.toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.roving_focus_button.update(cx, |_, cx| cx.notify());
        self.sequential_focus_button.update(cx, |_, cx| cx.notify());
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}
