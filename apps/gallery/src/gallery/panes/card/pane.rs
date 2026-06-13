use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, div, prelude::*, px};
use gpui_luma::controls::card::Card;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma::{hstack, vstack};

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_card_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct CardPane {
    report_card: Card,
    profile_card: Card,
    subject_field: TextField,
    description_field: TextField,
    cancel_button: Entity<Button>,
    submit_button: Entity<Button>,
    inspector: Entity<ColorInspectorShell>,
}

impl CardPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("card-inspector-tree", look.clone(), build_card_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "card-inspector",
                "card-inspector-split",
                "card-inspector-detail",
                build_card_inspect_tree,
                cx,
            )
        });
        let subject_field = look.textfield("card-demo-subject").placeholder("Subject").full_width(true).spawn(cx);
        let description_field =
            look.textfield("card-demo-description").placeholder("Short description").full_width(true).spawn(cx);
        let cancel_button = look.secondary_button("card-demo-cancel").label("Cancel").spawn(cx);
        let submit_button = look.primary_button("card-demo-submit").label("Submit").spawn(cx);

        let report_card = {
            let subject_field = subject_field.clone();
            let description_field = description_field.clone();
            let cancel_button = cancel_button.clone();
            let submit_button = submit_button.clone();
            look.card("card-demo-report")
                .title("Report an issue")
                .description("Title, description, body content, and footer all come from the shared Card control.")
                .child_render(move |_, _| {
                    vstack! {
                        gap=10;
                        subject_field.clone(),
                        description_field.clone(),
                    }
                    .into_any_element()
                })
                .footer(move |_, _| {
                    hstack! {
                        gap=8 justify=end;
                        cancel_button.clone(),
                        submit_button.clone(),
                    }
                    .into_any_element()
                })
                .spawn(cx)
        };

        let profile_card = look
            .card("card-demo-profile")
            .elevated(false)
            .header(move |_, _| {
                hstack! {
                    justify=between align=center;
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(14.0))
                                .line_height(px(18.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("Custom Header"),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(look.chrome().muted_text)
                                .child("This variant opts out of the shadow."),
                        ),
                    div()
                        .rounded_full()
                        .bg(look.color(ShadcnToken::Primary))
                        .text_color(look.color(ShadcnToken::PrimaryForeground))
                        .px(px(10.0))
                        .py(px(4.0))
                        .text_size(px(11.0))
                        .line_height(px(14.0))
                        .child("Preview"),
                }
                .into_any_element()
            })
            .child_render(move |_, _| {
                div()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .child(
                        "Use custom headers when the slot needs actions, badges, or layout that should not be baked into the default title/description row.",
                    )
                    .into_any_element()
            })
            .spawn(cx);

        Self { report_card, profile_card, subject_field, description_field, cancel_button, submit_button, inspector }
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_inspector_description(
            "Card",
            Some(
                "Shared card chrome with shadcn token resolution, plus title/description, custom header, body, and footer slots.",
            ),
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(18.0))
                .child(div().w(px(420.0)).max_w_full().child(self.report_card.clone()))
                .child(div().w(px(420.0)).max_w_full().child(self.profile_card.clone()))
                .into_any_element(),
            self.inspector.clone(),
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
        notify_entity(&self.report_card, cx);
        notify_entity(&self.profile_card, cx);
        notify_entity(&self.subject_field, cx);
        notify_entity(&self.description_field, cx);
        notify_entity(&self.cancel_button, cx);
        notify_entity(&self.submit_button, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}
