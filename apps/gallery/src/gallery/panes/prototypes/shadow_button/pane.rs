use std::sync::Arc;

use gpui::{
    AnyElement, App, BoxShadow, Context, Div, Entity, Hsla, IntoElement, Render, SharedString, Stateful, Subscription,
    Window, div, point, prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{Button, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, gallery_pane_with_description, notify_entity};
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
    look: Arc<ShadcnLook>,
}

impl ButtonTemplate<()> for PrototypeShadowButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<()>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let button = self.inner.render(model, window, cx);
        let state = ButtonVisualState::from(model.state);
        let Some(spec) = resolve_shadow_spec(self.controls.base_spec(&self.look), state) else {
            return button;
        };
        let appearance = self.look.resolve_primary_button(model.role, model.size, model.state);
        let radius = if model.round {
            appearance.height / 2.0
        } else {
            appearance.radius
        };
        let extent = shadow_projection_extent(spec);

        div()
            .id(format!("{}-shadow-root", model.id))
            .relative()
            .p(px(extent))
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

    offset_x_slider: Slider,
    offset_y_slider: Slider,
    blur_slider: Slider,
    spread_slider: Slider,
    opacity_slider: Slider,

    demo_clicks: usize,
    shadow: PrototypeShadowControls,
}

impl ButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let shadow = PrototypeShadowControls::default();
        let template = prototype_shadow_button_template(look.clone(), shadow);

        let demo_button =
            Button::new("shadow-demo-button").label("Floating Action").template(template.clone()).spawn(cx);

        let state_preview = cx.new(|_| ButtonStatePreview::new(look.clone(), template.clone()));
        let reset_button = look.secondary_button("shadow-reset").label("Reset Shadow").spawn(cx);

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
            offset_x_slider,
            offset_y_slider,
            blur_slider,
            spread_slider,
            opacity_slider,
            demo_clicks: 0,
            shadow,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.demo_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.shadow_button.handle_button_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.reset_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.shadow_button.handle_reset(event, cx);
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
        gallery_pane_with_description(
            "Shadow Button",
            Some("Gallery-only prototype for a CSS-style box shadow wrapped around the existing button template."),
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_wrap()
                .items_start()
                .justify_center()
                .gap(px(28.0))
                .child(self.render_demo_column(look))
                .child(self.render_controls_column(look))
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.demo_button, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.reset_button, cx);
        notify_entity(&self.offset_x_slider, cx);
        notify_entity(&self.offset_y_slider, cx);
        notify_entity(&self.blur_slider, cx);
        notify_entity(&self.spread_slider, cx);
        notify_entity(&self.opacity_slider, cx);
    }

    fn render_demo_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let base_spec = self.shadow.base_spec(look);
        let pressed_spec = resolve_shadow_spec(base_spec, ButtonVisualState::Pressed).unwrap_or(base_spec);

        div()
            .min_w(px(280.0))
            .max_w(px(420.0))
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.0))
            .child(self.demo_button.clone())
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .text_color(chrome.muted_text)
                    .child(format!("Clicks: {}", self.demo_clicks)),
            )
            .child(render_shadow_summary_card(chrome, base_spec, pressed_spec))
            .child(self.state_preview.clone())
            .into_any_element()
    }

    fn render_controls_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let base_spec = self.shadow.base_spec(look);

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
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .gap(px(12.0))
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
                            .child(self.reset_button.clone()),
                    )
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
                    )),
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
            self.sync_sliders(cx);
            self.apply_template(cx);
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

    fn apply_template(&mut self, cx: &mut Context<GalleryApp>) {
        let template = prototype_shadow_button_template(self.look.clone(), self.shadow);
        self.demo_button.update(cx, |button, cx| {
            button.set_template(template.clone(), cx);
        });
        self.state_preview.update(cx, |preview, cx| {
            preview.set_template(template.clone(), cx);
        });
        cx.notify();
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
) -> Arc<dyn ButtonTemplate<()>> {
    Arc::new(PrototypeShadowButtonTemplate { inner: look.button_template(ShadcnButtonStyle::Primary), controls, look })
}

fn render_shadow(id: &SharedString, spec: PrototypeShadowSpec, radius: f32) -> Stateful<Div> {
    let shadow_color = spec.color;
    let extent = shadow_projection_extent(spec);

    div()
        .id(format!("{}-shadow", id))
        .absolute()
        .top(px(extent))
        .left(px(extent))
        .right(px(extent))
        .bottom(px(extent))
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
        .w(px(420.0))
        .max_w_full()
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

fn shadow_layers(spec: PrototypeShadowSpec) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: spec.color.opacity(spec.opacity.clamp(0.0, 1.0)),
        offset: point(px(spec.offset_x), px(spec.offset_y)),
        blur_radius: px(spec.blur_radius.max(0.0)),
        spread_radius: px(spec.spread_radius),
    }]
}

fn shadow_projection_extent(spec: PrototypeShadowSpec) -> f32 {
    let blur = spec.blur_radius.max(0.0);
    let spread = spec.spread_radius.max(0.0);
    let offset_x = spec.offset_x.abs();
    let offset_y = spec.offset_y.abs();
    (blur + spread + offset_x.max(offset_y)).ceil().max(1.0)
}
