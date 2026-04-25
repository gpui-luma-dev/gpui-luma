use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, Subscription, div, prelude::*, px};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::prototypes::proto_button::{
    ProtoButton, ProtoButtonEvent, ProtoButtonTemplate, ProtoButtonTemplateParams, ThemedProtoButtonTemplate,
    proto_button_template_usage,
};
use gpui_luma::theme::{ButtonVariant, ControlSize, default_button_family_theme};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::{GalleryChrome, GalleryThemePack};

use super::super::super::shared::{gallery_pane_with_description, notify_entity};
use super::param_panel::render_proto_button_param_panel;

const EMERGENCY_DISABLED_OPACITY: f32 = 0.72;
const EMERGENCY_RADIUS: f32 = 0.0;
const EMERGENCY_BACKGROUND: Hsla = Hsla { h: 0.0, s: 0.83, l: 0.48, a: 1.0 };
const EMERGENCY_FOREGROUND: Hsla = Hsla { h: 0.0, s: 0.0, l: 0.98, a: 1.0 };

const MIN_RADIUS: f32 = 0.0;
const MAX_RADIUS: f32 = 24.0;
const RADIUS_STEP: f32 = 2.0;

#[derive(Clone)]
pub(in crate::gallery) struct ProtoButtonPane {
    default_button: Entity<ProtoButton>,
    emergency_button: Entity<ProtoButton>,

    radius_down_button: Entity<Button>,
    radius_up_button: Entity<Button>,
    flip_bg_fg_button: Entity<Button>,
    reset_button: Entity<Button>,

    emergency_clicks: usize,
    emergency_radius: f32,
    emergency_colors_flipped: bool,
}

impl ProtoButtonPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let default_button = ProtoButton::new("proto-button-default").label("Default ProtoButton").spawn(cx);
        let emergency_button = ProtoButton::new("proto-button-emergency")
            .label("Emergency Action")
            .template(emergency_proto_button_template())
            .spawn(cx);

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
            .kind(ButtonKind::Destructive)
            .template(theme.button_template())
            .spawn(cx);

        Self {
            default_button,
            emergency_button,
            radius_down_button,
            radius_up_button,
            flip_bg_fg_button,
            reset_button,
            emergency_clicks: 0,
            emergency_radius: EMERGENCY_RADIUS,
            emergency_colors_flipped: false,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.emergency_button, |app, _, event: &ProtoButtonEvent, cx| {
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
                .child(self.render_demo_column(chrome))
                .child(render_proto_button_param_panel(usage, chrome, theme))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.default_button, cx);
        notify_entity(&self.emergency_button, cx);

        notify_entity(&self.radius_down_button, cx);
        notify_entity(&self.radius_up_button, cx);
        notify_entity(&self.flip_bg_fg_button, cx);
        notify_entity(&self.reset_button, cx);
    }

    fn render_demo_column(&self, chrome: GalleryChrome) -> AnyElement {
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
            .child(self.emergency_button.clone())
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
                            .child(format!("radius: {:.1}px", self.emergency_radius)),
                    )
                    .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(chrome.muted_text).child(
                        format!("colors flipped: {}", if self.emergency_colors_flipped { "on" } else { "off" }),
                    )),
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
                            .child(self.reset_button.clone()),
                    ),
            )
            .into_any_element()
    }

    fn handle_proto_button_event(&mut self, event: &ProtoButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ProtoButtonEvent::Click) {
            self.emergency_clicks += 1;
            let label = format!("Emergency Action ({})", self.emergency_clicks);

            self.emergency_button.update(cx, |button, cx| {
                button.set_label(label, cx);
            });
        }
    }

    fn handle_radius_down(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.emergency_radius = (self.emergency_radius - RADIUS_STEP).clamp(MIN_RADIUS, MAX_RADIUS);
            self.apply_template_params(cx);
        }
    }

    fn handle_radius_up(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.emergency_radius = (self.emergency_radius + RADIUS_STEP).clamp(MIN_RADIUS, MAX_RADIUS);
            self.apply_template_params(cx);
        }
    }

    fn handle_flip_bg_fg(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.emergency_colors_flipped = !self.emergency_colors_flipped;
            self.apply_template_params(cx);
        }
    }

    fn handle_reset(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        if matches!(event, ButtonEvent::Click) {
            self.emergency_radius = EMERGENCY_RADIUS;
            self.emergency_colors_flipped = false;

            self.apply_template_params(cx);
        }
    }

    fn apply_template_params(&mut self, cx: &mut Context<GalleryApp>) {
        let radius = self.emergency_radius;
        let foreground = if self.emergency_colors_flipped {
            Some(EMERGENCY_BACKGROUND)
        } else {
            None
        };
        let background = if self.emergency_colors_flipped {
            Some(EMERGENCY_FOREGROUND)
        } else {
            Some(EMERGENCY_BACKGROUND)
        };

        self.emergency_button.update(cx, |button, cx| {
            let mut params = button.template_params().unwrap_or_default();
            params.variant = ButtonVariant::Destructive;
            params.size = ControlSize::Md;
            params.disabled_opacity = EMERGENCY_DISABLED_OPACITY;
            params.radius = Some(radius);
            params.background = background;
            params.foreground = foreground;
            let _ = button.set_template_params(params, cx);
        });

        cx.notify();
    }
}

fn emergency_proto_button_template() -> Arc<dyn ProtoButtonTemplate> {
    let params = ProtoButtonTemplateParams {
        variant: ButtonVariant::Destructive,
        size: ControlSize::Md,
        disabled_opacity: EMERGENCY_DISABLED_OPACITY,
        radius: Some(EMERGENCY_RADIUS),
        background: Some(EMERGENCY_BACKGROUND),
        ..Default::default()
    };

    Arc::new(ThemedProtoButtonTemplate::new(default_button_family_theme()).with_params(params))
}
