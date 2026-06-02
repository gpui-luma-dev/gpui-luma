use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::switch::Switch;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::RadixTheme;
use gpui_luma::{hstack, vstack};

use super::common::{card, card_header};

pub struct CookiesPanel {
    radix_theme: Arc<RadixTheme>,
    pub necessary_switch: Switch,
    pub functional_switch: Switch,
    save_button: Entity<Button>,
    _subscriptions: Vec<Subscription>,
}

impl CookiesPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>) -> Self {
        let necessary_switch = radix_theme
            .primary_switch("cookies-necessary")
            .with_data(true)
            .content(|_, _| div().into_any_element())
            .spawn(cx);
        let functional_switch = radix_theme
            .primary_switch("cookies-functional")
            .with_data(false)
            .content(|_, _| div().into_any_element())
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&necessary_switch, |_, switch, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                switch.update(cx, |button, cx| button.set_data(!*button.data(), cx));
            }
        }));
        subscriptions.push(cx.subscribe(&functional_switch, |_, switch, event, cx| {
            if matches!(event, ButtonEvent::Click) {
                switch.update(cx, |button, cx| button.set_data(!*button.data(), cx));
            }
        }));

        Self {
            necessary_switch,
            functional_switch,
            save_button: radix_theme.secondary_button("cookies-save").label("Save preferences").spawn(cx),
            radix_theme,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for CookiesPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();

        card(
            380.0,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=14;
                card_header(
                    "Cookie Settings",
                    "Manage your cookie preferences here.",
                    chrome.title_text,
                    chrome.muted_text,
                ),
                cookie_row(
                    "Strictly Necessary",
                    "These cookies are essential in order to use the website and use its features.",
                    &self.necessary_switch,
                    chrome,
                ),
                cookie_row(
                    "Functional Cookies",
                    "These cookies allow the website to provide personalized functionality.",
                    &self.functional_switch,
                    chrome,
                ),
                div().w_full().child(self.save_button.clone()),
            },
        )
    }
}

fn cookie_row(
    title: &'static str,
    body: &'static str,
    switch: &Switch,
    chrome: gpui_luma::theme::LumaChrome,
) -> impl IntoElement {
    hstack! {
        justify=between align=start gap=12;
        vstack! {
            gap=4;
            div()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .child(title),
            div()
                .w_full()
                .text_size(px(11.0))
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
