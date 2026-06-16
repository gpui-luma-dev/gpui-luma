use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextRole, ShadcnTextSize};
use gpui_luma::{declare_form, hstack, vstack};

use super::common::{avatar, AvatarSize, card};

const TEAM_CARD_WIDTH: f32 = 380.0;

declare_form! {
    pub struct TeamPanel {
        controls: {
            sofia_selector: Entity<Selector> = look
                .selector("team-sofia")
                .label("Owner")
                .items(role_items()),
            jackson_selector: Entity<Selector> = look
                .selector("team-jackson")
                .label("Developer")
                .items(role_items()),
            isabella_selector: Entity<Selector> = look
                .selector("team-isabella")
                .label("Billing")
                .items(role_items()),
        },
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {}
    }
}

impl Render for TeamPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let title_style = self.look.typography_role(ShadcnTextRole::H4);
            let body_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let caption_style = self.look.typography_scale(ShadcnTextSize::Xs);
            let sofia_selector = self.sofia_selector.clone();
            let jackson_selector = self.jackson_selector.clone();
            let isabella_selector = self.isabella_selector.clone();

            card(
                "theme-studio-team-card",
                &self.look,
                TEAM_CARD_WIDTH,
                move |_, _| {
                    vstack! {
                        gap=12;
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div().typography_style(title_style).text_color(chrome.title_text).child("Team Members"),
                            )
                            .child(
                                div().typography_style(body_style).text_color(chrome.muted_text).child("Invite your team members to collaborate."),
                            ),
                        member_row("SD", "Sofia Davis", "m@example.com", &sofia_selector, chrome, body_style, caption_style),
                        member_row("JL", "Jackson Lee", "m@example.com", &jackson_selector, chrome, body_style, caption_style),
                        member_row("IN", "Isabella Nguyen", "m@example.com", &isabella_selector, chrome, body_style, caption_style),
                    }
                    .w_full()
                    .overflow_hidden()
                    .into_any_element()
                },
                window,
                _cx,
            )
        })
    }
}

fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
    selector: &Entity<Selector>,
    chrome: gpui_luma::theme::LumaChrome,
    body_style: gpui_luma::theme::LumaTextStyle,
    caption_style: gpui_luma::theme::LumaTextStyle,
) -> impl IntoElement {
    hstack! {
        gap=10 align=center;
        avatar(initials, AvatarSize::Sm),
        vstack! {
            gap=2;
            div()
                .typography_style(body_style)
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .overflow_hidden()
                .child(name),
            div()
                .typography_style(caption_style)
                .text_color(chrome.muted_text)
                .overflow_hidden()
                .child(email),
        }
        .flex_1()
        .min_w_0()
        .overflow_hidden(),
        div().flex_none().child(selector.clone()),
    }
    .py(px(6.0))
    .overflow_hidden()
}

fn role_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("owner").label("Owner"),
        SelectorItem::new("developer").label("Developer"),
        SelectorItem::new("billing").label("Billing"),
    ]
}
