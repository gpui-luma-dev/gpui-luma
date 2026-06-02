use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{ControlSize, RadixTheme};
use gpui_luma::{declare_form, form_field, hstack, vstack};

use super::common::{card, card_header, or_divider};

declare_form! {
    pub struct AccountPanel {
        controls: {
            github_button: Entity<Button> = radix_theme.outline_button("account-github").label("GitHub").size(size),
            google_button: Entity<Button> = radix_theme.outline_button("account-google").label("Google").size(size),
            email_field: TextField = radix_theme.textfield("account-email").placeholder("Email").full_width(true),
            password_field: TextField = radix_theme
                .textfield("account-password")
                .placeholder("Password")
                .full_width(true),
            create_button: Entity<Button> = radix_theme.primary_button("account-create").label("Create account").size(size),
        },
        args: {
            radix_theme: Arc<RadixTheme>,
            size: ControlSize,
        },
        fields: {}
    }
}

impl Render for AccountPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();

        card(
            340.0,
            chrome.border,
            chrome.panel_background,
            vstack! {
                gap=12;
                card_header(
                    "Create an account",
                    "Enter your email below to create your account",
                    chrome.title_text,
                    chrome.muted_text,
                ),
                hstack! {
                    gap=8;
                    div().flex_1().child(self.github_button.clone()),
                    div().flex_1().child(self.google_button.clone()),
                },
                or_divider("OR CONTINUE WITH", chrome.border, chrome.muted_text),
                form_field!("Email", chrome; self.email_field.clone()),
                form_field!("Password", chrome; self.password_field.clone()),
                div().w_full().child(self.create_button.clone()),
            },
        )
    }
}
