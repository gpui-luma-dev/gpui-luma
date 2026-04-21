use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*};
use gpui_luma::controls::toggle_group::{ToggleGroup, ToggleGroupEvent, ToggleGroupItem};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    single_group: Entity<ToggleGroup>,
    multiple_group: Entity<ToggleGroup>,
    disabled_group: Entity<ToggleGroup>,
    placement: String,
    visible_edges: Vec<String>,
}

impl ToggleGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            single_group: ToggleGroup::new("placement-toggle-group")
                .items(placement_items())
                .selected("bottom")
                .template(theme.toggle_group_template())
                .spawn(cx),
            multiple_group: ToggleGroup::new("edge-toggle-group")
                .multiple()
                .items(edge_items())
                .selected_ids(["top", "left"])
                .template(theme.toggle_group_template())
                .spawn(cx),
            disabled_group: ToggleGroup::new("disabled-placement-toggle-group")
                .items(placement_items())
                .selected("right")
                .enabled(false)
                .template(theme.toggle_group_template())
                .spawn(cx),
            placement: "Bottom".to_string(),
            visible_edges: vec!["Top".to_string(), "Left".to_string()],
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single_group, |app, _, event: &ToggleGroupEvent, cx| {
            app.panes.toggle_group.handle_single_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.multiple_group, |app, _, event: &ToggleGroupEvent, cx| {
            app.panes.toggle_group.handle_multiple_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane(
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
                .child(self.disabled_group.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single_group, cx);
        notify_entity(&self.multiple_group, cx);
        notify_entity(&self.disabled_group, cx);
    }

    fn handle_single_event(&mut self, event: &ToggleGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleGroupEvent::Change { label, selected, .. } => {
                self.placement = if *selected {
                    label.to_string()
                } else {
                    "None".to_string()
                };
                cx.notify();
            }
        }
    }

    fn handle_multiple_event(&mut self, event: &ToggleGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleGroupEvent::Change { selected_ids, .. } => {
                self.visible_edges = selected_ids.iter().map(ToString::to_string).map(label_for_edge_id).collect();
            }
        }

        if self.visible_edges.is_empty() {
            self.visible_edges.push("None".to_string());
        }

        cx.notify();
    }
}

fn placement_items() -> [ToggleGroupItem; 4] {
    [
        ToggleGroupItem::new("top").label("Top"),
        ToggleGroupItem::new("bottom").label("Bottom"),
        ToggleGroupItem::new("left").label("Left"),
        ToggleGroupItem::new("right").label("Right"),
    ]
}

fn edge_items() -> [ToggleGroupItem; 4] {
    [
        ToggleGroupItem::new("top").label("Top"),
        ToggleGroupItem::new("bottom").label("Bottom"),
        ToggleGroupItem::new("left").label("Left"),
        ToggleGroupItem::new("right").label("Right"),
    ]
}

fn label_for_edge_id(id: String) -> String {
    match id.as_str() {
        "top" => "Top",
        "bottom" => "Bottom",
        "left" => "Left",
        "right" => "Right",
        _ => id.as_str(),
    }
    .to_string()
}
