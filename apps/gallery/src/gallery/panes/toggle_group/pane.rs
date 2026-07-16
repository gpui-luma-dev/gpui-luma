use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, Subscription, div, prelude::*, px};
use gpui_luma::controls::button_group::{IconGroup, IconGroupEvent, IconGroupItem, IconGroupItemLike};
use gpui_luma::controls::control_group::{ControlGroupBuilder, toggle_button_item_template};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_toggle_group_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

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

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    demos: Vec<DemoGroup>,
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

        let mut demos: Vec<DemoGroup> = STYLE_VARIANTS
            .iter()
            .map(|(title, style)| DemoGroup {
                title,
                group: icon_group(
                    &look,
                    format!("{}-placement-toggle-group", title.to_ascii_lowercase()),
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
            group: icon_group(&look, "edge-toggle-group", ShadcnButtonStyle::Secondary, &["top", "left"], true, cx),
            selection: DemoSelection::Multiple(vec!["Top".into(), "Left".into()]),
        });

        Self { demos, inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        for index in 0..self.demos.len() {
            let group = self.demos[index].group.clone();
            subscriptions.push(cx.subscribe(&group, move |app, _, event: &IconGroupEvent, cx| {
                app.panes.toggle_group.apply_event(index, event, cx);
            }));
        }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();
        gallery_pane_with_inspector(
            "toggle-group",
            "Toggle Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .children(self.demos.iter().map(|demo| demo_section(demo, chrome.body_text, chrome.muted_text)))
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        for demo in &self.demos {
            notify_entity(&demo.group, cx);
        }
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn apply_event(&mut self, index: usize, event: &IconGroupEvent, cx: &mut Context<GalleryApp>) {
        let Some(demo) = self.demos.get_mut(index) else {
            return;
        };
        let IconGroupEvent::Change { changed_id, selected, selected_ids, .. } = event;
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
}

fn icon_group(
    look: &Arc<ShadcnLook>,
    id: impl Into<gpui::SharedString>,
    style: ShadcnButtonStyle,
    selected: &[&'static str],
    multiple: bool,
    cx: &mut Context<GalleryApp>,
) -> IconGroup<IconGroupItem> {
    let group_look = look.control_group_theme().resolve_list(true);
    let mut builder = look
        .button_group(id)
        .items(items(&PLACEMENT))
        .item_template(toggle_button_item_template(look.toggle_template(style), true, |item: &IconGroupItem| {
            lucide_glyph(placement_icon(item.id().as_ref()))
        }))
        .with_item_layout(move |mut items, _, _, _| {
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

fn demo_section(demo: &DemoGroup, body: Hsla, muted: Hsla) -> AnyElement {
    let caption = match &demo.selection {
        DemoSelection::Single(value) => format!("Single: {value}"),
        DemoSelection::Multiple(values) => format!("Selected: {}", values.join(", ")),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
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
        .into_any_element()
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
