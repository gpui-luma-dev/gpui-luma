use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel, ButtonTemplate, default_button_template};
use gpui_luma::controls::command::icon_button::{self, IconButton};
use gpui_luma::controls::button_family::{ButtonKind, ButtonSize};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::theme::InteractionState;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct IconButtonPane {
    default_icon_button: IconButton,
    subtle_icon_button: IconButton,
    ghost_icon_button: IconButton,
    prominent_icon_button: IconButton,
    state_preview: Entity<IconButtonStatePreview>,
    default_icon_clicks: usize,
    subtle_icon_clicks: usize,
    ghost_icon_clicks: usize,
    prominent_icon_clicks: usize,
}

impl IconButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            default_icon_button: icon_button::new("icon-button-default-example", LucideIcon::Plus)
                .kind(ButtonKind::Standard)
                .spawn(cx),
            subtle_icon_button: icon_button::new("icon-button-subtle-example", LucideIcon::Plus)
                .kind(ButtonKind::Subtle)
                .spawn(cx),
            ghost_icon_button: icon_button::new("icon-button-ghost-example", LucideIcon::Plus)
                .kind(ButtonKind::Ghost)
                .spawn(cx),
            prominent_icon_button: icon_button::new("icon-button-prominent-example", LucideIcon::Plus)
                .kind(ButtonKind::Prominent)
                .spawn(cx),
            state_preview: cx.new(|_| IconButtonStatePreview::new(theme)),
            default_icon_clicks: 0,
            subtle_icon_clicks: 0,
            ghost_icon_clicks: 0,
            prominent_icon_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_default_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.subtle_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_subtle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.ghost_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_ghost_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.prominent_icon_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.icon_button.handle_prominent_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Command (Icon)",
            "Icon Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(self.default_icon_button.clone())
                        .child(self.subtle_icon_button.clone())
                        .child(self.ghost_icon_button.clone())
                        .child(self.prominent_icon_button.clone()),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_icon_button, cx);
        notify_entity(&self.subtle_icon_button, cx);
        notify_entity(&self.ghost_icon_button, cx);
        notify_entity(&self.prominent_icon_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn toggle_icon(button: &IconButton, clicks: &mut usize, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                *clicks += 1;
                let icon = if clicks.is_multiple_of(2) {
                    LucideIcon::Plus
                } else {
                    LucideIcon::Check
                };

                button.update(cx, |button, cx| {
                    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> =
                        Arc::new(move |_, _| {
                            div()
                                .font_family("lucide")
                                .text_size(px(16.0))
                                .child(char::from(icon).to_string())
                                .into_any_element()
                        });
                    button.set_presenter(content, cx);
                });
            }
        }
    }

    fn handle_default_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.default_icon_button, &mut self.default_icon_clicks, event, cx);
    }

    fn handle_subtle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.subtle_icon_button, &mut self.subtle_icon_clicks, event, cx);
    }

    fn handle_ghost_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.ghost_icon_button, &mut self.ghost_icon_clicks, event, cx);
    }

    fn handle_prominent_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        Self::toggle_icon(&self.prominent_icon_button, &mut self.prominent_icon_clicks, event, cx);
    }
}

#[derive(Clone)]
struct IconButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ButtonTemplate<()>>,
}

struct IconButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl IconButtonStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: default_button_template() }
    }
}

impl Render for IconButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            IconButtonStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            IconButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            IconButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            IconButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            IconButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(render_state_row(
                &self.template,
                "prominent",
                "Prominent",
                ButtonKind::Prominent,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                "subtle",
                "Subtle",
                ButtonKind::Subtle,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                "default",
                "Standard",
                ButtonKind::Standard,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                "ghost",
                "Ghost",
                ButtonKind::Ghost,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_state_row(
    template: &Arc<dyn ButtonTemplate<()>>,
    row_id: &'static str,
    row_label: &'static str,
    kind: ButtonKind,
    samples: &[IconButtonStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples
                    .iter()
                    .map(|sample| render_state_sample(template, row_id, kind, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    row_id: &'static str,
    kind: ButtonKind,
    sample: &IconButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("icon-button-preview-{}-{}", row_id, sample.id));
    let icon = LucideIcon::Plus;
    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> =
        Arc::new(move |_: &ButtonRenderModel<()>, _| {
            div()
                .font_family("lucide")
                .text_size(px(16.0))
                .child(char::from(icon).to_string())
                .into_any_element()
        });
    let model = ButtonRenderModel {
        id,
        data: (),
        content,
        kind,
        role: ButtonFamilyRole::Icon,
        size: ButtonSize::Md,
        state: sample.state,
        round: true,
        radius_override: std::cell::Cell::new(None),
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
