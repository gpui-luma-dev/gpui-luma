//! Badge look exposition — gallery-aligned variants, sizes, and icons (non-interactive).

use std::sync::Arc;

use gpui::{Context, Render, Window, div, prelude::*, px};
use gpui_luma::{flow, hstack};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::{ControlExpositionLayout, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "look.badge(label)",
        surface: "Look",
        notes: "ShadcnLookControlExt factory returning a Badge builder.",
    },
    PublicInterfaceSpec {
        symbol: "Badge::variant / size / start_icon / end_icon",
        surface: "Builder",
        notes: "Default, Secondary, Outline, Ghost variants; Sm/Md/Lg sizing; optional Lucide icons.",
    },
    PublicInterfaceSpec {
        symbol: "Badge (element)",
        surface: "Type",
        notes: "Inline status chip — render at compose time; no GPUI entity or events.",
    },
];

pub struct BadgeControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
}

impl BadgeControlExposition {
    pub fn new(_cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("badge").expect("badge catalog entry");
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

impl Render for BadgeControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let title_color = chrome.muted_text;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.0))
                .child(demo_section(
                    look,
                    "Reference",
                    title_color,
                    hstack! {
                        gap=10.0 align=center;
                        look.badge("work").variant(BadgeVariant::Default),
                        look.badge("budget").variant(BadgeVariant::Outline),
                    },
                ))
                .child(demo_section(
                    look,
                    "Variants",
                    title_color,
                    flow! {
                        gap=10.0;
                        look.badge("Default").variant(BadgeVariant::Default),
                        look.badge("Secondary").variant(BadgeVariant::Secondary),
                        look.badge("Outline").variant(BadgeVariant::Outline),
                        look.badge("Ghost").variant(BadgeVariant::Ghost),
                    }
                    .justify_center()
                    .items_center(),
                ))
                .child(demo_section(
                    look,
                    "Sizes",
                    title_color,
                    hstack! {
                        gap=10.0 align=center justify=center;
                        look.badge("Small").size(ControlSize::Sm).variant(BadgeVariant::Secondary),
                        look.badge("Medium").size(ControlSize::Md).variant(BadgeVariant::Secondary),
                        look.badge("Large").size(ControlSize::Lg).variant(BadgeVariant::Secondary),
                    },
                ))
                .child(demo_section(
                    look,
                    "Icons",
                    title_color,
                    flow! {
                        gap=10.0;
                        look.badge("Verified").variant(BadgeVariant::Default).start_icon(LucideIcon::BadgeCheck),
                        look.badge("Updated").variant(BadgeVariant::Secondary).end_icon(LucideIcon::ArrowUpRight),
                        look.badge("Draft").variant(BadgeVariant::Outline).start_icon(LucideIcon::Pencil),
                        look.badge("Muted").variant(BadgeVariant::Ghost).end_icon(LucideIcon::Dot),
                    }
                    .justify_center()
                    .items_center(),
                ));

            render_control_exposition_card(
                look,
                self.entry,
                preview.into_any_element(),
                Some(render_exposition_doc_sections(look, &[], PUBLIC_INTERFACE_SPECS)),
                ControlExpositionLayout::BORDERLESS,
            )
        })
    }
}

fn demo_section(
    look: &ShadcnLook,
    title: &'static str,
    title_color: gpui::Hsla,
    content: impl IntoElement,
) -> impl IntoElement {
    let section_title = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .w_full()
        .max_w(px(720.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(div().typography_style(section_title).text_color(title_color).child(title))
        .child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .rounded(px(16.0))
                .bg(look.color(ShadcnToken::Card))
                .border_1()
                .border_color(look.color(ShadcnToken::Border))
                .p(px(18.0))
                .child(content),
        )
}
