use std::sync::Arc;

use gpui::{
    AnyElement, App, BoxShadow, Context, Div, Entity, FontWeight, Hsla, IntoElement, Render, SharedString, Stateful,
    Subscription, Window, div, point, prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent, Validator};
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, notify_entity};
use crate::gallery::theme::GalleryChrome;

const DEFAULT_OFFSET_X: f32 = 0.0;
const DEFAULT_OFFSET_Y: f32 = 5.0;
const DEFAULT_BLUR: f32 = 5.0;
const DEFAULT_SPREAD: f32 = 0.0;
const DEFAULT_OPACITY: f32 = 0.35;

const PRESSED_OFFSET_FACTOR: f32 = 0.55;
const PRESSED_OPACITY_FACTOR: f32 = 0.82;
const HOVER_OFFSET_BONUS: f32 = 1.0;
const HOVER_OPACITY_BONUS: f32 = 0.03;

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

#[derive(Clone, Copy, Debug)]
pub struct PrototypeShadowSpec {
    pub color: Hsla,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug)]
struct PrototypeShadowControls {
    offset_x: f32,
    offset_y: f32,
    blur_radius: f32,
    spread_radius: f32,
    opacity: f32,
}

impl Default for PrototypeShadowControls {
    fn default() -> Self {
        Self {
            offset_x: DEFAULT_OFFSET_X,
            offset_y: DEFAULT_OFFSET_Y,
            blur_radius: DEFAULT_BLUR,
            spread_radius: DEFAULT_SPREAD,
            opacity: DEFAULT_OPACITY,
        }
    }
}

impl PrototypeShadowControls {
    fn base_spec(self, look: &ShadcnLook) -> PrototypeShadowSpec {
        PrototypeShadowSpec {
            color: resolved_shadow_color(look),
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            blur_radius: self.blur_radius,
            spread_radius: self.spread_radius,
            opacity: self.opacity,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ShadowSliderField {
    OffsetX,
    OffsetY,
    Blur,
    Spread,
    Opacity,
}

#[derive(Clone)]
struct PrototypeShadowButtonTemplate {
    inner: Arc<dyn ButtonTemplate<()>>,
    controls: PrototypeShadowControls,
    color_override: Option<Hsla>,
    look: Arc<ShadcnLook>,
}

impl ButtonTemplate<()> for PrototypeShadowButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<()>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let button = self.inner.render(model, window, cx);
        let state = ButtonVisualState::from(model.state);
        let mut base_spec = self.controls.base_spec(&self.look);
        if let Some(color) = self.color_override {
            base_spec.color = color;
        }
        let Some(spec) = resolve_shadow_spec(base_spec, state) else {
            return button;
        };
        let appearance = self.look.resolve_primary_button(model.role, model.size, model.state);
        let radius = if model.round {
            appearance.height / 2.0
        } else {
            appearance.radius
        };
        let insets = shadow_projection_insets(spec);

        div()
            .id(format!("{}-shadow-root", model.id))
            .relative()
            .pt(px(insets.top))
            .pr(px(insets.right))
            .pb(px(insets.bottom))
            .pl(px(insets.left))
            .child(render_shadow(&model.id, spec, radius))
            .child(button)
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct ButtonPane {
    look: Arc<ShadcnLook>,
    demo_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    reset_button: Entity<Button>,
    bounds_toggle: Entity<Button<bool>>,
    color_field: TextField,

    offset_x_slider: Slider,
    offset_y_slider: Slider,
    blur_slider: Slider,
    spread_slider: Slider,
    opacity_slider: Slider,

    demo_clicks: usize,
    shadow: PrototypeShadowControls,
    color_input: SharedString,
    color_override: Option<Hsla>,
    color_input_valid: bool,
    show_bounds: bool,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let shadow = PrototypeShadowControls::default();
        let template = prototype_shadow_button_template(look.clone(), shadow, None);
        let default_shadow_color = resolved_shadow_color(&look);

        let demo_button =
            Button::new("shadow-demo-button").label("Floating Action").template(template.clone()).spawn(cx);

        let state_preview = cx.new(|_| ButtonStatePreview::new(look.clone(), template.clone()));
        let reset_button = look.secondary_button("shadow-reset").label("Reset Shadow").spawn(cx);
        let bounds_toggle = look
            .secondary_toggle("shadow-show-bounds")
            .with_data(true)
            .content(|_, _| div().child("Show Bounds").into_any_element())
            .spawn(cx);
        let color_field = look
            .textfield("shadow-color")
            .placeholder(default_shadow_color_placeholder(default_shadow_color))
            .full_width(true)
            .clean_on_escape(true)
            .validator(shadow_color_validator())
            .spawn(cx);

        let offset_x_slider = look.slider("shadow-offset-x").range(-20..20).step(1).value(shadow.offset_x).spawn(cx);
        let offset_y_slider = look.slider("shadow-offset-y").range(-8..24).step(1).value(shadow.offset_y).spawn(cx);
        let blur_slider = look.slider("shadow-blur").range(0..32).step(1).value(shadow.blur_radius).spawn(cx);
        let spread_slider = look.slider("shadow-spread").range(-12..20).step(1).value(shadow.spread_radius).spawn(cx);
        let opacity_slider =
            look.slider("shadow-opacity").range(0..100).step(1).value(shadow.opacity * 100.0).spawn(cx);

        Self {
            look,
            demo_button,
            state_preview,
            reset_button,
            bounds_toggle,
            color_field,
            offset_x_slider,
            offset_y_slider,
            blur_slider,
            spread_slider,
            opacity_slider,
            demo_clicks: 0,
            shadow,
            color_input: SharedString::default(),
            color_override: None,
            color_input_valid: true,
            show_bounds: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.demo_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.shadow_button.handle_button_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.reset_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.shadow_button.handle_reset(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.bounds_toggle, |app, _, event: &ButtonEvent, cx| {
            app.panes.shadow_button.handle_bounds_toggle(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.color_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.shadow_button.handle_color_field_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.offset_x_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.shadow_button.handle_slider_event(ShadowSliderField::OffsetX, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.offset_y_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.shadow_button.handle_slider_event(ShadowSliderField::OffsetY, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.blur_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.shadow_button.handle_slider_event(ShadowSliderField::Blur, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.spread_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.shadow_button.handle_slider_event(ShadowSliderField::Spread, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.opacity_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.shadow_button.handle_slider_event(ShadowSliderField::Opacity, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.content_background)
            .p(px(28.0))
            .child(
                div()
                    .id("shadow-button-content")
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap(px(24.0))
                    .overflow_y_scroll()
                    .child(render_pane_header(
                        "Shadow Button",
                        Some("Gallery-only prototype for a CSS-style box shadow wrapped around the existing button template."),
                        chrome,
                    ))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .w_full()
                                    .max_w(px(980.0))
                                    .flex()
                                    .flex_wrap()
                                    .items_start()
                                    .justify_center()
                                    .gap(px(32.0))
                                    .child(self.render_demo_column(look))
                                    .child(self.render_controls_column(look)),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.demo_button, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.reset_button, cx);
        notify_entity(&self.bounds_toggle, cx);
        self.sync_color_field(cx);
        notify_entity(&self.color_field, cx);
        notify_entity(&self.offset_x_slider, cx);
        notify_entity(&self.offset_y_slider, cx);
        notify_entity(&self.blur_slider, cx);
        notify_entity(&self.spread_slider, cx);
        notify_entity(&self.opacity_slider, cx);
    }

    fn render_demo_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let foreground = look.token_color("foreground").unwrap_or(chrome.title_text);
        let base_spec = self.base_spec(look);
        let pressed_spec = resolve_shadow_spec(base_spec, ButtonVisualState::Pressed).unwrap_or(base_spec);

        div()
            .w(px(420.0))
            .max_w_full()
            .flex()
            .flex_col()
            .items_stretch()
            .gap(px(18.0))
            .child(
                div()
                    .w_full()
                    .min_h(px(184.0))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .rounded(px(10.0))
                    .p(px(24.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .when(self.show_bounds, |container| container.border_1().border_color(foreground))
                            .child(self.demo_button.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(chrome.muted_text)
                            .child(format!("Clicks: {}", self.demo_clicks)),
                    ),
            )
            .child(render_shadow_summary_card(chrome, base_spec, pressed_spec))
            .child(self.state_preview.clone())
            .into_any_element()
    }

    fn render_controls_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let base_spec = self.base_spec(look);
        let color_help_text = self.color_help_text(look);
        let color_help_color = if self.color_input_valid {
            chrome.muted_text
        } else {
            look.token_color("destructive").unwrap_or(chrome.body_text)
        };

        div()
            .w(px(420.0))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .rounded(px(10.0))
                    .p(px(14.0))
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
                                    .text_size(px(13.0))
                                    .line_height(px(18.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(chrome.title_text)
                                    .child("Shadow Tuning"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .line_height(px(16.0))
                                    .text_color(chrome.muted_text)
                                    .child("CSS-style shadow prototype: x, y, blur, spread, and a theme-aware neutral shadow tone."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .size(px(20.0))
                                            .rounded(px(999.0))
                                            .bg(base_spec.color)
                                            .border_1()
                                            .border_color(base_spec.color.opacity(0.42)),
                                    )
                                    .child(
                                        div()
                                            .font_family("Monaco")
                                            .text_size(px(11.0))
                                            .line_height(px(15.0))
                                            .text_color(chrome.muted_text)
                                            .child(format_compact_hsla(base_spec.color)),
                                    ),
                            )
                            .child(self.reset_button.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .child(self.bounds_toggle.clone()),
                    )
                    .child(render_text_field_row(
                        "Color",
                        "RGB, rgb(...), or #hex",
                        self.color_field.clone(),
                        color_help_text,
                        color_help_color,
                        chrome,
                    ))
                    .child(render_slider_row(
                        "X offset",
                        format!("{:.0}px", self.shadow.offset_x),
                        self.offset_x_slider.clone(),
                        chrome,
                    ))
                    .child(render_slider_row(
                        "Y offset",
                        format!("{:.0}px", self.shadow.offset_y),
                        self.offset_y_slider.clone(),
                        chrome,
                    ))
                    .child(render_slider_row(
                        "Blur",
                        format!("{:.0}px", self.shadow.blur_radius),
                        self.blur_slider.clone(),
                        chrome,
                    ))
                    .child(render_slider_row(
                        "Spread",
                        format!("{:.0}px", self.shadow.spread_radius),
                        self.spread_slider.clone(),
                        chrome,
                    ))
                    .child(render_slider_row(
                        "Opacity",
                        format!("{:.0}%", self.shadow.opacity * 100.0),
                        self.opacity_slider.clone(),
                        chrome,
                    ))
                    .child(render_css_shadow_value(base_spec, chrome)),
            )
            .into_any_element()
    }

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.demo_clicks += 1;
            let label = format!("Floating Action ({})", self.demo_clicks);
            self.demo_button.update(cx, |button, cx| {
                button.set_label(label, cx);
            });
        }
    }

    fn handle_reset(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.shadow = PrototypeShadowControls::default();
            self.color_input = SharedString::default();
            self.color_override = None;
            self.color_input_valid = true;
            self.sync_sliders(cx);
            self.sync_color_field(cx);
            self.apply_template(cx);
        }
    }

    fn handle_bounds_toggle(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.bounds_toggle.update(cx, |button, cx| {
                let new_selected = !*button.data();
                button.set_data(new_selected, cx);
                self.show_bounds = new_selected;
            });
            cx.notify();
        }
    }

    fn handle_color_field_event(&mut self, event: &TextFieldEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => {
                self.color_input = value.clone().into();
                let trimmed = value.trim();

                if trimmed.is_empty() {
                    self.color_override = None;
                    self.color_input_valid = true;
                    self.apply_template(cx);
                } else if let Some(color) = parse_shadow_color_input(trimmed) {
                    self.color_override = Some(color);
                    self.color_input_valid = true;
                    self.apply_template(cx);
                } else {
                    self.color_input_valid = false;
                    cx.notify();
                }
            }
            TextFieldEvent::Focus | TextFieldEvent::Blur => {}
        }
    }

    fn handle_slider_event(&mut self, field: ShadowSliderField, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        let SliderEvent::Change { value } = event;
        match field {
            ShadowSliderField::OffsetX => self.shadow.offset_x = *value,
            ShadowSliderField::OffsetY => self.shadow.offset_y = *value,
            ShadowSliderField::Blur => self.shadow.blur_radius = *value,
            ShadowSliderField::Spread => self.shadow.spread_radius = *value,
            ShadowSliderField::Opacity => self.shadow.opacity = (*value / 100.0).clamp(0.0, 1.0),
        }

        self.apply_template(cx);
    }

    fn sync_sliders(&self, cx: &mut Context<GalleryApp>) {
        self.offset_x_slider.update(cx, |slider, cx| slider.set_value(self.shadow.offset_x, cx));
        self.offset_y_slider.update(cx, |slider, cx| slider.set_value(self.shadow.offset_y, cx));
        self.blur_slider.update(cx, |slider, cx| slider.set_value(self.shadow.blur_radius, cx));
        self.spread_slider.update(cx, |slider, cx| slider.set_value(self.shadow.spread_radius, cx));
        self.opacity_slider.update(cx, |slider, cx| slider.set_value(self.shadow.opacity * 100.0, cx));
    }

    fn sync_color_field(&self, cx: &mut Context<GalleryApp>) {
        let placeholder = default_shadow_color_placeholder(resolved_shadow_color(&self.look));
        let value = self.color_input.clone();

        self.color_field.update(cx, |text_field, cx| {
            text_field.set_placeholder(placeholder, cx);
            text_field.set_value(value.as_ref(), cx);
        });
    }

    fn apply_template(&mut self, cx: &mut Context<GalleryApp>) {
        let template = prototype_shadow_button_template(self.look.clone(), self.shadow, self.color_override);
        self.demo_button.update(cx, |button, cx| {
            button.set_template(template.clone(), cx);
        });
        self.state_preview.update(cx, |preview, cx| {
            preview.set_template(template.clone(), cx);
        });
        cx.notify();
    }

    fn base_spec(&self, look: &ShadcnLook) -> PrototypeShadowSpec {
        let mut spec = self.shadow.base_spec(look);
        if let Some(color) = self.color_override {
            spec.color = color;
        }
        spec
    }

    fn color_help_text(&self, look: &ShadcnLook) -> String {
        if self.color_input.is_empty() {
            format!("Theme default: {}", format_rgb_triplet(resolved_shadow_color(look)))
        } else if let Some(color) = self.color_override {
            format!("Using: {}", format_rgb_triplet(color))
        } else {
            String::from("Invalid color. Use 255, 255, 255, rgb(255,255,255), or #ffffff.")
        }
    }
}

#[derive(Clone)]
struct ButtonStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ButtonTemplate<()>>,
}

struct ButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ButtonStatePreview {
    fn new(look: Arc<ShadcnLook>, template: Arc<dyn ButtonTemplate<()>>) -> Self {
        Self { look, template }
    }

    fn set_template(&mut self, template: Arc<dyn ButtonTemplate<()>>, cx: &mut Context<Self>) {
        self.template = template;
        cx.notify();
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
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
                label: "Pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        div()
            .w_full()
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
                div().w_full().flex().flex_wrap().items_start().justify_start().gap(px(16.0)).children(
                    samples.into_iter().map(|sample| {
                        render_button_state_sample(&self.template, sample, chrome.muted_text, window, cx)
                    }),
                ),
            )
    }
}

fn prototype_shadow_button_template(
    look: Arc<ShadcnLook>,
    controls: PrototypeShadowControls,
    color_override: Option<Hsla>,
) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(PrototypeShadowButtonTemplate {
        inner: look.button_template(ShadcnButtonStyle::Primary),
        controls,
        color_override,
        look,
    })
}

fn render_shadow(id: &SharedString, spec: PrototypeShadowSpec, radius: f32) -> Stateful<Div> {
    let shadow_color = spec.color;
    let insets = shadow_projection_insets(spec);

    div()
        .id(format!("{}-shadow", id))
        .absolute()
        .top(px(insets.top))
        .left(px(insets.left))
        .right(px(insets.right))
        .bottom(px(insets.bottom))
        .rounded(px(radius))
        .bg(shadow_color.opacity(0.0))
        .shadow(shadow_layers(spec))
}

fn render_shadow_summary_card(
    chrome: GalleryChrome,
    base_spec: PrototypeShadowSpec,
    pressed_spec: PrototypeShadowSpec,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(10.0))
        .bg(chrome.panel_background)
        .p(px(12.0))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_family("Monaco")
                .text_color(chrome.muted_text)
                .child("effective shadow spec"),
        )
        .child(render_summary_row("color", format_compact_hsla(base_spec.color), chrome))
        .child(render_summary_row("x", format!("{:.0}px", base_spec.offset_x), chrome))
        .child(render_summary_row("y", format!("{:.0}px", base_spec.offset_y), chrome))
        .child(render_summary_row("blur", format!("{:.0}px", base_spec.blur_radius), chrome))
        .child(render_summary_row("spread", format!("{:.0}px", base_spec.spread_radius), chrome))
        .child(render_summary_row("pressed y", format!("{:.0}px", pressed_spec.offset_y), chrome))
        .child(render_summary_row("pressed opacity", format!("{:.0}%", pressed_spec.opacity * 100.0), chrome))
        .into_any_element()
}

fn render_pane_header(title: &'static str, description: Option<&'static str>, chrome: GalleryChrome) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(20.0))
                .line_height(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(title),
        )
        .when_some(description, |header, description| {
            header.child(
                div()
                    .max_w(px(760.0))
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(chrome.muted_text)
                    .child(description),
            )
        })
        .into_any_element()
}

fn render_summary_row(label: &'static str, value: String, chrome: GalleryChrome) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(chrome.body_text).child(label))
        .child(
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.muted_text)
                .child(value),
        )
        .into_any_element()
}

fn render_slider_row(label: &'static str, value: String, slider: Slider, chrome: GalleryChrome) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
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
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(label),
                )
                .child(
                    div()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(value),
                ),
        )
        .child(slider)
        .into_any_element()
}

fn render_text_field_row(
    label: &'static str,
    value_hint: &'static str,
    text_field: TextField,
    assistive_text: String,
    assistive_color: Hsla,
    chrome: GalleryChrome,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
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
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(label),
                )
                .child(
                    div()
                        .font_family("Monaco")
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(value_hint),
                ),
        )
        .child(text_field)
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(assistive_color).child(assistive_text))
        .into_any_element()
}

fn render_css_shadow_value(spec: PrototypeShadowSpec, chrome: GalleryChrome) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .child("CSS box-shadow"),
        )
        .child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.content_background)
                .rounded(px(8.0))
                .p(px(10.0))
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(16.0))
                .text_color(chrome.body_text)
                .child(format_css_box_shadow(spec)),
        )
        .into_any_element()
}

fn render_button_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    sample: ButtonStateSample,
    label_color: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("shadow-preview-{}", sample.id));
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
        radius_override: std::cell::Cell::new(None),
        appearance: None,
    };

    div()
        .w(px(120.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn resolve_shadow_spec(base: PrototypeShadowSpec, state: ButtonVisualState) -> Option<PrototypeShadowSpec> {
    match state {
        ButtonVisualState::Disabled => None,
        ButtonVisualState::Hovered => Some(PrototypeShadowSpec {
            offset_y: base.offset_y + HOVER_OFFSET_BONUS,
            opacity: (base.opacity + HOVER_OPACITY_BONUS).clamp(0.0, 1.0),
            ..base
        }),
        ButtonVisualState::Pressed => Some(PrototypeShadowSpec {
            offset_y: (base.offset_y * PRESSED_OFFSET_FACTOR).max(1.0),
            opacity: (base.opacity * PRESSED_OPACITY_FACTOR).clamp(0.0, 1.0),
            ..base
        }),
        ButtonVisualState::Default | ButtonVisualState::Focused => Some(base),
    }
}

fn resolved_shadow_color(look: &ShadcnLook) -> Hsla {
    let chrome = look.chrome();
    let background = chrome.panel_background;

    if background.l <= 0.35 {
        Hsla { h: 0.0, s: 0.0, l: 0.82, a: 1.0 }
    } else {
        Hsla { h: 0.0, s: 0.0, l: 0.08, a: 1.0 }
    }
}

fn default_shadow_color_placeholder(color: Hsla) -> String {
    format!("{}, {}, {}", rgb_triplet(color).0, rgb_triplet(color).1, rgb_triplet(color).2)
}

fn format_css_box_shadow(spec: PrototypeShadowSpec) -> String {
    format!(
        "box-shadow: {} {}px {}px {}px {}px;",
        format_css_rgba(spec.color, spec.opacity),
        rounded_px(spec.offset_x),
        rounded_px(spec.offset_y),
        rounded_px(spec.blur_radius.max(0.0)),
        rounded_px(spec.spread_radius),
    )
}

fn format_css_rgba(color: Hsla, opacity: f32) -> String {
    let (r, g, b) = rgb_triplet(color);
    format!("rgba({r}, {g}, {b}, {})", compact_alpha(opacity.clamp(0.0, 1.0)))
}

fn format_rgb_triplet(color: Hsla) -> String {
    let (r, g, b) = rgb_triplet(color);
    format!("{r}, {g}, {b}")
}

fn rounded_px(value: f32) -> i32 {
    value.round() as i32
}

fn rgb_triplet(color: Hsla) -> (u8, u8, u8) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 0.0);
    let g = hue_to_channel(p, q, h + 2.0);
    let b = hue_to_channel(p, q, h + 4.0);

    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn shadow_color_validator() -> Validator {
    Arc::new(|value: &str| value.trim().is_empty() || parse_shadow_color_input(value).is_some())
}

fn parse_shadow_color_input(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }

    let component_source = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("rgb(") && trimmed.ends_with(')')
    {
        &trimmed[4..trimmed.len() - 1]
    } else {
        trimmed
    };

    let components: Vec<&str> = component_source
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect();

    if components.len() != 3 {
        return None;
    }

    let r = components[0].parse::<u8>().ok()?;
    let g = components[1].parse::<u8>().ok()?;
    let b = components[2].parse::<u8>().ok()?;
    Some(gpui::rgb(rgb_hex(r, g, b)).into())
}

fn parse_hex_color(raw: &str) -> Option<Hsla> {
    let hex = raw.trim().trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        _ => None,
    }
}

fn rgb_hex(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

fn compact_alpha(value: f32) -> String {
    let formatted = format!("{value:.3}");
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn shadow_layers(spec: PrototypeShadowSpec) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: spec.color.opacity(spec.opacity.clamp(0.0, 1.0)),
        offset: point(px(spec.offset_x), px(spec.offset_y)),
        blur_radius: px(spec.blur_radius.max(0.0)),
        spread_radius: px(spec.spread_radius),
    }]
}

#[derive(Clone, Copy, Debug)]
struct ShadowProjectionInsets {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

fn shadow_projection_insets(spec: PrototypeShadowSpec) -> ShadowProjectionInsets {
    let reach = (spec.blur_radius.max(0.0) + spec.spread_radius).max(0.0);

    ShadowProjectionInsets {
        top: (reach - spec.offset_y).ceil().max(0.0),
        right: (reach + spec.offset_x).ceil().max(0.0),
        bottom: (reach + spec.offset_y).ceil().max(0.0),
        left: (reach - spec.offset_x).ceil().max(0.0),
    }
}

fn hue_to_channel(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 6.0;
    }
    if t >= 6.0 {
        t -= 6.0;
    }
    if t < 1.0 {
        p + (q - p) * t
    } else if t < 3.0 {
        q
    } else if t < 4.0 {
        p + (q - p) * (4.0 - t)
    } else {
        p
    }
}
