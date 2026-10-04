use crate::studio::color_format::format_color_readout;
use gpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use std::sync::Arc;

use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, EventEmitter, IntoElement, Render, Subscription, Window, div,
    prelude::*, px, size,
};
use gpui_luma_color::{ColorSwatchButtonTemplate, ColorSwatchData};
use gpui_luma::controls::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma_color::color_slider::color_spec::{Hsv, ColorSpecification};
use gpui_luma_color::color_slider::{
    AlphaDelegate, ColorSliderBuilder, ColorSliderDomainRenderer, primary_slider_value, sizing,
};
use gpui_luma_color::composition::ColorCompositionSync;
use gpui_luma::controls::popover_button::{PopoverButton, PopoverDismissPolicy, PopoverPlacement};
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnSize};
use gpui_luma_look_shadcn as shadcn;

#[derive(Clone, Debug)]
pub(crate) enum ColorPickerEvent {
    Change(ColorValue),
}

pub(crate) type ColorPickerChangeHandler = Arc<dyn Fn(ColorValue, &mut App) + 'static>;
pub(crate) type ColorPickerContent =
    Arc<dyn Fn(ColorValue, ColorPickerChangeHandler, &mut Window, &mut App) -> AnyElement + 'static>;

pub(crate) struct ColorPickerPopover {
    color: ColorValue,
    hsv: Hsv,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    popover: Entity<PopoverButton>,
    button: Entity<Button<ColorSwatchData>>,
    swatch_size: f32,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorPickerEvent> for ColorPickerPopover {}

impl ColorPickerPopover {
    pub(crate) fn new(
        look: Arc<ShadcnLook>,
        color: ColorValue,
        instance_id: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size(look, color, instance_id.into(), 30.0, cx)
    }

    fn new_with_size(
        look: Arc<ShadcnLook>,
        color: ColorValue,
        instance_id: String,
        swatch_size: f32,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size_and_content(look, color, instance_id, swatch_size, None, false, cx)
    }

    #[allow(dead_code)]
    pub(crate) fn new_with_content(
        look: Arc<ShadcnLook>,
        color: ColorValue,
        instance_id: impl Into<String>,
        swatch_size: f32,
        content: ColorPickerContent,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size_and_content(look, color, instance_id.into(), swatch_size, Some(content), false, cx)
    }

    fn new_with_size_and_content(
        look: Arc<ShadcnLook>,
        color: ColorValue,
        instance_id: String,
        swatch_size: f32,
        content: Option<ColorPickerContent>,
        include_copy_button: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let hsv = preview_hsv(color);
        let field = cx.new(|_| {
            ColorFieldState::saturation_value(format!("{instance_id}-field"), hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .no_border()
                .rounded(px(0.0))
        });
        let hue_slider = ColorSliderBuilder::hue(format!("{instance_id}-hue"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge()
            .spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha(format!("{instance_id}-alpha"), hsv.a, hsv)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let button = Button::new(format!("{instance_id}-trigger"))
            .typed(ColorSwatchData { color, size: swatch_size, radius: 4.0, open: false, checkerboard: true })
            .template(Arc::new(ColorSwatchButtonTemplate))
            .spawn(cx);

        let copy_button = include_copy_button.then(|| {
            shadcn::Button::new(format!("{instance_id}-copy"))
                .look(look.as_ref())
                .outline()
                .size(ShadcnSize::Sm)
                .label("Copy color spec")
                .spawn(cx)
        });

        let default_content: ColorPickerContent = {
            let field = field.clone();
            let hue_slider = hue_slider.clone();
            let alpha_slider = alpha_slider.clone();
            let look = look.clone();
            let copy_button = copy_button.clone();
            Arc::new(move |_, _, _, _| {
                let mut content = div()
                    .w(px(260.0))
                    .p(px(12.0))
                    .gap(px(10.0))
                    .flex()
                    .flex_col()
                    .bg(look.chrome().content_background)
                    .border_1()
                    .border_color(look.chrome().border)
                    .rounded(px(6.0))
                    .child(div().w(px(236.0)).h(px(180.0)).child(field.clone()))
                    .child(div().w(px(236.0)).child(hue_slider.clone()))
                    .child(div().w(px(236.0)).child(alpha_slider.clone()));
                if let Some(copy_button) = &copy_button {
                    content = content.child(copy_button.clone());
                }
                content.into_any_element()
            })
        };
        let content = content.unwrap_or(default_content);
        let picker_entity = cx.entity().clone();
        let content_for_popover = content.clone();
        let mut subscriptions = vec![
            cx.subscribe(&field, |picker, _, event: &ColorFieldEvent, cx| {
                let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                    return;
                };
                picker.hsv.a = picker.color.alpha();
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
                if let Ok(color) = picker.color.with_alpha(value) {
                    picker.color = color;
                    picker.hsv.a = value;
                    picker.sync_controls(cx, true);
                    cx.emit(ColorPickerEvent::Change(color));
                    cx.notify();
                }
            }),
        ];
        if let Some(copy_button) = &copy_button {
            subscriptions.push(cx.subscribe(copy_button, |picker, _, event: &ButtonEvent, cx| {
                if matches!(event, ButtonEvent::Click) {
                    cx.write_to_clipboard(ClipboardItem::new_string(format_color_readout(picker.color)));
                }
            }));
        }

        let button_for_popover = button.clone();
        let popover = PopoverButton::new(format!("{instance_id}-popover"))
            .typed(())
            .trigger(move |_, _| button_for_popover.clone().into_any_element())
            .content(move |_, window, cx| {
                let picker_for_change = picker_entity.clone();
                let on_change: ColorPickerChangeHandler = Arc::new(move |color, cx| {
                    picker_for_change.update(cx, |picker, cx| picker.set_color_and_emit(color, cx));
                });
                let color = picker_entity.read(cx).color;
                (content_for_popover)(color, on_change, window, cx)
            })
            .placement(PopoverPlacement::Smart)
            .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
            .offset_y(px(0.0))
            .initial_content_size(size(px(260.0), px(254.0)))
            .trigger_on_pointer_down(false)
            .spawn(cx);

        let popover_for_button = popover.clone();
        let button_subscription = cx.subscribe(&button, move |_, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                popover_for_button.update(cx, |popover, cx| popover.toggle_guarded(cx));
            }
        });

        Self {
            color,
            hsv,
            sync: ColorCompositionSync::new(),
            field,
            hue_slider,
            alpha_slider,
            alpha_domain,
            popover,
            button,
            swatch_size,
            _subscriptions: subscriptions.into_iter().chain([button_subscription]).collect(),
        }
    }

    pub(crate) fn new_palette(
        look: Arc<ShadcnLook>,
        color: ColorValue,
        instance_id: impl Into<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_size_and_content(look, color, instance_id.into(), 55.0, None, true, cx)
    }

    pub(crate) fn set_color(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        self.color = color;
        self.hsv = preview_hsv(color);
        let swatch_size = self.swatch_size;
        self.button.update(cx, |button, cx| {
            button.set_data(
                ColorSwatchData { color, size: swatch_size, radius: 4.0, open: false, checkerboard: true },
                cx,
            )
        });
        self.sync_controls(cx, true);
        cx.notify();
    }

    fn set_color_and_emit(&mut self, color: ColorValue, cx: &mut Context<Self>) {
        self.set_color(color, cx);
        cx.emit(ColorPickerEvent::Change(color));
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
        self.color = self.hsv.to_color_value();
        cx.emit(ColorPickerEvent::Change(self.color));
        cx.notify();
    }
}

impl Render for ColorPickerPopover {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.popover.clone()
    }
}

fn preview_hsv(source: ColorValue) -> Hsv {
    Hsv::from_hsla_ext(
        gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap_or_else(|_| gpui::transparent_black()),
    )
}
#[cfg(all(test, feature = "test-support"))]
mod source_tests {
    use super::*;
    #[test]
    fn loading_and_syncing_picker_preserve_p3_and_alpha() {
        let app = gpui::TestAppContext::single();
        let source = ColorValue::display_p3(1.123456, -0.123456, 0.234567, 0.345678);
        app.update(|cx| {
            let picker =
                cx.new(|cx| ColorPickerPopover::new(Arc::new(ShadcnLook::built_in()), source, "source-test", cx));
            picker.update(cx, |picker, cx| {
                assert_eq!(picker.color, source);
                picker.sync_controls(cx, true);
                assert_eq!(picker.color, source);
                let translucent = source.with_alpha(0.456789).unwrap();
                picker.set_color_and_emit(translucent, cx);
                assert_eq!(picker.color, translucent);
            });
        });
    }
}
