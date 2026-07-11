#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_toggle_inspect_tree;
use super::super::button::labeling::render_vertical_section_rail;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

type ChoiceContentRenderer = dyn Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync;

#[derive(Clone)]
pub(in crate::gallery) struct TogglePane {
    secondary_toggle: Entity<Button<bool>>,
    primary_toggle: Entity<Button<bool>>,
    secondary_round_icon_toggle: Entity<Button<bool>>,
    primary_round_icon_toggle: Entity<Button<bool>>,
    state_preview: Entity<ToggleStatePreview>,
    inspector: Entity<ColorInspectorShell>,
    secondary_selected: bool,
    primary_selected: bool,
    secondary_round_icon_selected: bool,
    primary_round_icon_selected: bool,
}

impl TogglePane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree("toggle-inspector-tree", look.clone(), build_toggle_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "toggle-inspector",
                "toggle-inspector-split",
                "toggle-inspector-detail",
                build_toggle_inspect_tree,
                cx,
            )
        });

        Self {
            secondary_toggle: look
                .secondary_toggle("toggle-secondary-example")
                .with_data(true)
                .content(|_, _| div().child("Secondary").into_any_element())
                .spawn(cx),
            primary_toggle: look
                .primary_toggle("toggle-primary-example")
                .with_data(false)
                .content(|_, _| div().child("Primary").into_any_element())
                .spawn(cx),
            secondary_round_icon_toggle: look
                .secondary_toggle("toggle-secondary-round-icon-example")
                .with_data(false)
                .round(true)
                .content(|_, _| round_icon_glyph(false).into_any_element())
                .spawn(cx),
            primary_round_icon_toggle: look
                .primary_toggle("toggle-primary-round-icon-example")
                .with_data(true)
                .round(true)
                .content(|_, _| round_icon_glyph(true).into_any_element())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleStatePreview::new(look)),
            inspector,
            secondary_selected: true,
            primary_selected: false,
            secondary_round_icon_selected: false,
            primary_round_icon_selected: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.secondary_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_secondary_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_primary_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.secondary_round_icon_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_secondary_round_icon_toggle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.primary_round_icon_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.toggle.handle_primary_round_icon_toggle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector(
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
                        .child(self.primary_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary selected: {}", self.primary_selected)),
                        )
                        .child(self.secondary_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary selected: {}", self.secondary_selected)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.primary_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Primary icon selected: {}", self.primary_round_icon_selected)),
                        )
                        .child(self.secondary_round_icon_toggle.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.body_text)
                                .child(format!("Secondary icon selected: {}", self.secondary_round_icon_selected)),
                        ),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.secondary_toggle, cx);
        notify_entity(&self.primary_toggle, cx);
        notify_entity(&self.secondary_round_icon_toggle, cx);
        notify_entity(&self.primary_round_icon_toggle, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn flip_toggle(button: &Entity<Button<bool>>, selected: &mut bool, cx: &mut Context<GalleryApp>) {
        button.update(cx, |button, cx| {
            let new_selected = !*button.data();
            button.set_data(new_selected, cx);
            *selected = new_selected;
        });
    }

    fn handle_secondary_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_toggle(&self.secondary_toggle, &mut self.secondary_selected, cx);
            cx.notify();
        }
    }

    fn handle_primary_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            Self::flip_toggle(&self.primary_toggle, &mut self.primary_selected, cx);
            cx.notify();
        }
    }

    fn handle_secondary_round_icon_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.secondary_round_icon_toggle.update(cx, |button, cx| {
                let new_selected = !*button.data();
                button.set_data(new_selected, cx);
                button.set_presenter(Arc::new(move |_, _| round_icon_glyph(new_selected).into_any_element()), cx);
                self.secondary_round_icon_selected = new_selected;
            });
            cx.notify();
        }
    }

    fn handle_primary_round_icon_toggle_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.primary_round_icon_toggle.update(cx, |button, cx| {
                let new_selected = !*button.data();
                button.set_data(new_selected, cx);
                button.set_presenter(Arc::new(move |_, _| round_icon_glyph(new_selected).into_any_element()), cx);
                self.primary_round_icon_selected = new_selected;
            });
            cx.notify();
        }
    }
}

#[derive(Clone)]
struct ToggleStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ButtonTemplate<bool>>,
}

struct ToggleStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
    selected: bool,
}

#[derive(Clone, Copy)]
enum ToggleTemplateVariant {
    Text,
    RoundIcon,
}

impl ToggleTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::RoundIcon => "round-icon",
        }
    }

    fn round(self) -> bool {
        matches!(self, Self::RoundIcon)
    }

    fn content(self, selected: bool) -> Arc<ChoiceContentRenderer> {
        match self {
            Self::Text => {
                let label = SharedString::from("Toggle");
                Arc::new(move |_, _| div().child(label.clone()).into_any_element())
            }
            Self::RoundIcon => Arc::new(move |_, _| round_icon_glyph(selected).into_any_element()),
        }
    }
}

impl ToggleStatePreview {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look: look.clone(), template: look.toggle_template(ShadcnButtonStyle::Secondary) }
    }
}

impl Render for ToggleStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ToggleStateSample { id: "default", header: "default", state: InteractionState::default(), selected: false },
            ToggleStateSample {
                id: "hover",
                header: "hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
                selected: false,
            },
            ToggleStateSample {
                id: "focused",
                header: "focused",
                state: InteractionState { focused: true, ..InteractionState::default() },
                selected: false,
            },
            ToggleStateSample {
                id: "pressed",
                header: "pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
                selected: false,
            },
            ToggleStateSample {
                id: "selected",
                header: "selected",
                state: InteractionState::default(),
                selected: true,
            },
            ToggleStateSample {
                id: "disabled",
                header: "disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
                selected: false,
            },
            ToggleStateSample {
                id: "disabled-selected",
                header: "disabled · selected",
                state: InteractionState { disabled: true, ..InteractionState::default() },
                selected: true,
            },
        ];
        let variants = [ToggleTemplateVariant::Text, ToggleTemplateVariant::RoundIcon];

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
                    &self.look,
                    "Primary",
                    ShadcnButtonStyle::Primary,
                    &variants,
                    &samples,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_section(
                    &self.template,
                    &self.look,
                    "Secondary",
                    ShadcnButtonStyle::Secondary,
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
    look: &Arc<ShadcnLook>,
    section_label: &'static str,
    style: ShadcnButtonStyle,
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
                .children(
                    variants
                        .iter()
                        .map(|variant| render_variant_row(template, look, style, *variant, samples, window, cx)),
                ),
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
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variant: ToggleTemplateVariant,
    samples: &[ToggleStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .children(samples.iter().map(|sample| render_state_sample(template, look, style, variant, sample, window, cx)))
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ButtonTemplate<bool>>,
    look: &Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
    variant: ToggleTemplateVariant,
    sample: &ToggleStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = sample.selected;
    let id = SharedString::from(format!("toggle-preview-{}-{}-{}", shadcn_style_id(style), variant.id(), sample.id));
    let look = look_for_style(look.clone(), style);
    let model = ButtonRenderModel {
        id,
        data: selected,
        content: variant.content(selected),
        role: ButtonFamilyRole::Toggle { selected },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: Some(look),
    };

    div()
        .w(px(116.0))
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}

fn look_for_style(
    theme: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> gpui_luma::controls::command::button::ButtonLookSource<bool> {
    Arc::new(move |model| {
        let role = ButtonFamilyRole::Toggle { selected: model.data };
        match style {
            ShadcnButtonStyle::Primary => theme.as_ref().resolve_primary_button(role, model.size, model.state),
            ShadcnButtonStyle::Secondary => theme.as_ref().resolve_secondary_button(role, model.size, model.state),
            ShadcnButtonStyle::Outline => theme.as_ref().resolve_outline_button(role, model.size, model.state),
            ShadcnButtonStyle::Ghost => theme.as_ref().resolve_ghost_button(role, model.size, model.state),
        }
    })
}

fn shadcn_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
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
