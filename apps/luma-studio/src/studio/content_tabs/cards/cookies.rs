use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use luma::controls::presenter::HasPresenter;
use luma::controls::command::button::Button;
use luma::controls::switch::{Switch, SwitchEvent};
use luma_look_shadcn::prelude::*;
use luma_look_shadcn::ShadcnLook;
use luma::{declare_form, hstack, vstack};

use super::common::titled_card;

declare_form! {
    pub struct CookiesPanel {
        controls: {
            necessary_switch: Switch = look
                .primary_switch("cookies-necessary")
                .with_data(true)
                .content(|_, _| div().into_any_element())
                => SwitchEvent |this, event, cx| {
                    if let SwitchEvent::Change { on } = event {
                        this.necessary_enabled = *on;
                        cx.notify();
                    }
                },
            functional_switch: Switch = look
                .primary_switch("cookies-functional")
                .with_data(false)
                .content(|_, _| div().into_any_element())
                => SwitchEvent |this, event, cx| {
                    if let SwitchEvent::Change { on } = event {
                        this.functional_enabled = *on;
                        cx.notify();
                    }
                },
            save_button: Entity<Button> = look.outline_button("cookies-save").label("Save preferences"),
        },
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {
            necessary_enabled: bool = true,
            functional_enabled: bool = false,
        }
    }
}

impl Render for CookiesPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let necessary_switch = self.necessary_switch.clone();
        let functional_switch = self.functional_switch.clone();
        let save_button = self.save_button.clone();

        titled_card(
            "luma-studio-cookies-card",
            &self.look,
            380.0,
            "Cookie Settings",
            "Manage your cookie preferences here.",
            move |_, _| {
                vstack! {
                    gap=14;
                    cookie_row(
                        "Strictly Necessary",
                        "These cookies are essential in order to use the website and use its features.",
                        &necessary_switch,
                        chrome,
                    ),
                    cookie_row(
                        "Functional Cookies",
                        "These cookies allow the website to provide personalized functionality.",
                        &functional_switch,
                        chrome,
                    ),
                    div().w_full().child(save_button.clone()),
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn cookie_row(
    title: &'static str,
    body: &'static str,
    switch: &Switch,
    chrome: luma::theme::LumaChrome,
) -> impl IntoElement {
    hstack! {
        justify=between align=start gap=12;
        vstack! {
            gap=4;
            div().text_sm().font_weight(gpui::FontWeight::MEDIUM).text_color(chrome.body_text).child(title),
            div()
                .w_full()
                .text_xs()
                .line_height(px(15.0))
                .text_color(chrome.muted_text)
                .child(body),
        }
        .flex_1()
        .min_w_0(),
        div().flex_none().child(switch.clone()),
    }
    .w_full()
}
