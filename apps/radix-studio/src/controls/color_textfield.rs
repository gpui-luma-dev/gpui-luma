use std::sync::Arc;

use crate::color_hex::{format_hex, parse_hex};

use gpui::{
    Context, Entity, EventEmitter, Hsla, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::button::{Button, ButtonEvent};
use gpui_luma::controls::popover_button::{PopoverButton, PopoverDismissPolicy, PopoverPlacement};
use gpui_luma::controls::slider::{SliderControl, SliderEvent};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use gpui_luma_color::color_slider::color_spec::Hsv;
use gpui_luma_color::color_slider::{ColorSliderBuilder, primary_slider_value, sizing};
use gpui_luma_color::composition::ColorCompositionSync;
use gpui_luma_color::{ColorSwatchButtonTemplate, ColorSwatchData};
use gpui_luma_look_radix::{Look, SemanticRole};
use gpui_luma_look_radix as radix;

const SWATCH_SIZE: f32 = 16.0;
const SWATCH_RADIUS: f32 = 2.0;

#[derive(Clone, Debug)]
pub enum ColorTextFieldEvent {
    Change { color: Hsla },
}

pub struct ColorTextField {
    id: SharedString,
    look: Arc<Look>,
    color: Hsla,
    hsv: Hsv,
    focused: bool,
    swatch: Entity<Button<ColorSwatchData>>,
    field: TextField,
    picker_field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    picker_hex: TextField,
    popover: Entity<PopoverButton>,
    sync: ColorCompositionSync,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ButtonEvent> for ColorTextField {}
impl EventEmitter<ColorTextFieldEvent> for ColorTextField {}

impl ColorTextField {
    pub const HEIGHT: f32 = 32.0;

    pub fn new(look: Arc<Look>, id: impl Into<SharedString>, color: Hsla, cx: &mut Context<Self>) -> Self {
        let id = id.into();
        let hsv = Hsv::from_hsla_ext(color);

        let swatch = Button::new(format!("{id}-swatch"))
            .typed(ColorSwatchData {
                color,
                size: SWATCH_SIZE,
                radius: SWATCH_RADIUS,
                open: false,
                checkerboard: false,
            })
            .template(Arc::new(ColorSwatchButtonTemplate))
            .tab_stop(false)
            .spawn(cx);
        let field = embedded_textfield(&look, format!("{id}-field"), format_hex(color), cx);
        let picker_field = cx.new(|_| {
            ColorFieldState::saturation_value(format!("{id}-picker-sv"), hsv, sizing::THUMB_SIZE_MEDIUM)
                .no_border()
                .rounded(px(0.0))
        });
        let hue_builder = ColorSliderBuilder::hue(format!("{id}-picker-hue"), hsv.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let hue_slider = hue_builder.spawn(cx);
        let picker_hex = embedded_textfield(&look, format!("{id}-picker-hex"), format_hex(color), cx);

        let swatch_for_popover = swatch.clone();
        let picker_field_for_popover = picker_field.clone();
        let hue_slider_for_popover = hue_slider.clone();
        let picker_hex_for_popover = picker_hex.clone();
        let popup_look = Arc::clone(&look);
        let popover = PopoverButton::new(format!("{id}-popover"))
            .typed(())
            .trigger(move |_, _| swatch_for_popover.clone().into_any_element())
            .content(move |_, _, _| {
                div()
                    .w(px(360.0))
                    .p(px(12.0))
                    .gap(px(10.0))
                    .flex()
                    .flex_col()
                    .bg(popup_look.resolve_role(SemanticRole::Surface).hsla())
                    .child(div().w_full().h(px(180.0)).child(picker_field_for_popover.clone()))
                    .child(div().w_full().child(hue_slider_for_popover.clone()))
                    .child(div().w_full().child(picker_hex_for_popover.clone()))
                    .into_any_element()
            })
            .placement(PopoverPlacement::Smart)
            .dismiss_policy(PopoverDismissPolicy::CloseOnClickAway)
            .offset_y(px(4.0))
            .initial_content_size(gpui::size(px(384.0), px(264.0)))
            .trigger_on_pointer_down(false)
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&field, |this, _, event: &TextFieldEvent, cx| match event {
            TextFieldEvent::FocusChanged { focused } => {
                this.focused = *focused;
                cx.notify();
            }
            TextFieldEvent::Change { value } => this.apply_hex(value, cx),
            _ => {}
        }));
        subscriptions.push(cx.subscribe(&picker_hex, |this, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                this.apply_hex(value, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&picker_field, |this, _, event: &ColorFieldEvent, cx| {
            let (ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv)) = event else {
                return;
            };
            this.hsv.s = hsv.s;
            this.hsv.v = hsv.v;
            this.apply_hsv(cx);
        }));
        subscriptions.push(cx.subscribe(&hue_slider, |this, _, event: &SliderEvent, cx| {
            let Some(value) = primary_slider_value(event) else {
                return;
            };
            this.hsv.h = value;
            this.apply_hsv(cx);
        }));
        let popover_for_swatch = popover.clone();
        subscriptions.push(cx.subscribe(&swatch, move |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                popover_for_swatch.update(cx, |popover, cx| popover.toggle_guarded(cx));
                cx.emit(ButtonEvent::Click);
                this.focused = false;
                cx.notify();
            }
        }));

        Self {
            id,
            look,
            color,
            hsv,
            focused: false,
            swatch,
            field,
            picker_field,
            hue_slider,
            picker_hex,
            popover,
            sync: ColorCompositionSync::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn set_color(&mut self, color: Hsla, cx: &mut Context<Self>) {
        self.color = color;
        self.hsv = Hsv::from_hsla_ext(color);
        self.sync_fields(cx);
        cx.notify();
    }

    fn apply_hex(&mut self, value: &str, cx: &mut Context<Self>) {
        let Some(color) = parse_hex(value) else { return };
        self.set_color(color, cx);
        cx.emit(ColorTextFieldEvent::Change { color });
    }

    fn apply_hsv(&mut self, cx: &mut Context<Self>) {
        let color = self.hsv.to_hsla_ext();
        // Keep HSV as the gesture source: converting black/gray back from HSL
        // discards hue (and at black, saturation), moving the hue slider spuriously.
        self.color = color;
        self.sync_fields(cx);
        cx.notify();
        cx.emit(ColorTextFieldEvent::Change { color });
    }

    fn sync_fields(&self, cx: &mut Context<Self>) {
        let color = self.color;
        self.swatch.update(cx, |swatch, cx| {
            swatch.set_data(
                ColorSwatchData { color, size: SWATCH_SIZE, radius: SWATCH_RADIUS, open: false, checkerboard: false },
                cx,
            );
        });

        self.picker_field.update(cx, |field, cx| field.set_hsv(self.hsv, cx));
        self.sync.sync_slider_value(&self.hue_slider, self.hsv.h, cx);
        let hex = format_hex(self.color);
        // Both text fields represent the same selected color as the swatch.
        // Avoid resetting the caret when the text is already synchronized.
        for field in [&self.field, &self.picker_hex] {
            field.update(cx, |field, cx| {
                if field.value().as_ref() != hex {
                    field.set_value(hex.clone(), cx);
                }
            });
        }
    }
}

impl Render for ColorTextField {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let border = if self.focused {
            self.look.resolve_role(SemanticRole::Primary).hsla()
        } else {
            self.look.resolve_role(SemanticRole::Border).hsla()
        };
        let background = self.look.resolve_role(SemanticRole::Background).hsla();

        div()
            .id(self.id.clone())
            .h(px(Self::HEIGHT))
            .w_full()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(7.0))
            .bg(background)
            .border_1()
            .border_color(border)
            .rounded(px(4.0))
            .child(self.popover.clone())
            .child(div().min_w(px(0.0)).flex_1().child(self.field.clone()))
    }
}

fn embedded_textfield(
    look: &Arc<Look>,
    id: impl Into<SharedString>,
    value: impl Into<String>,
    cx: &mut Context<ColorTextField>,
) -> TextField {
    radix::TextField::new(id)
        .look(look)
        .value(value.into())
        .look_override(|mut field| {
            field.background = gpui::transparent_black();
            field.border = gpui::transparent_black();
            field.focus_border = None;
            field.shadow = None;
            field.padding_x = 0.0;
            field.padding_y = 0.0;
            // The enclosing color field supplies the chrome and vertical spacing.
            // Center the text line itself alongside the 16px swatch.
            field.min_height = field.typography.line_height;
            field.radius = 0.0;
            field.border_width = 0.0;
            field
        })
        .spawn(cx)
}
