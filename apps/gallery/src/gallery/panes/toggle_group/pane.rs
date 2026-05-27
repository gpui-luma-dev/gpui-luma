use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::button_group::{self, IconGroup, IconGroupEvent, IconGroupItem, IconGroupItemLike};
use gpui_luma::controls::control_group::toggle_button_item_template;
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::theme::{RadixButtonStyle, RadixTheme};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage_descriptions, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    single_group: IconGroup<IconGroupItem>,
    multiple_group: IconGroup<IconGroupItem>,
    placement: String,
    visible_edges: Vec<String>,
}

impl ToggleGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let toggle_template = radix_theme.toggle_template(RadixButtonStyle::Ghost);
        let group_template = radix_theme.control_group_template();

        let single_group = button_group::new("placement-toggle-group")
            .horizontal()
            .template(group_template.clone())
            .with_template_modifier(|element, _| element.rounded_full().gap(px(6.0)).px(px(6.0)).py(px(4.0)))
            .selected("bottom")
            .items(placement_items())
            .item_template(toggle_button_item_template(toggle_template.clone(), true, placement_icon_content))
            .spawn(cx);

        let multiple_group = button_group::new("edge-toggle-group")
            .horizontal()
            .template(group_template)
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

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage_descriptions(
            "Toggle Group",
            None,
            &["Control Group", "Toggle"],
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
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single_group, cx);
        notify_entity(&self.multiple_group, cx);
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
