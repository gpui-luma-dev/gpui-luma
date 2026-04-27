use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px, rgb};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::prototypes::mod_button::{ButtonTemplate, ModButton, ModButtonEvent, ModButtonTemplate};
use gpui_luma::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, ControlSize, DefaultButtonFamilyTheme,
    InteractionState,
};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

use super::super::super::shared::{gallery_pane_with_description, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ModButtonPane {
    command_button: Entity<Button>,
    modified_button: Entity<ModButton>,
    command_clicks: usize,
    modified_clicks: usize,
}

impl ModButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let modified_template = modified_mod_button_template(theme);

        Self {
            command_button: Button::new("mod-button-command-example").label("Command Button").spawn(cx),
            modified_button: ModButton::new("mod-button-modified-example")
                .label("Modified ModButton")
                .kind(ButtonKind::Standard)
                .template(modified_template)
                .spawn(cx),
            command_clicks: 0,
            modified_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.command_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.mod_button.handle_command_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.modified_button, |app, _, event: &ModButtonEvent, cx| {
            app.panes.mod_button.handle_modified_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_description(
            "ModButton",
            Some("Prototype: command Button behavior with a modifier-pipeline template."),
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_stretch()
                        .justify_center()
                        .gap(px(16.0))
                        .child(render_example_card(
                            "Command Button",
                            "The existing controls/command/button rendered with the gallery button template.",
                            self.command_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "Modified ModButton",
                            "The new controls/prototypes/mod_button with modifiers applied after theme resolution.",
                            self.modified_button.clone(),
                            chrome,
                        )),
                )
                .child(render_modifier_notes(chrome))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.command_button, cx);
        notify_entity(&self.modified_button, cx);
    }

    fn handle_command_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.command_clicks += 1;
                let label = format!("Command {}", self.command_clicks);

                self.command_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_modified_event(&mut self, event: &ModButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ModButtonEvent::Click => {
                self.modified_clicks += 1;
                let label = format!("Modified {}", self.modified_clicks);

                self.modified_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }
}

fn render_example_card(
    title: &'static str,
    description: &'static str,
    control: impl IntoElement,
    chrome: GalleryChrome,
) -> AnyElement {
    div()
        .w(px(320.0))
        .min_h(px(168.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(14.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .p(px(16.0))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .line_height(px(18.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div().text_size(px(12.0)).line_height(px(17.0)).text_color(chrome.muted_text).child(description),
                ),
        )
        .child(control)
        .into_any_element()
}

fn render_modifier_notes(chrome: GalleryChrome) -> AnyElement {
    div()
        .max_w(px(700.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .p(px(14.0))
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Modifier pipeline example"),
        )
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(17.0))
                .text_color(chrome.body_text)
                .child("The ModButton starts from the same themed command-button base, then runs modifiers over the Stateful<Div> before the focus-ring wrapper is applied."),
        )
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(17.0))
                .text_color(chrome.muted_text)
                .child("This example modifier reads model.state to swap background, border, and foreground colors for default, hover, pressed, focused, and disabled states."),
        )
        .into_any_element()
}

fn modified_mod_button_template(theme: &GalleryThemePack) -> Arc<dyn ModButtonTemplate> {
    Arc::new(
        ButtonTemplate::new(Arc::new(GalleryModButtonFamilyTheme { theme: theme.clone() }))
            .with_modifier(|el, model| {
                let (background, border, foreground) = if model.state.disabled {
                    (rgb(0xf3f4f6), rgb(0xe5e7eb), rgb(0x9ca3af))
                } else if model.state.pressed {
                    (rgb(0x4338ca), rgb(0x312e81), rgb(0xffffff))
                } else if model.state.hovered {
                    (rgb(0x6366f1), rgb(0x4f46e5), rgb(0xffffff))
                } else if model.state.focused {
                    (rgb(0xeef2ff), rgb(0x4f46e5), rgb(0x3730a3))
                } else {
                    (rgb(0xf5f3ff), rgb(0xc4b5fd), rgb(0x5b21b6))
                };

                el.bg(background).border_color(border).text_color(foreground).rounded(px(999.0)).px(px(18.0))
            })
            .with_modifier(|el, model| {
                if model.state.pressed {
                    el.py(px(7.0))
                } else {
                    el.py(px(8.0))
                }
            }),
    )
}

#[derive(Clone)]
struct GalleryModButtonFamilyTheme {
    theme: GalleryThemePack,
}

impl ButtonFamilyTheme for GalleryModButtonFamilyTheme {
    fn resolve(
        &self,
        variant: ButtonVariant,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
    ) -> ButtonFamilyAppearance {
        DefaultButtonFamilyTheme::new(self.theme.tokens()).resolve(variant, role, size, state)
    }
}
