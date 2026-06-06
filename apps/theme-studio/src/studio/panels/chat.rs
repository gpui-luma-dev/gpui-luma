use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ControlIcon};
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{ControlSize};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma::{declare_form, hstack, vstack};
use lucide_icons::Icon as LucideIcon;

use super::common::{avatar_circle, card, message_bubble};

declare_form! {
    pub struct ChatPanel {
        controls: {
            message_field: TextField = look
                .textfield("chat-message")
                .placeholder("Type your message…")
                .full_width(true),
            send_button: Entity<Button> = look
                .primary_icon_button("chat-send", ControlIcon::Lucide(LucideIcon::ArrowUp))
                .size(size)
                .round(true),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ControlSize,
        },
        fields: {}
    }
}

impl Render for ChatPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let incoming_bg = chrome.panel_background;
        let primary = self.look.mode_tokens().palette.action(ShadcnButtonStyle::Primary);
        let outgoing_bg = primary.background;
        let outgoing_fg = primary.foreground;
        let avatar_bg = gpui::hsla(0.55, 0.12, 0.35, 1.0);

        card(
            360.0,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=12;
                hstack! {
                    justify=between align=center;
                    hstack! {
                        gap=10 align=center;
                        avatar_circle("SD", 36.0, avatar_bg, chrome.title_text),
                        vstack! {
                            gap=2;
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Sofia Davis"),
                            div()
                                .text_size(px(11.0))
                                .text_color(chrome.muted_text)
                                .child("m@example.com"),
                        },
                    },
                    div()
                        .size(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(chrome.border)
                        .font_family("lucide")
                        .text_size(px(14.0))
                        .text_color(chrome.body_text)
                        .child(char::from(LucideIcon::Plus).to_string()),
                },
                vstack! {
                    gap=8;
                    message_bubble("Hi, how can I help you today?", false, incoming_bg, chrome.body_text),
                    message_bubble("Hey, I'm having trouble with my account.", true, outgoing_bg, outgoing_fg),
                    message_bubble("What seems to be the problem?", false, incoming_bg, chrome.body_text),
                    message_bubble("I can't log in.", true, outgoing_bg, outgoing_fg),
                }
                .min_h(px(180.0)),
                hstack! {
                    gap=8 align=center;
                    div().flex_1().child(self.message_field.clone()),
                    self.send_button.clone(),
                },
            },
        )
    }
}
