use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px, rgb,
};
use gpui_luma::controls::command::button::ButtonKind;
use gpui_luma::controls::prototypes::mod_button::{
    Button, ButtonEvent, ButtonTemplate, DefaultButtonTemplate, HasContent,
};
use gpui_luma::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, ControlSize,
    DefaultButtonFamilyTheme, InteractionState,
};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

use super::super::super::shared::{gallery_pane_with_description, notify_entity};

#[derive(Clone, Default)]
struct CounterState {
    count: usize,
}

#[derive(Clone)]
pub(in crate::gallery) struct ModButtonPane {
    basic_button: Entity<Button>,
    modified_button: Entity<Button>,
    custom_button: Entity<Button>,
    reactive_button: Entity<Button<CounterState>>,

    standard_icon_only: Entity<Button>,
    standard_text_icon: Entity<Button>,
    standard_icon_text: Entity<Button>,

    prominent_icon_only: Entity<Button>,
    prominent_text_icon: Entity<Button>,
    prominent_icon_text: Entity<Button>,

    basic_clicks: usize,
    modified_clicks: usize,
    custom_clicks: usize,
}

impl ModButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let basic_button = Button::new("Button").spawn(cx);

        let modified_button = Button::new("Modified Button")
            .kind(ButtonKind::Standard)
            .template(modified_button_template::<()>(theme))
            .spawn(cx);

        let custom_button = Button::new("custom-btn")
            .content(|model, _| {
                let color = if model.state.hovered { rgb(0x4f46e5) } else { rgb(0x3730a3) };
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .font_family("lucide")
                            .text_size(px(16.0))
                            .child(char::from(LucideIcon::Check).to_string()),
                    )
                    .child("Custom Layout")
                    .text_color(color)
            })
            .kind(ButtonKind::Standard)
            .template(modified_button_template::<()>(theme))
            .spawn(cx);

        let reactive_button = Button::new("reactive-btn")
            .data(CounterState { count: 0 })
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .bg(rgb(0x4f46e5))
                            .text_color(rgb(0xffffff))
                            .px(px(6.0))
                            .rounded(px(4.0))
                            .child(model.data.count.to_string()),
                    )
                    .child("Reactive Counter")
            })
            .kind(ButtonKind::Standard)
            .template(modified_button_template::<CounterState>(theme))
            .spawn(cx);

        let standard_icon_only = Button::icon("std-icon-only", LucideIcon::Smile)
            .kind(ButtonKind::Standard)
            .spawn(cx);

        let standard_text_icon = Button::new("Label")
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::Smile))
            })
            .kind(ButtonKind::Standard)
            .spawn(cx);

        let standard_icon_text = Button::new("Label")
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::Smile))
                    .child("Label")
            })
            .kind(ButtonKind::Standard)
            .spawn(cx);

        let prominent_icon_only = Button::icon("prom-icon-only", LucideIcon::Smile)
            .kind(ButtonKind::Prominent)
            .spawn(cx);

        let prominent_text_icon = Button::new("Label")
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Label")
                    .child(render_lucide_icon(LucideIcon::Smile))
            })
            .kind(ButtonKind::Prominent)
            .spawn(cx);

        let prominent_icon_text = Button::new("Label")
            .content(|_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_lucide_icon(LucideIcon::Smile))
                    .child("Label")
            })
            .kind(ButtonKind::Prominent)
            .spawn(cx);

        Self {
            basic_button,
            modified_button,
            custom_button,
            reactive_button,
            standard_icon_only,
            standard_text_icon,
            standard_icon_text,
            prominent_icon_only,
            prominent_text_icon,
            prominent_icon_text,
            basic_clicks: 0,
            modified_clicks: 0,
            custom_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.basic_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.custom_button.handle_basic_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.modified_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.custom_button.handle_modified_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.custom_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.custom_button.handle_custom_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.reactive_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.custom_button.handle_reactive_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_description(
            "Command (Customized)",
            Some("Advanced: Unified Button architecture using custom templates, modifiers, and reactive presenters."),
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
                            "Basic Button",
                            "The unified Button using Button::new(\"Button\").",
                            self.basic_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "New Button (Text)",
                            "The unified Button using Button::new().",
                            self.modified_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "Custom Content",
                            "Using .content() for free-form layouts.",
                            self.custom_button.clone(),
                            chrome,
                        ))
                        .child(render_example_card(
                            "Reactive Button",
                            "State-driven data (CounterState).",
                            self.reactive_button.clone(),
                            chrome,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_color(chrome.title_text).font_weight(gpui::FontWeight::SEMIBOLD).child("Standard Factory Layouts"))
                        .child(
                            div()
                                .flex()
                                .gap(px(12.0))
                                .child(self.standard_icon_only.clone())
                                .child(self.standard_text_icon.clone())
                                .child(self.standard_icon_text.clone())
                        )
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_color(chrome.title_text).font_weight(gpui::FontWeight::SEMIBOLD).child("Prominent Factory Layouts"))
                        .child(
                            div()
                                .flex()
                                .gap(px(12.0))
                                .child(self.prominent_icon_only.clone())
                                .child(self.prominent_text_icon.clone())
                                .child(self.prominent_icon_text.clone())
                        )
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.basic_button, cx);
        notify_entity(&self.modified_button, cx);
        notify_entity(&self.custom_button, cx);
        notify_entity(&self.reactive_button, cx);
        notify_entity(&self.standard_icon_only, cx);
        notify_entity(&self.standard_text_icon, cx);
        notify_entity(&self.standard_icon_text, cx);
        notify_entity(&self.prominent_icon_only, cx);
        notify_entity(&self.prominent_text_icon, cx);
        notify_entity(&self.prominent_icon_text, cx);
    }

    fn handle_basic_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.basic_clicks += 1;
                let label = format!("Basic {}", self.basic_clicks);

                self.basic_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_modified_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.modified_clicks += 1;
                let label = format!("New Button {}", self.modified_clicks);

                self.modified_button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_custom_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.custom_clicks += 1;
                let label = format!("Customized {}", self.custom_clicks);

                self.custom_button.update(cx, |button, cx| {
                    button.set_content(Arc::new(move |model, _| {
                        let color = if model.state.hovered { rgb(0x4f46e5) } else { rgb(0x3730a3) };
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .font_family("lucide")
                                    .text_size(px(16.0))
                                    .child(char::from(LucideIcon::Check).to_string()),
                            )
                            .child(label.clone())
                            .text_color(color)
                            .into_any_element()
                    }), cx);
                });
            }
        }
    }

    fn handle_reactive_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.reactive_button.update(cx, |button, cx| {
                    let mut data = button.data().clone();
                    data.count += 1;
                    button.set_data(data, cx);
                });
            }
        }
    }
}

fn render_lucide_icon(icon: LucideIcon) -> AnyElement {
    div()
        .font_family("lucide")
        .text_size(px(16.0))
        .child(char::from(icon).to_string())
        .into_any_element()
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

fn modified_button_template<D: 'static>(theme: &GalleryThemePack) -> Arc<dyn ButtonTemplate<D>> {
    let mut template = DefaultButtonTemplate::new(Arc::new(GalleryModButtonFamilyTheme { theme: theme.clone() }));
    
    template = template.with_modifier(|el, model| {
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
                
                model.radius_override.set(Some(999.0));

                el.bg(background).border_color(border).text_color(foreground).rounded(px(999.0)).px(px(18.0))
            })
            .with_modifier(|el, model| {
                if model.state.pressed {
                    el.py(px(7.0))
                } else {
                    el.py(px(8.0))
                }
            });
            
    Arc::new(template)
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
