use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::icon_button::{
    IconButton, IconButtonEvent, IconButtonIcon, IconButtonKind, IconButtonRenderModel, IconButtonSize,
    IconButtonTemplate,
};
use gpui_luma::theme::InteractionState;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct IconButtonPane {
    icon_button: Entity<IconButton>,
    state_preview: Entity<IconButtonStatePreview>,
    icon_clicks: usize,
}

impl IconButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            icon_button: IconButton::new("icon-button-example", LucideIcon::Plus)
                .kind(IconButtonKind::Primary)
                .template(theme.icon_button_template())
                .spawn(cx),
            state_preview: cx.new(|_| IconButtonStatePreview::new(theme)),
            icon_clicks: 0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.icon_button, |app, _, event: &IconButtonEvent, cx| {
            app.panes.icon_button.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Icon Button",
            "Icon Button",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.icon_button.clone())
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.icon_button, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &IconButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            IconButtonEvent::Click => {
                self.icon_clicks += 1;
                let icon = if self.icon_clicks.is_multiple_of(2) {
                    LucideIcon::Plus
                } else {
                    LucideIcon::Check
                };

                self.icon_button.update(cx, |button, cx| {
                    button.set_icon(icon, cx);
                });
            }
        }
    }
}

#[derive(Clone)]
struct IconButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn IconButtonTemplate>,
}

struct IconButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl IconButtonStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.icon_button_template() }
    }
}

impl Render for IconButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            IconButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
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
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_state_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_state_sample(
    template: &Arc<dyn IconButtonTemplate>,
    sample: IconButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("icon-button-preview-{}", sample.id));
    let icon = IconButtonIcon::from(LucideIcon::Plus);
    let model = IconButtonRenderModel {
        id: &id,
        icon: &icon,
        kind: IconButtonKind::Primary,
        size: IconButtonSize::Md,
        state: sample.state,
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
