use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, Subscription, div, hsla, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorPlacement, SelectorItem};
use gpui_luma::theme::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::panel_preview::SelectorPanelPreview;
use super::preview::SelectorStatePreview;
use super::super::shared::{format_compact_hsla, gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SelectorPane {
    selector_smart: Entity<Selector>,
    selector_below: Entity<Selector>,
    selector_above: Entity<Selector>,
    selector_overlay: Entity<Selector>,
    selector_swatch: Entity<Selector>,
    state_preview: Entity<SelectorStatePreview>,
    panel_preview: Entity<SelectorPanelPreview>,
    selection: String,
    selected_swatch_id: String,
}

impl SelectorPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let selector_template = radix_theme.selector_template();
        let initial_swatch_id = selected_swatch_id();
        let initial_swatch_label = selected_swatch_label();

        Self {
            selector_smart: Selector::new("popup-selector-smart-example")
                .label("Select status")
                .items(selector_items())
                .placement(SelectorPlacement::Smart)
                .template(selector_template.clone())
                .spawn(cx),
            selector_below: Selector::new("popup-selector-below-example")
                .label("Below selector")
                .items(selector_items())
                .placement(SelectorPlacement::BelowStart)
                .template(selector_template.clone())
                .spawn(cx),
            selector_above: Selector::new("popup-selector-above-example")
                .label("Above selector")
                .items(selector_items())
                .placement(SelectorPlacement::AboveStart)
                .template(selector_template.clone())
                .spawn(cx),
            selector_overlay: Selector::new("popup-selector-overlay-example")
                .label("Overlay selector")
                .items(selector_items())
                .placement(SelectorPlacement::OverlayOnTrigger)
                .template(selector_template.clone())
                .spawn(cx),
            selector_swatch: Selector::new("popup-selector-swatch-example")
                .label("Choose color")
                .items(swatch_items())
                .selected_id(initial_swatch_id)
                .with_item_template(|item, _cx| {
                    let swatch = swatch_color(item.item.id().as_ref());
                    let selected_weight = if item.selected {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(20.0))
                                .rounded(px(2.0))
                                .bg(swatch)
                                .border_1()
                                .border_color(hsla(0.0, 0.0, 1.0, 0.18)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .child(div().font_weight(selected_weight).child(item.item.label_text().clone()))
                                .child(
                                    div()
                                        .font_family("Monaco")
                                        .text_size(px(10.0))
                                        .line_height(px(14.0))
                                        .opacity(0.72)
                                        .child(format_compact_hsla(swatch)),
                                ),
                        )
                })
                .template(selector_template.clone())
                .spawn(cx),
            state_preview: cx.new(|_| SelectorStatePreview::new(radix_theme.clone())),
            panel_preview: cx.new(|_| SelectorPanelPreview::new(radix_theme.clone())),
            selection: initial_swatch_label.to_string(),
            selected_swatch_id: initial_swatch_id.to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.selector_swatch, |app, _, event: &SelectorEvent, cx| {
            app.panes.selector.handle_swatch_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage(
            "Selector",
            "Selector",
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(self.selector_below.clone())
                                .child(self.selector_above.clone()),
                        )
                        .child(div().flex().items_center().gap_3().child(self.selector_overlay.clone()))
                        .child(self.selector_swatch.clone())
                        .child(div().text_color(chrome.body_text).child(format!("Selected: {}", self.selection)))
                        .child(self.state_preview.clone())
                        .child(self.panel_preview.clone()),
                )
                .child(self.selector_smart.clone())
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.selector_smart, cx);
        notify_entity(&self.selector_below, cx);
        notify_entity(&self.selector_above, cx);
        notify_entity(&self.selector_overlay, cx);
        notify_entity(&self.selector_swatch, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.panel_preview, cx);
    }

    fn handle_swatch_event(&mut self, event: &SelectorEvent, cx: &mut Context<GalleryApp>) {
        match event {
            SelectorEvent::Change { item_id, label } => {
                self.selected_swatch_id = item_id.to_string();
                self.selection = label.to_string();
                cx.notify();
            }
        }
    }
}

pub(super) fn selector_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

const SWATCHES: [(&str, &str, (f32, f32, f32, f32)); 16] = [
    ("sky-500", "Sky 500 (#0EA5E9)", (0.55, 0.85, 0.48, 1.0)),
    ("emerald-500", "Emerald 500 (#10B981)", (0.44, 0.85, 0.39, 1.0)),
    ("amber-500", "Amber 500 (#F59E0B)", (0.11, 0.92, 0.51, 1.0)),
    ("rose-500", "Rose 500 (#F43F5E)", (0.96, 0.89, 0.60, 1.0)),
    ("violet-500", "Violet 500 (#8B5CF6)", (0.74, 0.84, 0.66, 1.0)),
    ("slate-500", "Slate 500 (#64748B)", (0.60, 0.18, 0.47, 1.0)),
    ("indigo-500", "Indigo 500 (#6366F1)", (0.66, 0.84, 0.67, 1.0)),
    ("cyan-500", "Cyan 500 (#06B6D4)", (0.52, 0.94, 0.43, 1.0)),
    ("teal-500", "Teal 500 (#14B8A6)", (0.48, 0.81, 0.40, 1.0)),
    ("lime-500", "Lime 500 (#84CC16)", (0.23, 0.80, 0.44, 1.0)),
    ("yellow-500", "Yellow 500 (#EAB308)", (0.13, 0.93, 0.47, 1.0)),
    ("orange-500", "Orange 500 (#F97316)", (0.07, 0.95, 0.53, 1.0)),
    ("red-500", "Red 500 (#EF4444)", (0.00, 0.84, 0.60, 1.0)),
    ("pink-500", "Pink 500 (#EC4899)", (0.92, 0.82, 0.60, 1.0)),
    ("fuchsia-500", "Fuchsia 500 (#D946EF)", (0.83, 0.84, 0.61, 1.0)),
    ("blue-500", "Blue 500 (#3B82F6)", (0.60, 0.91, 0.60, 1.0)),
];

const INITIAL_SWATCH_ID: &str = "emerald-500";

fn selected_swatch() -> (&'static str, &'static str, (f32, f32, f32, f32)) {
    SWATCHES.iter().copied().find(|(id, _, _)| *id == INITIAL_SWATCH_ID).unwrap_or(SWATCHES[0])
}

fn selected_swatch_id() -> &'static str {
    selected_swatch().0
}

fn selected_swatch_label() -> &'static str {
    selected_swatch().1
}

fn swatch_items() -> [SelectorItem; 16] {
    let mut swatches = SWATCHES;
    swatches.sort_by(|a, b| {
        let (ah, asat, al, _) = a.2;
        let (bh, bsat, bl, _) = b.2;

        ah.partial_cmp(&bh)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| asat.partial_cmp(&bsat).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| al.partial_cmp(&bl).unwrap_or(std::cmp::Ordering::Equal))
    });

    std::array::from_fn(|index| {
        let (id, label, _) = swatches[index];
        SelectorItem::new(id).label(label)
    })
}

fn swatch_color(id: &str) -> Hsla {
    SWATCHES
        .iter()
        .find_map(|(swatch_id, _, (h, s, l, a))| (*swatch_id == id).then(|| hsla(*h, *s, *l, *a)))
        .unwrap_or_else(|| hsla(0.0, 0.0, 0.5, 1.0))
}
