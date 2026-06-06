use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma::{declare_form, hstack, vstack};

use super::common::{avatar_circle, card, card_header};

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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let avatar_bg = gpui::hsla(0.55, 0.12, 0.35, 1.0);

        card(
            TEAM_CARD_WIDTH,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=12;
                card_header(
                    "Team Members",
                    "Invite your team members to collaborate.",
                    chrome.title_text,
                    chrome.muted_text,
                ),
                member_row("SD", "Sofia Davis", "m@example.com", &self.sofia_selector, chrome, avatar_bg),
                member_row("JL", "Jackson Lee", "m@example.com", &self.jackson_selector, chrome, avatar_bg),
                member_row("IN", "Isabella Nguyen", "m@example.com", &self.isabella_selector, chrome, avatar_bg),
            }
            .w_full()
            .overflow_hidden(),
        )
    }
}

fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
    selector: &Entity<Selector>,
    chrome: gpui_luma::theme::LumaChrome,
    avatar_bg: gpui::Hsla,
) -> impl IntoElement {
    hstack! {
        gap=10 align=center;
        avatar_circle(initials, 32.0, avatar_bg, chrome.title_text),
        vstack! {
            gap=2;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .overflow_hidden()
                .child(name),
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
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
