use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::button_group::{IconGroup, IconGroupEvent, IconGroupItem, IconGroupItemLike};
use gpui_luma::controls::control_group::toggle_button_item_template;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_toggle_group_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    single_group: IconGroup<IconGroupItem>,
    multiple_group: IconGroup<IconGroupItem>,
    placement: String,
    visible_edges: Vec<String>,
    inspector: Entity<ColorInspectorShell>,
}

impl ToggleGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "toggle-group-inspector-tree",
            look.clone(),
            build_toggle_group_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "toggle-group-inspector",
                "toggle-group-inspector-split",
                "toggle-group-inspector-detail",
                build_toggle_group_inspect_tree,
                cx,
            )
        });

        let toggle_template = look.toggle_template(ShadcnButtonStyle::Ghost);

        let single_group = look
            .button_group("placement-toggle-group")
            .horizontal()
            .with_template_modifier(|element, _| element.rounded_full().gap(px(6.0)).px(px(6.0)).py(px(4.0)))
            .selected("bottom")
            .items(placement_items())
            .item_template(toggle_button_item_template(toggle_template.clone(), true, placement_icon_content))
            .spawn(cx);

        let multiple_group = look
            .button_group("edge-toggle-group")
            .horizontal()
            .with_template_modifier(|element, _| element.rounded_full().gap(px(6.0)).px(px(6.0)).py(px(4.0)))
            .multiple()
            .selected_ids(["top", "left"])
            .items(edge_items())
            .item_template(toggle_button_item_template(toggle_template, true, placement_icon_content))
            .spawn(cx);

        Self {
            single_group,
            multiple_group,
            placement: "Bottom".to_string(),
            visible_edges: vec!["Top".to_string(), "Left".to_string()],
            inspector,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single_group, |app, _, event: &IconGroupEvent, cx| {
            app.panes.toggle_group.handle_single_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.multiple_group, |app, _, event: &IconGroupEvent, cx| {
            app.panes.toggle_group.handle_multiple_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector(
            "Toggle Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(self.single_group.clone())
                        .child(div().text_color(chrome.body_text).child(format!("Single: {}", self.placement))),
                )
                .child(div().flex().flex_col().items_center().gap_2().child(self.multiple_group.clone()).child(
                    div().text_color(chrome.body_text).child(format!("Multiple: {}", self.visible_edges.join(", "))),
                ))
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single_group, cx);
        notify_entity(&self.multiple_group, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_single_event(&mut self, event: &IconGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            IconGroupEvent::Change { changed_id, selected, .. } => {
                self.placement = if *selected {
                    label_for_id(changed_id.as_ref())
                } else {
                    "None".to_string()
                };
                cx.notify();
            }
        }
    }

    fn handle_multiple_event(&mut self, event: &IconGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            IconGroupEvent::Change { selected_ids, .. } => {
                self.visible_edges = if selected_ids.is_empty() {
                    vec!["None".to_string()]
                } else {
                    selected_ids.iter().map(|id| label_for_id(id.as_ref())).collect()
                };
                cx.notify();
            }
        }
    }
}

fn placement_icon_content(item: &IconGroupItem) -> AnyElement {
    lucide_glyph(placement_icon_for_id(item.id().as_ref()))
}

fn placement_icon_for_id(id: &str) -> LucideIcon {
    match id {
        "top" => LucideIcon::PanelTop,
        "bottom" => LucideIcon::PanelBottom,
        "left" => LucideIcon::PanelLeft,
        "right" => LucideIcon::PanelRight,
        _ => LucideIcon::Settings,
    }
}

fn placement_items() -> [IconGroupItem; 4] {
    [
        IconGroupItem::new("top").label("Top"),
        IconGroupItem::new("bottom").label("Bottom"),
        IconGroupItem::new("left").label("Left"),
        IconGroupItem::new("right").label("Right"),
    ]
}

fn edge_items() -> [IconGroupItem; 4] {
    placement_items()
}

fn label_for_id(id: &str) -> String {
    match id {
        "top" => "Top",
        "bottom" => "Bottom",
        "left" => "Left",
        "right" => "Right",
        _ => id,
    }
    .to_string()
}
