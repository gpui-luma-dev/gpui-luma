use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, Hsla, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*,
    px,
};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::prototypes::proto_button::{
    ProtoButton, ProtoButtonEvent, ProtoButtonRenderModel, ProtoButtonSize, ProtoButtonStatefulOverride,
    ProtoButtonTemplate, ProtoButtonTemplateParams, ProtoButtonVisualState, ThemedProtoButtonTemplate,
    proto_button_template_usage,
};
use gpui_luma::theme::{ButtonFamilyRole, ButtonVariant, ControlSize, InteractionState, default_button_family_theme};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

use super::super::super::shared::{format_compact_hsla, gallery_pane_with_description, notify_entity};
use super::param_panel::render_proto_button_param_panel;

const DEMO_DISABLED_OPACITY: f32 = 0.72;
const DEMO_RADIUS: f32 = 0.0;
const DEMO_BACKGROUND: Hsla = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 1.0 };
const DEMO_FOREGROUND: Hsla = Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 };

const MIN_RADIUS: f32 = 0.0;
const MAX_RADIUS: f32 = 24.0;
const RADIUS_STEP: f32 = 2.0;

#[derive(Clone)]
pub(in crate::gallery) struct ProtoButtonPane {
    default_button: Entity<ProtoButton>,
    demo_button: Entity<ProtoButton>,
    state_preview: Entity<ProtoButtonStatePreview>,

    radius_down_button: Entity<Button>,
    radius_up_button: Entity<Button>,
    flip_bg_fg_button: Entity<Button>,
    reset_button: Entity<Button>,
    state_cycle_button: Entity<Button>,

    demo_clicks: usize,
    demo_radius: f32,
    demo_colors_flipped: bool,
    selected_visual_state: ProtoButtonVisualState,
    demo_background_overrides: ProtoButtonStatefulOverride<Hsla>,
    demo_foreground_overrides: ProtoButtonStatefulOverride<Hsla>,
}

impl ProtoButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let default_button = ProtoButton::new("proto-button-default").label("Default ProtoButton").spawn(cx);

        let demo_template = demo_proto_button_template();
        let demo_button =
            ProtoButton::new("proto-button-demo").label("Demo Action").template(demo_template.clone()).spawn(cx);
        let demo_preview_template = demo_template.clone();
        let state_preview = cx.new(move |_| ProtoButtonStatePreview::new(theme, demo_preview_template.clone()));

        let radius_down_button = Button::new("proto-button-radius-down")
            .label("Radius -")
            .kind(ButtonKind::Default)
            .template(theme.button_template())
            .spawn(cx);
        let radius_up_button = Button::new("proto-button-radius-up")
            .label("Radius +")
            .kind(ButtonKind::Default)
            .template(theme.button_template())
            .spawn(cx);

        let flip_bg_fg_button = Button::new("proto-button-flip-bg-fg")
            .label("Flip bg/fg")
            .kind(ButtonKind::Default)
            .template(theme.button_template())
            .spawn(cx);

        let reset_button = Button::new("proto-button-reset")
            .label("Reset")
            .kind(ButtonKind::Default)
            .template(theme.button_template())
            .spawn(cx);

        let state_cycle_button = Button::new("proto-button-state-cycle")
            .label(format!("State: {}", visual_state_label(ProtoButtonVisualState::Default)))
            .kind(ButtonKind::Default)
            .template(theme.button_template())
            .spawn(cx);

        Self {
            default_button,
            demo_button,
            state_preview,
            radius_down_button,
            radius_up_button,
            flip_bg_fg_button,
            reset_button,
            state_cycle_button,
            demo_clicks: 0,
            demo_radius: DEMO_RADIUS,
            demo_colors_flipped: false,
            selected_visual_state: ProtoButtonVisualState::Default,
            demo_background_overrides: ProtoButtonStatefulOverride {
                base: Some(DEMO_BACKGROUND),
                ..Default::default()
            },
            demo_foreground_overrides: ProtoButtonStatefulOverride {
                base: Some(DEMO_FOREGROUND),
                ..Default::default()
            },
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.demo_button, |app, _, event: &ProtoButtonEvent, cx| {
            app.panes.proto_button.handle_proto_button_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.radius_down_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.proto_button.handle_radius_down(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.radius_up_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.proto_button.handle_radius_up(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.flip_bg_fg_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.proto_button.handle_flip_bg_fg(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.reset_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.proto_button.handle_reset(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.state_cycle_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.proto_button.handle_cycle_state(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();
        let usage = proto_button_template_usage();

        gallery_pane_with_description(
            "ProtoButton",
            Some("Prototype: template parameterization isolated under controls/prototypes."),
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .items_stretch()
                .justify_center()
                .gap(px(28.0))
                .child(self.render_demo_column(chrome, theme))
                .child(render_proto_button_param_panel(
                    usage,
                    chrome,
                    theme,
                    visual_state_label(self.selected_visual_state),
                ))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_button, cx);
        notify_entity(&self.demo_button, cx);
        notify_entity(&self.state_preview, cx);

        notify_entity(&self.radius_down_button, cx);
        notify_entity(&self.radius_up_button, cx);
        notify_entity(&self.flip_bg_fg_button, cx);
        notify_entity(&self.reset_button, cx);
        notify_entity(&self.state_cycle_button, cx);
    }

    fn render_demo_column(&self, chrome: GalleryChrome, _theme: &GalleryThemePack) -> AnyElement {
        let theme_appearance = default_button_family_theme().resolve(
            ButtonVariant::Default,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            interaction_state_for_visual_state(self.selected_visual_state),
        );
        let (effective_background, background_source) = resolve_color_with_source(
            &self.demo_background_overrides,
            self.selected_visual_state,
            theme_appearance.background,
        );
        let (effective_foreground, foreground_source) = resolve_color_with_source(
            &self.demo_foreground_overrides,
            self.selected_visual_state,
            theme_appearance.foreground,
        );

        div()
            .min_w(px(0.0))
            .h_full()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.0))
            .child(self.default_button.clone())
            .child(self.demo_button.clone())
            .child(self.state_preview.clone())
            .child(
                div()
                    .mt(px(8.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(chrome.muted_text)
                            .child(format!("radius: {:.1}px", self.demo_radius)),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(chrome.muted_text)
                            .child(format!("colors flipped: {}", if self.demo_colors_flipped { "on" } else { "off" })),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(chrome.muted_text)
                            .child(format!("selected state: {}", visual_state_label(self.selected_visual_state))),
                    ),
            )
            .child(
                div()
                    .w(px(420.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .border_1()
                    .border_color(chrome.border)
                    .rounded(px(6.0))
                    .p(px(10.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(15.0))
                            .font_family("Monaco")
                            .text_color(chrome.muted_text)
                            .child("effective/source inspector"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .line_height(px(16.0))
                                    .text_color(chrome.body_text)
                                    .child("background"),
                            )
                            .child(
                                div()
                                    .font_family("Monaco")
                                    .text_size(px(11.0))
                                    .line_height(px(15.0))
                                    .text_color(chrome.muted_text)
                                    .child(format_compact_hsla(effective_background)),
                            )
                            .child(
                                div()
                                    .font_family("Monaco")
                                    .text_size(px(10.0))
                                    .line_height(px(14.0))
                                    .text_color(chrome.muted_text)
                                    .child(background_source),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .line_height(px(16.0))
                                    .text_color(chrome.body_text)
                                    .child("foreground"),
                            )
                            .child(
                                div()
                                    .font_family("Monaco")
                                    .text_size(px(11.0))
                                    .line_height(px(15.0))
                                    .text_color(chrome.muted_text)
                                    .child(format_compact_hsla(effective_foreground)),
                            )
                            .child(
                                div()
                                    .font_family("Monaco")
                                    .text_size(px(10.0))
                                    .line_height(px(14.0))
                                    .text_color(chrome.muted_text)
                                    .child(foreground_source),
                            ),
                    ),
            )
            .child(
                div()
                    .mt(px(4.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(self.radius_down_button.clone())
                            .child(self.radius_up_button.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(self.flip_bg_fg_button.clone())
                            .child(self.reset_button.clone())
                            .child(self.state_cycle_button.clone()),
                    ),
            )
            .into_any_element()
    }

    fn handle_proto_button_event(&mut self, event: &ProtoButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ProtoButtonEvent::Click) {
            self.demo_clicks += 1;
            let label = format!("Demo Action ({})", self.demo_clicks);

            self.demo_button.update(cx, |button, cx| {
                button.set_label(label, cx);
            });
        }
    }

    fn handle_radius_down(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.demo_radius = (self.demo_radius - RADIUS_STEP).clamp(MIN_RADIUS, MAX_RADIUS);
            self.apply_template_params(cx);
        }
    }

    fn handle_radius_up(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.demo_radius = (self.demo_radius + RADIUS_STEP).clamp(MIN_RADIUS, MAX_RADIUS);
            self.apply_template_params(cx);
        }
    }

    fn handle_flip_bg_fg(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.demo_colors_flipped = !self.demo_colors_flipped;
            self.apply_template_params(cx);
        }
    }

    fn handle_reset(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.demo_radius = DEMO_RADIUS;
            self.demo_colors_flipped = false;

            self.apply_template_params(cx);
        }
    }

    fn handle_cycle_state(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.selected_visual_state = next_visual_state(self.selected_visual_state);
            let label = format!("State: {}", visual_state_label(self.selected_visual_state));

            self.state_cycle_button.update(cx, |button, cx| {
                button.set_label(label, cx);
            });

            cx.notify();
        }
    }

    fn apply_template_params(&mut self, cx: &mut Context<GalleryApp>) {
        let radius = self.demo_radius;
        let foreground = if self.demo_colors_flipped {
            Some(DEMO_BACKGROUND)
        } else {
            Some(DEMO_FOREGROUND)
        };
        let background = if self.demo_colors_flipped {
            Some(DEMO_FOREGROUND)
        } else {
            Some(DEMO_BACKGROUND)
        };

        write_state_override(&mut self.demo_background_overrides, self.selected_visual_state, background.clone());
        write_state_override(&mut self.demo_foreground_overrides, self.selected_visual_state, foreground.clone());

        self.demo_button.update(cx, |button, cx| {
            let mut params = button.template_params().unwrap_or_default();
            params.size = ControlSize::Md;
            params.disabled_opacity = DEMO_DISABLED_OPACITY;
            params.radius = Some(radius);
            write_state_override(&mut params.background, self.selected_visual_state, background);
            write_state_override(&mut params.foreground, self.selected_visual_state, foreground);
            let _ = button.set_template_params(params, cx);
        });

        cx.notify();
    }
}

#[derive(Clone)]
struct ProtoButtonStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ProtoButtonTemplate>,
}

struct ProtoButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ProtoButtonStatePreview {
    fn new(theme: &GalleryThemePack, template: Arc<dyn ProtoButtonTemplate>) -> Self {
        Self { theme: theme.clone(), template }
    }
}

impl Render for ProtoButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ProtoButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ProtoButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ProtoButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ProtoButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ProtoButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .mt(px(2.0))
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
                        .map(|sample| render_proto_state_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_proto_state_sample(
    template: &Arc<dyn ProtoButtonTemplate>,
    sample: ProtoButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("proto-button-preview-{}", sample.id));
    let label = SharedString::from("ProtoButton");
    let model = ProtoButtonRenderModel { id: &id, label: &label, size: ProtoButtonSize::Md, state: sample.state };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn visual_state_label(state: ProtoButtonVisualState) -> &'static str {
    match state {
        ProtoButtonVisualState::Default => "default",
        ProtoButtonVisualState::Hovered => "hovered",
        ProtoButtonVisualState::Pressed => "pressed",
        ProtoButtonVisualState::Focused => "focused",
        ProtoButtonVisualState::Disabled => "disabled",
    }
}

fn next_visual_state(state: ProtoButtonVisualState) -> ProtoButtonVisualState {
    match state {
        ProtoButtonVisualState::Default => ProtoButtonVisualState::Hovered,
        ProtoButtonVisualState::Hovered => ProtoButtonVisualState::Pressed,
        ProtoButtonVisualState::Pressed => ProtoButtonVisualState::Focused,
        ProtoButtonVisualState::Focused => ProtoButtonVisualState::Disabled,
        ProtoButtonVisualState::Disabled => ProtoButtonVisualState::Default,
    }
}

fn write_state_override<T>(
    overrides: &mut ProtoButtonStatefulOverride<T>,
    state: ProtoButtonVisualState,
    value: Option<T>,
) {
    match state {
        ProtoButtonVisualState::Default => overrides.base = value,
        ProtoButtonVisualState::Hovered => overrides.hovered = value,
        ProtoButtonVisualState::Pressed => overrides.pressed = value,
        ProtoButtonVisualState::Focused => overrides.focused = value,
        ProtoButtonVisualState::Disabled => overrides.disabled = value,
    }
}

fn interaction_state_for_visual_state(state: ProtoButtonVisualState) -> InteractionState {
    match state {
        ProtoButtonVisualState::Default => {
            InteractionState { hovered: false, pressed: false, focused: false, disabled: false }
        }
        ProtoButtonVisualState::Hovered => {
            InteractionState { hovered: true, pressed: false, focused: false, disabled: false }
        }
        ProtoButtonVisualState::Pressed => {
            InteractionState { hovered: false, pressed: true, focused: false, disabled: false }
        }
        ProtoButtonVisualState::Focused => {
            InteractionState { hovered: false, pressed: false, focused: true, disabled: false }
        }
        ProtoButtonVisualState::Disabled => {
            InteractionState { hovered: false, pressed: false, focused: false, disabled: true }
        }
    }
}

fn resolve_color_with_source(
    overrides: &ProtoButtonStatefulOverride<Hsla>,
    state: ProtoButtonVisualState,
    theme_value: Hsla,
) -> (Hsla, &'static str) {
    if let Some(value) = overrides.for_state(state).copied() {
        (value, "state override")
    } else if let Some(value) = overrides.base {
        (value, "base override")
    } else {
        (theme_value, "theme")
    }
}

fn demo_proto_button_template() -> Arc<dyn ProtoButtonTemplate> {
    let params = ProtoButtonTemplateParams {
        variant: ButtonVariant::Default,
        size: ControlSize::Md,
        disabled_opacity: DEMO_DISABLED_OPACITY,
        radius: Some(DEMO_RADIUS),
        background: ProtoButtonStatefulOverride { base: Some(DEMO_BACKGROUND), ..Default::default() },
        foreground: ProtoButtonStatefulOverride { base: Some(DEMO_FOREGROUND), ..Default::default() },
        ..Default::default()
    };

    Arc::new(ThemedProtoButtonTemplate::new(default_button_family_theme()).with_params(params))
}
