use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, Hsla, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*,
    px, rgb,
};
use gpui_luma::controls::command::button::{
    Button, ButtonEvent, ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate, HasPresenter,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize, default_button_family_theme};
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{InteractionState, RadixTheme, ThemeMode};

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, gallery_pane_with_description, notify_entity};
use crate::gallery::theme::GalleryChrome;

const DEMO_RADIUS: f32 = 0.0;
const DEMO_BACKGROUND: Hsla = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 1.0 };
const DEMO_FOREGROUND: Hsla = Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 };

const MIN_RADIUS: f32 = 0.0;
const MAX_RADIUS: f32 = 24.0;
const RADIUS_STEP: f32 = 2.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonVisualState {
    Default,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

impl From<InteractionState> for ButtonVisualState {
    fn from(state: InteractionState) -> Self {
        if state.disabled {
            Self::Disabled
        } else if state.pressed {
            Self::Pressed
        } else if state.hovered {
            Self::Hovered
        } else if state.focused {
            Self::Focused
        } else {
            Self::Default
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ButtonStatefulOverride<T> {
    pub base: Option<T>,
    pub hovered: Option<T>,
    pub pressed: Option<T>,
    pub focused: Option<T>,
    pub disabled: Option<T>,
}

impl<T> ButtonStatefulOverride<T> {
    pub fn for_state(&self, state: ButtonVisualState) -> Option<&T> {
        match state {
            ButtonVisualState::Default => None,
            ButtonVisualState::Hovered => self.hovered.as_ref(),
            ButtonVisualState::Pressed => self.pressed.as_ref(),
            ButtonVisualState::Focused => self.focused.as_ref(),
            ButtonVisualState::Disabled => self.disabled.as_ref(),
        }
    }

    pub fn resolve(&self, state: ButtonVisualState) -> Option<&T> {
        self.for_state(state).or(self.base.as_ref())
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    radix_theme: Arc<RadixTheme>,
    default_button: Entity<Button>,
    danger_button: Entity<Button>,
    demo_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,

    radius_down_button: Entity<Button>,
    radius_up_button: Entity<Button>,
    flip_bg_fg_button: Entity<Button>,
    reset_button: Entity<Button>,
    state_cycle_button: Entity<Button>,

    demo_clicks: usize,
    demo_radius: f32,
    demo_colors_flipped: bool,
    selected_visual_state: ButtonVisualState,
    demo_background_overrides: ButtonStatefulOverride<Hsla>,
    demo_foreground_overrides: ButtonStatefulOverride<Hsla>,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let default_button = radix_theme.secondary_button("button-default").label("Default ProtoButton").spawn(cx);

        let danger_button = Button::new("button-danger")
            .label("Danger Action")
            .template(danger_button_template(&radix_theme))
            .spawn(cx);

        let demo_radius = DEMO_RADIUS;
        let demo_background_overrides = ButtonStatefulOverride { base: Some(DEMO_BACKGROUND), ..Default::default() };
        let demo_foreground_overrides = ButtonStatefulOverride { base: Some(DEMO_FOREGROUND), ..Default::default() };

        let demo_template =
            demo_button_template(&demo_background_overrides, &demo_foreground_overrides, demo_radius, &radix_theme);
        let demo_button = Button::new("button-demo").label("Demo Action").template(demo_template.clone()).spawn(cx);

        let preview_theme = radix_theme.clone();
        let state_preview =
            cx.new(move |_| ButtonStatePreview::new(preview_theme, demo_template.clone(), Some(demo_radius)));

        let radius_down_button = radix_theme.secondary_button("button-radius-down").label("Radius -").spawn(cx);
        let radius_up_button = radix_theme.secondary_button("button-radius-up").label("Radius +").spawn(cx);

        let flip_bg_fg_button = radix_theme.secondary_button("button-flip-bg-fg").label("Flip bg/fg").spawn(cx);

        let reset_button = radix_theme.secondary_button("button-reset").label("Reset").spawn(cx);

        let state_cycle_button = radix_theme
            .secondary_button("button-state-cycle")
            .label(format!("State: {}", visual_state_label(ButtonVisualState::Default)))
            .spawn(cx);

        Self {
            radix_theme,
            default_button,
            danger_button,
            demo_button,
            state_preview,
            radius_down_button,
            radius_up_button,
            flip_bg_fg_button,
            reset_button,
            state_cycle_button,
            demo_clicks: 0,
            demo_radius,
            demo_colors_flipped: false,
            selected_visual_state: ButtonVisualState::Default,
            demo_background_overrides,
            demo_foreground_overrides,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.demo_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_button_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.radius_down_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_radius_down(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.radius_up_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_radius_up(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.flip_bg_fg_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_flip_bg_fg(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.reset_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_reset(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.state_cycle_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.decorated_button.handle_cycle_state(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_description(
            "Button",
            Some("Prototype migrated to canonical Button + Modifier pipeline."),
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .items_stretch()
                .justify_center()
                .gap(px(28.0))
                .child(self.render_demo_column(chrome))
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_button, cx);
        notify_entity(&self.danger_button, cx);
        notify_entity(&self.demo_button, cx);
        notify_entity(&self.state_preview, cx);

        notify_entity(&self.radius_down_button, cx);
        notify_entity(&self.radius_up_button, cx);
        notify_entity(&self.flip_bg_fg_button, cx);
        notify_entity(&self.reset_button, cx);
        notify_entity(&self.state_cycle_button, cx);
    }

    fn render_demo_column(&self, chrome: GalleryChrome) -> AnyElement {
        let theme_appearance = default_button_family_theme().resolve(
            ButtonFamilyRole::Text,
            ButtonSize::Md,
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
            .child(self.danger_button.clone())
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
                            .child("effective/source inspector (via modifiers)"),
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

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
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

        let theme = self.radix_theme.clone();
        let bg_overrides = self.demo_background_overrides.clone();
        let fg_overrides = self.demo_foreground_overrides.clone();

        let new_template = demo_button_template(&bg_overrides, &fg_overrides, radius, &theme);
        self.demo_button.update(cx, |button, cx| {
            button.set_template(new_template.clone(), cx);
        });

        self.state_preview.update(cx, |preview, cx| {
            preview.set_template(new_template, Some(radius), cx);
        });

        cx.notify();
    }
}

#[derive(Clone)]
struct ButtonStatePreview {
    radix_theme: Arc<RadixTheme>,
    template: Arc<dyn ButtonTemplate<()>>,
    radius: Option<f32>,
}

struct ButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ButtonStatePreview {
    fn new(radix_theme: Arc<RadixTheme>, template: Arc<dyn ButtonTemplate<()>>, radius: Option<f32>) -> Self {
        Self { radix_theme, template, radius }
    }

    fn set_template(&mut self, template: Arc<dyn ButtonTemplate<()>>, radius: Option<f32>, cx: &mut Context<Self>) {
        self.template = template;
        self.radius = radius;
        cx.notify();
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let samples = [
            ButtonStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            ButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        let radius = self.radius;

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
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_button_state_sample(&self.template, radius, sample, chrome.muted_text, window, cx)
                }),
            ))
    }
}

fn render_button_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    radius: Option<f32>,
    sample: ButtonStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("button-preview-{}", sample.id));
    let label = SharedString::from("Button");
    let content: gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> =
        Arc::new(move |_: &ButtonRenderModel<()>, _| div().child(label.clone()).into_any_element());
    let model = ButtonRenderModel {
        id,
        data: (),
        content,
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(radius),
        appearance: None,
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

fn danger_button_template(radix_theme: &RadixTheme) -> Arc<dyn ButtonTemplate<()>> {
    let mode = radix_theme.mode();
    let disabled_bg = radix_theme.chrome().panel_background;
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()).with_modifier(move |element, model| {
        let (danger_background, danger_hover_background, danger_pressed_background, danger_foreground): (
            Hsla,
            Hsla,
            Hsla,
            Hsla,
        ) = match mode {
            ThemeMode::Light => {
                (rgb(0xdc2626).into(), rgb(0xb91c1c).into(), rgb(0x991b1b).into(), rgb(0xffffff).into())
            }
            ThemeMode::Dark => (rgb(0xf87171).into(), rgb(0xfca5a5).into(), rgb(0xfecaca).into(), rgb(0x450a0a).into()),
        };

        let bg = if model.state.disabled {
            disabled_bg
        } else if model.state.pressed {
            danger_pressed_background
        } else if model.state.hovered {
            danger_hover_background
        } else {
            danger_background
        };

        element.bg(bg).text_color(danger_foreground).border_color(bg)
    }))
}

fn demo_button_template(
    bg_overrides: &ButtonStatefulOverride<Hsla>,
    fg_overrides: &ButtonStatefulOverride<Hsla>,
    radius: f32,
    _radix_theme: &RadixTheme,
) -> Arc<dyn ButtonTemplate<()>> {
    let bg_overrides = bg_overrides.clone();
    let fg_overrides = fg_overrides.clone();
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()).with_modifier(move |element, model| {
        let visual_state = ButtonVisualState::from(model.state);
        let mut element = element;

        if let Some(bg) = bg_overrides.resolve(visual_state) {
            element = element.bg(*bg).border_color(*bg);
        }

        if let Some(fg) = fg_overrides.resolve(visual_state) {
            element = element.text_color(*fg);
        }

        model.radius_override.set(Some(radius));

        element.rounded(px(radius))
    }))
}

fn visual_state_label(state: ButtonVisualState) -> &'static str {
    match state {
        ButtonVisualState::Default => "default",
        ButtonVisualState::Hovered => "hovered",
        ButtonVisualState::Pressed => "pressed",
        ButtonVisualState::Focused => "focused",
        ButtonVisualState::Disabled => "disabled",
    }
}

fn next_visual_state(state: ButtonVisualState) -> ButtonVisualState {
    match state {
        ButtonVisualState::Default => ButtonVisualState::Hovered,
        ButtonVisualState::Hovered => ButtonVisualState::Pressed,
        ButtonVisualState::Pressed => ButtonVisualState::Focused,
        ButtonVisualState::Focused => ButtonVisualState::Disabled,
        ButtonVisualState::Disabled => ButtonVisualState::Default,
    }
}

fn write_state_override<T>(overrides: &mut ButtonStatefulOverride<T>, state: ButtonVisualState, value: Option<T>) {
    match state {
        ButtonVisualState::Default => overrides.base = value,
        ButtonVisualState::Hovered => overrides.hovered = value,
        ButtonVisualState::Pressed => overrides.pressed = value,
        ButtonVisualState::Focused => overrides.focused = value,
        ButtonVisualState::Disabled => overrides.disabled = value,
    }
}

fn interaction_state_for_visual_state(state: ButtonVisualState) -> InteractionState {
    match state {
        ButtonVisualState::Default => {
            InteractionState { hovered: false, pressed: false, focused: false, disabled: false }
        }
        ButtonVisualState::Hovered => {
            InteractionState { hovered: true, pressed: false, focused: false, disabled: false }
        }
        ButtonVisualState::Pressed => {
            InteractionState { hovered: false, pressed: true, focused: false, disabled: false }
        }
        ButtonVisualState::Focused => {
            InteractionState { hovered: false, pressed: false, focused: true, disabled: false }
        }
        ButtonVisualState::Disabled => {
            InteractionState { hovered: false, pressed: false, focused: false, disabled: true }
        }
    }
}

fn resolve_color_with_source(
    overrides: &ButtonStatefulOverride<Hsla>,
    state: ButtonVisualState,
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
