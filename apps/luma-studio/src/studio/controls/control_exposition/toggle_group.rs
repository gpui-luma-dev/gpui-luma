//! Toggle group control exposition — gallery-aligned icon placement groups.

use std::sync::Arc;

use gpui::{Context, Entity, Hsla, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::button_group::{IconGroup, IconGroupEvent, IconGroupItem, IconGroupItemLike};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::control_group::{
    ControlGroupBuilder, ControlGroupItemTemplate, button_item_template, make_control_group_item_template,
};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const STYLE_VARIANTS: [(&str, ShadcnButtonStyle); 4] = [
    ("Primary", ShadcnButtonStyle::Primary),
    ("Secondary", ShadcnButtonStyle::Secondary),
    ("Outline", ShadcnButtonStyle::Outline),
    ("Ghost", ShadcnButtonStyle::Ghost),
];

const PLACEMENT: [(&str, &str); 4] = [("top", "Top"), ("bottom", "Bottom"), ("left", "Left"), ("right", "Right")];

#[derive(Clone)]
enum DemoSelection {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone)]
struct DemoGroup {
    title: &'static str,
    group: IconGroup<IconGroupItem>,
    selection: DemoSelection,
}

pub struct ToggleGroupControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demos: Vec<DemoGroup>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ToggleGroupControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("toggle-group").expect("toggle-group catalog entry");

        let mut demos: Vec<DemoGroup> = STYLE_VARIANTS
            .iter()
            .map(|(title, style)| DemoGroup {
                title,
                group: icon_group(
                    look.clone(),
                    format!("controls-doc-{}-placement-toggle-group", title.to_ascii_lowercase()),
                    *style,
                    &["bottom"],
                    false,
                    cx,
                ),
                selection: DemoSelection::Single("Bottom".into()),
            })
            .collect();

        demos.push(DemoGroup {
            title: "Multiple · Secondary",
            group: icon_group(
                look.clone(),
                "controls-doc-edge-toggle-group",
                ShadcnButtonStyle::Secondary,
                &["top", "left"],
                true,
                cx,
            ),
            selection: DemoSelection::Multiple(vec!["Top".into(), "Left".into()]),
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toggle-group-event-log",
                "Toggle placement icons; IconGroupEvent variants from the Primary group appear below.",
            )
        });

        let mut subscriptions = Vec::new();
        for (index, demo) in demos.iter().enumerate() {
            let group = demo.group.clone();
            let event_stream = event_stream.clone();
            subscriptions.push(cx.subscribe(&group, move |this, _, event: &IconGroupEvent, cx| {
                this.apply_event(index, event, cx);
                if index == 0
                    && let Some(line) = format_icon_group_event(event)
                {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }));
        }

        Self { look, entry, demos, event_stream, _subscriptions: subscriptions }
    }

    fn apply_event(&mut self, index: usize, event: &IconGroupEvent, cx: &mut Context<Self>) {
        let Some(demo) = self.demos.get_mut(index) else {
            return;
        };
        let IconGroupEvent::Change { changed_id, selected, selected_ids, .. } = event else {
            return;
        };
        demo.selection = match &demo.selection {
            DemoSelection::Single(_) => DemoSelection::Single(if *selected {
                label_for(changed_id.as_ref())
            } else {
                "None".into()
            }),
            DemoSelection::Multiple(_) => DemoSelection::Multiple(if selected_ids.is_empty() {
                vec!["None".into()]
            } else {
                selected_ids.iter().map(|id| label_for(id.as_ref())).collect()
            }),
        };
        cx.notify();
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for demo in &self.demos {
            demo.group.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ToggleGroupControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.0))
                .children(
                    self.demos
                        .iter()
                        .map(|demo| demo_section(demo, chrome.body_text, chrome.muted_text).into_any_element()),
                )
                .child(self.event_stream.clone());

            render_control_exposition_card(
                &self.look,
                self.entry,
                preview.into_any_element(),
                None,
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn icon_group(
    look: Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    style: ShadcnButtonStyle,
    selected: &[&'static str],
    multiple: bool,
    cx: &mut Context<ToggleGroupControlExposition>,
) -> IconGroup<IconGroupItem> {
    let mut builder = look
        .button_group(id)
        .items(items(&PLACEMENT))
        .item_template(theme_aware_toggle_item_template(look.clone(), style))
        .with_item_layout(move |mut items, _, _, _| {
            let group_look = look.control_group_theme().resolve_list(true);
            gpui_luma::hstack![
                gap = 6 align = center;
                items.take("top"),
                items.take("bottom"),
                items.take("left"),
                items.take("right"),
            ]
            .relative()
            .overflow_hidden()
            .rounded_full()
            .bg(group_look.background)
            .border_1()
            .border_color(group_look.border)
            .px(px(6.0))
            .py(px(4.0))
        });
    builder = apply_selection(builder, selected, multiple);
    builder.spawn(cx)
}

fn theme_aware_toggle_item_template(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> ControlGroupItemTemplate<IconGroupItem> {
    make_control_group_item_template(move |item, window, cx| {
        let button_template = look.toggle_item_template(style);
        button_item_template(
            button_template,
            |selected| ButtonFamilyRole::Toggle { selected },
            true,
            |item: &IconGroupItem| lucide_glyph(placement_icon(item.id().as_ref())),
        )(item, window, cx)
    })
}

fn apply_selection(
    builder: ControlGroupBuilder<IconGroupItem>,
    selected: &[&'static str],
    multiple: bool,
) -> ControlGroupBuilder<IconGroupItem> {
    if multiple {
        builder.multiple().selected_ids(selected.iter().copied())
    } else {
        builder.selected(selected[0])
    }
}

fn demo_section(demo: &DemoGroup, body: Hsla, muted: Hsla) -> impl IntoElement {
    let caption = match &demo.selection {
        DemoSelection::Single(value) => format!("Single: {value}"),
        DemoSelection::Multiple(values) => format!("Selected: {}", values.join(", ")),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(muted)
                .child(demo.title),
        )
        .child(demo.group.clone())
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(body).child(caption))
}

fn items(entries: &[(&'static str, &'static str)]) -> Vec<IconGroupItem> {
    entries.iter().map(|(id, label)| IconGroupItem::new(*id).label(*label)).collect()
}

fn label_for(id: &str) -> String {
    PLACEMENT
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, label)| (*label).to_string())
        .unwrap_or_else(|| id.to_string())
}

fn placement_icon(id: &str) -> LucideIcon {
    match id {
        "top" => LucideIcon::PanelTop,
        "bottom" => LucideIcon::PanelBottom,
        "left" => LucideIcon::PanelLeft,
        "right" => LucideIcon::PanelRight,
        _ => LucideIcon::Settings,
    }
}

fn format_icon_group_event(event: &IconGroupEvent) -> Option<String> {
    match event {
        IconGroupEvent::Change { changed_id, selected, selected_ids, .. } => Some(format!(
            "IconGroupEvent::Change {{ changed_id: \"{changed_id}\", selected: {selected}, selected_ids: {selected_ids:?} }}"
        )),
        IconGroupEvent::FocusChanged { focused } => {
            Some(format!("IconGroupEvent::FocusChanged {{ focused: {focused} }}"))
        }
        _ => None,
    }
}
