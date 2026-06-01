use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::switch::Switch;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::RadixTheme;

use super::common::card;

pub struct CookiesPanel {
    radix_theme: Arc<RadixTheme>,
    pub necessary_switch: Switch,
    pub functional_switch: Switch,
    save_button: Entity<Button>,
}

impl CookiesPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>) -> Self {
        Self {
            necessary_switch: radix_theme
                .primary_switch("cookies-necessary")
                .with_data(true)
                .content(|_, _| div().into_any_element())
                .spawn(cx),
            functional_switch: radix_theme
                .primary_switch("cookies-functional")
                .with_data(false)
                .content(|_, _| div().into_any_element())
                .spawn(cx),
            save_button: radix_theme.secondary_button("cookies-save").label("Save preferences").spawn(cx),
            radix_theme,
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
            div()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(16.0))
                                .line_height(px(22.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Cookie Settings"),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child("Manage your cookie preferences here."),
                        ),
                )
                .child(cookie_row(
                    "Strictly Necessary",
                    "These cookies are essential in order to use the website and use its features.",
                    &self.necessary_switch,
                    chrome,
                ))
                .child(cookie_row(
                    "Functional Cookies",
                    "These cookies allow the website to provide personalized functionality.",
                    &self.functional_switch,
                    chrome,
                ))
                .child(div().w_full().child(self.save_button.clone())),
        )
    }
}

fn cookie_row(
    title: &'static str,
    body: &'static str,
    switch: &Switch,
    chrome: gpui_luma::theme::LumaChrome,
) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(title),
                )
                .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(chrome.muted_text).child(body)),
        )
        .child(switch.clone())
}
