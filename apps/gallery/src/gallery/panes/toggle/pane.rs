use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{
    Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, HasPresenter,
};
use gpui_luma::controls::toggle::{self, default_toggle_template};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonKind, ButtonSize};
use gpui_luma::theme::InteractionState;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TogglePane {
    standard_toggle: Entity<Button<bool>>,
    prominent_toggle: Entity<Button<bool>>,
    standard_round_icon_toggle: Entity<Button<bool>>,
    prominent_round_icon_toggle: Entity<Button<bool>>,
    state_preview: Entity<ToggleStatePreview>,
    standard_selected: bool,
    prominent_selected: bool,
    standard_round_icon_selected: bool,
    prominent_round_icon_selected: bool,
}

impl TogglePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            standard_toggle: toggle::new("toggle-standard-example")
                .kind(ButtonKind::Standard)
                .with_data(true)
                .content(|_, _| div().child("Standard").into_any_element())
                .spawn(cx),
            prominent_toggle: toggle::new("toggle-prominent-example")
                .kind(ButtonKind::Prominent)
                .with_data(false)
                .content(|_, _| div().child("Prominent").into_any_element())
                .spawn(cx),
            standard_round_icon_toggle: toggle::new("toggle-standard-round-icon-example")
                .kind(ButtonKind::Standard)
                .with_data(false)
                .round(true)
                .content(|_, _| round_icon_glyph(false).into_any_element())
                .spawn(cx),
            prominent_round_icon_toggle: toggle::new("toggle-prominent-round-icon-example")
                .kind(ButtonKind::Prominent)
                .with_data(true)
                .round(true)
                .content(|_, _| round_icon_glyph(true).into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleStatePreview::new(theme)),
            standard_selected: true,
            prominent_selected: false,
            standard_round_icon_selected: false,
            prominent_round_icon_selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.standard_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_standard_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.prominent_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_prominent_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.standard_round_icon_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_standard_round_icon_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.prominent_round_icon_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_prominent_round_icon_toggle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Toggle",
            "Toggle",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.standard_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Standard selected: {}", self.standard_selected)),
                        )
                        .child(self.prominent_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Prominent selected: {}", self.prominent_selected)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.standard_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Standard icon selected: {}", self.standard_round_icon_selected)),
                        )
                        .child(self.prominent_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Prominent icon selected: {}", self.prominent_round_icon_selected)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.standard_toggle, cx);
        notify_entity(&self.prominent_toggle, cx);
        notify_entity(&self.standard_round_icon_toggle, cx);
        notify_entity(&self.prominent_round_icon_toggle, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn flip_toggle(button: &Entity<Button<bool>>, selected: &mut bool, cx: &mut Context<GalleryApp>) {
        button.update(cx, |button, cx| {
            let new_selected = !*button.data();
            button.set_data(new_selected, cx);
            *selected = new_selected;
        });
    }

    fn handle_standard_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_toggle(&self.standard_toggle, &mut self.standard_selected, cx);
            cx.notify();
        }
    }

    fn handle_prominent_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_toggle(&self.prominent_toggle, &mut self.prominent_selected, cx);
            cx.notify();
        }
    }

    fn handle_standard_round_icon_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.standard_round_icon_toggle.update(cx, |button, cx| {
                let new_selected = !*button.data();
                button.set_data(new_selected, cx);
                button.set_presenter(Arc::new(move |_, _| round_icon_glyph(new_selected).into_any_element()), cx);
                self.standard_round_icon_selected = new_selected;
            });
            cx.notify();
        }
    }

    fn handle_prominent_round_icon_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.prominent_round_icon_toggle.update(cx, |button, cx| {
                let new_selected = !*button.data();
                button.set_data(new_selected, cx);
                button.set_presenter(Arc::new(move |_, _| round_icon_glyph(new_selected).into_any_element()), cx);
                self.prominent_round_icon_selected = new_selected;
            });
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct ToggleStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ButtonTemplate<bool>>,
}

struct ToggleStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum ToggleTemplateVariant {
    TextUnselected,
    TextSelected,
    RoundIconUnselected,
    RoundIconSelected,
}

impl ToggleTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::TextUnselected => "text-unselected",
            Self::TextSelected => "text-selected",
            Self::RoundIconUnselected => "round-icon-unselected",
            Self::RoundIconSelected => "round-icon-selected",
        }
    }

    fn selected(self) -> bool {
        matches!(self, Self::TextSelected | Self::RoundIconSelected)
    }

    fn round(self) -> bool {
        matches!(self, Self::RoundIconUnselected | Self::RoundIconSelected)
    }

    fn content(self) -> Arc<dyn Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync> {
        match self {
            Self::TextUnselected | Self::TextSelected => {
                let label = SharedString::from("Toggle");
                Arc::new(move |_, _| div().child(label.clone()).into_any_element())
            }
            Self::RoundIconUnselected => Arc::new(move |_, _| round_icon_glyph(false).into_any_element()),
            Self::RoundIconSelected => Arc::new(move |_, _| round_icon_glyph(true).into_any_element()),
        }
    }
}

impl ToggleStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: default_toggle_template() }
    }
}

impl Render for ToggleStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ToggleStateSample { id: "default", header: "default", state: InteractionState::default() },
            ToggleStateSample {
                id: "hover",
                header: "hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ToggleStateSample {
                id: "focused",
                header: "focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ToggleStateSample {
                id: "pressed",
                header: "pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ToggleStateSample {
                id: "disabled",
                header: "disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];
        let variants = [
            ToggleTemplateVariant::TextUnselected,
            ToggleTemplateVariant::TextSelected,
            ToggleTemplateVariant::RoundIconUnselected,
            ToggleTemplateVariant::RoundIconSelected,
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template matrix preview"),
            )
            .child(div().flex().flex_col().items_start().gap(px(20.0)).children([
                render_section(
                    &self.template,
                    "Prominent",
                    ButtonKind::Prominent,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    "Standard",
                    ButtonKind::Standard,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
    }
}

fn render_section(
    template: &Arc<dyn ButtonTemplate<bool>>,
    section_label: &'static str,
    kind: ButtonKind,
    variants: &[ToggleTemplateVariant],
    samples: &[ToggleStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_start()
        .gap(px(8.0))
        .child(render_vertical_section_rail(section_label, label_color))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(render_header_row(samples, label_color))
                .children(variants.iter().map(|variant| {
                    render_variant_row(template, kind, *variant, samples, window, cx)
                })),
        )
        .into_any_element()
}

fn render_header_row(samples: &[ToggleStateSample], label_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| {
            div()
                .w(px(116.0))
                .flex()
                .justify_center()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(label_color)
                .child(sample.header)
        }))
        .into_any_element()
}

fn render_variant_row(
    template: &Arc<dyn ButtonTemplate<bool>>,
    kind: ButtonKind,
    variant: ToggleTemplateVariant,
    samples: &[ToggleStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| render_state_sample(template, kind, variant, sample, window, cx)))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    kind: ButtonKind,
    variant: ToggleTemplateVariant,
    sample: &ToggleStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = variant.selected();
    let id = SharedString::from(format!("toggle-preview-{}-{}-{}", button_kind_id(kind), variant.id(), sample.id));
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: variant.content(),
        kind,
        role: ButtonFamilyRole::Toggle { selected },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
    };

    div()
        .w(px(116.0))
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn button_kind_id(kind: ButtonKind) -> &'static str {
    match kind {
        ButtonKind::Prominent => "prominent",
        ButtonKind::Subtle => "subtle",
        ButtonKind::Standard => "standard",
        ButtonKind::Ghost => "ghost",
    }
}

fn round_icon_glyph(selected: bool) -> impl IntoElement {
    let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };

    div()
        .font_family("lucide")
        .text_size(px(16.0))
        .line_height(px(16.0))
        .child(char::from(icon).to_string())
}
