use std::sync::Arc;

use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use gpui_luma::theme::ControlSize;
use gpui_luma::{flow, hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_badge_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity, InspectorToggleRegistry};

#[derive(Clone)]
pub(in crate::gallery) struct BadgePane {
    inspector: gpui::Entity<ColorInspectorShell>,
}

impl BadgePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("badge-inspector-tree", look.clone(), build_badge_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "badge-inspector",
                "badge-inspector-split",
                "badge-inspector-detail",
                build_badge_inspect_tree,
                cx,
            )
        });
        Self { inspector }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        gallery_pane_with_inspector_description(
            "badge",
            "Badge",
            Some(
                "Look-specific inline status element with variant colors, shared metric scaling, and optional start/end icons.",
            ),
            vstack! {
                gap=20.0 align=center;
                render_section(
                    "Reference",
                    hstack! {
                        gap=10.0 align=center;
                        look.badge("work").variant(BadgeVariant::Default),
                        look.badge("budget").variant(BadgeVariant::Outline),
                    }
                    .into_any_element(),
                    look,
                ),
                render_section(
                    "Variants",
                    flow! {
                        gap=10.0;
                        look.badge("Default").variant(BadgeVariant::Default),
                        look.badge("Secondary").variant(BadgeVariant::Secondary),
                        look.badge("Outline").variant(BadgeVariant::Outline),
                        look.badge("Ghost").variant(BadgeVariant::Ghost),
                    }
                    .justify_center()
                    .items_center()
                    .into_any_element(),
                    look,
                ),
                render_section(
                    "Sizes",
                    hstack! {
                        gap=10.0 align=center justify=center;
                        look.badge("Small").size(ControlSize::Sm).variant(BadgeVariant::Secondary),
                        look.badge("Medium").size(ControlSize::Md).variant(BadgeVariant::Secondary),
                        look.badge("Large").size(ControlSize::Lg).variant(BadgeVariant::Secondary),
                    }
                    .into_any_element(),
                    look,
                ),
                render_section(
                    "Icons",
                    flow! {
                        gap=10.0;
                        look.badge("Verified").variant(BadgeVariant::Default).start_icon(LucideIcon::BadgeCheck),
                        look.badge("Updated").variant(BadgeVariant::Secondary).end_icon(LucideIcon::ArrowUpRight),
                        look.badge("Draft").variant(BadgeVariant::Outline).start_icon(LucideIcon::Pencil),
                        look.badge("Muted").variant(BadgeVariant::Ghost).end_icon(LucideIcon::Dot),
                    }
                    .justify_center()
                    .items_center()
                    .into_any_element(),
                    look,
                ),
            }
            .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn subscribe(
        &self,
        _cx: &mut Context<GalleryApp>,
        _subscriptions: &mut Vec<gpui::Subscription>,
    ) {
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

fn render_section(title: &'static str, content: AnyElement, look: &ShadcnLook) -> AnyElement {
    let title_style = look.typography_scale(ShadcnTextSize::Sm);

    vstack! {
        gap=10.0 align=center;
        div().typography_style(title_style).text_color(look.chrome().muted_text).child(title),
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
    }
    .w_full()
    .max_w(px(720.0))
    .into_any_element()
}
