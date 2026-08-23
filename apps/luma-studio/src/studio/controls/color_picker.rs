use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, EventEmitter, Hsla, IntoElement, Render, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::anchored_panel::{AnchoredPanel, AnchoredPanelDismissPolicy, AnchoredPanelPlacement};
use gpui_luma::controls::color::ColorSwatch;
use gpui_luma::controls::color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma::controls::color::color_slider::color_spec::Hsv;
use gpui_luma::controls::color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, sizing,
};
use gpui_luma::controls::color::composition::ColorCompositionSync;
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

#[derive(Clone, Debug)]
pub(crate) enum ColorPickerEvent {
    Change(Hsla),
}

pub(crate) struct ColorPickerPopover {
    look: Arc<ShadcnLook>,
    color: Hsla,
    hsv: Hsv,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    panel: Entity<AnchoredPanel>,
    transparent_trigger: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorPickerEvent> for ColorPickerPopover {}

impl ColorPickerPopover {
    pub(crate) fn new(look: Arc<ShadcnLook>, color: Hsla, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv::from_hsla_ext(color);
        let field = cx.new(|_| {
            ColorFieldState::saturation_value("colors-picker-field", hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .no_border()
                .rounded(px(0.0))
        });
        let hue_slider = ColorSliderBuilder::hue("colors-picker-hue", hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha("colors-picker-alpha", hsv.a, hsv)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let entity = cx.entity().clone();
        let panel = AnchoredPanel::new("colors-picker-popup")
            .content(move |_, _, cx| entity.update(cx, |picker, _| picker.render_popup()))
            .placement(AnchoredPanelPlacement::SmartStart)
            .dismiss_policy(AnchoredPanelDismissPolicy::CloseOnClickAway)
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&field, |picker, _, event: &ColorFieldEvent, cx| {
                let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                    return;
                };
                picker.hsv.a = picker.color.a;
                picker.hsv.s = hsv.s;
                picker.hsv.v = hsv.v;
                picker.sync_controls(cx, false);
                picker.emit_change(cx);
            }),
            cx.subscribe(&hue_slider, |picker, _, event: &SliderEvent, cx| {
                let Some(value) = primary_slider_value(event) else {
                    return;
                };
                picker.hsv.h = value;
                picker.sync_controls(cx, true);
                picker.emit_change(cx);
            }),
            cx.subscribe(&alpha_slider, |picker, _, event: &SliderEvent, cx| {
                let Some(value) = primary_slider_value(event) else {
                    return;
                };
                picker.hsv.a = value;
                picker.sync_controls(cx, true);
                picker.emit_change(cx);
            }),
        ];

        Self {
            look,
            color,
            hsv,
            sync: ColorCompositionSync::new(),
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            panel,
            transparent_trigger: false,
            _subscriptions: subscriptions,
        }
    }

    pub(crate) fn new_transparent(look: Arc<ShadcnLook>, color: Hsla, cx: &mut Context<Self>) -> Self {
        let mut picker = Self::new(look, color, cx);
        picker.transparent_trigger = true;
        picker
    }

    pub(crate) fn set_color(&mut self, color: Hsla, cx: &mut Context<Self>) {
        self.color = color;
        self.hsv = Hsv::from_hsla_ext(color);
        self.sync_controls(cx, true);
        cx.notify();
    }

    fn sync_controls(&self, cx: &mut Context<Self>, sync_field: bool) {
        if sync_field {
            self.field.update(cx, |field, cx| field.set_hsv(self.hsv, cx));
        }
        self.sync.sync_slider_value(&self.hue_slider, self.hsv.h, cx);
        self.sync.sync_color_slider(
            &self.alpha_slider,
            &self.alpha_domain,
            Arc::new(AlphaDelegate { spec: self.hsv }),
            self.alpha_domain.context(),
            self.hsv.a,
            cx,
        );
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.color = self.hsv.to_hsla_ext();
        cx.emit(ColorPickerEvent::Change(self.color));
        cx.notify();
    }

    fn render_popup(&self) -> AnyElement {
        div()
            .w(px(260.0))
            .p(px(12.0))
            .gap(px(10.0))
            .flex()
            .flex_col()
            .bg(self.look.chrome().content_background)
            .border_1()
            .border_color(self.look.chrome().border)
            .rounded(px(6.0))
            .child(div().w(px(236.0)).h(px(180.0)).child(self.field.clone()))
            .child(div().w(px(236.0)).child(self.hue_slider.clone()))
            .child(div().w(px(236.0)).child(self.alpha_slider.clone()))
            .into_any_element()
    }
}

impl Render for ColorPickerPopover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let panel = self.panel.clone();
        let color = self.color;
        let mut swatch =
            div().id("colors-picker-swatch").size(px(30.0)).rounded(px(4.0)).on_click(move |_, window, cx| {
                panel.update(cx, |panel, cx| panel.toggle_guarded_from(None, cx));
                window.prevent_default();
            });
        if !self.transparent_trigger {
            swatch = swatch.child(ColorSwatch::new(color).height(px(30.0)).rounded(px(4.0)).checkerboard(true));
        }
        let panel_for_bounds = self.panel.clone();
        swatch = swatch.on_prepaint(move |bounds, _, cx| {
            panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(bounds, cx));
        });
        div().size(px(30.0)).relative().child(swatch).child(self.panel.clone())
    }
}
