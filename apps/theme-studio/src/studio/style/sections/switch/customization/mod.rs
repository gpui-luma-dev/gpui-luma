use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::{ButtonEvent, ButtonRenderModel};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::switch::{Switch, SwitchBuilder};
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::paint::switch_look;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;

use crate::studio::style::shared::icons::render_lucide_icon;

const DEMO_STACK_GAP: f32 = 4.0;
const LABEL_TRACK_EXTRA_WIDTH: f32 = 12.0;
const LABEL_TRACK_PADDING_X: f32 = 4.0;

pub(crate) struct SwitchCustomizationPreview {
    look: Arc<ShadcnLook>,
    primary_switch: Switch,
    secondary_switch: Switch,
    disabled_off_switch: Switch,
    disabled_on_switch: Switch,
    _subscriptions: Vec<gpui::Subscription>,
}

impl SwitchCustomizationPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let primary_switch = labeled_switch(
            look.primary_switch("theme-studio-labeled-switch-primary").with_data(true),
            look.clone(),
            ShadcnButtonStyle::Primary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);
        let secondary_switch = labeled_switch(
            look.secondary_switch("theme-studio-labeled-switch-secondary").with_data(false),
            look.clone(),
            ShadcnButtonStyle::Secondary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);
        let disabled_off_switch = labeled_switch(
            look.primary_switch("theme-studio-labeled-switch-disabled-off").with_data(false).enabled(false),
            look.clone(),
            ShadcnButtonStyle::Primary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);
        let disabled_on_switch = labeled_switch(
            look.primary_switch("theme-studio-labeled-switch-disabled-on").with_data(true).enabled(false),
            look.clone(),
            ShadcnButtonStyle::Primary,
        )
        .content(|_, _| div().into_any_element())
        .spawn(cx);

        let mut preview = Self {
            look,
            primary_switch,
            secondary_switch,
            disabled_off_switch,
            disabled_on_switch,
            _subscriptions: Vec::new(),
        };
        preview.subscribe(cx);
        preview
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.primary_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Primary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Primary), cx);
        });
        self.secondary_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Secondary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Secondary), cx);
        });
        self.disabled_off_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Primary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Primary), cx);
        });
        self.disabled_on_switch.update(cx, |switch, cx| {
            switch.set_template(look.switch_template(ShadcnButtonStyle::Primary), cx);
            switch.set_switch_track_content(labeled_track_content(look.clone(), ShadcnButtonStyle::Primary), cx);
        });
        cx.notify();
    }

    fn subscribe(&mut self, cx: &mut Context<Self>) {
        self._subscriptions.push(cx.subscribe(&self.primary_switch, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                flip_labeled_switch(&this.primary_switch, cx);
                cx.notify();
            }
        }));
        self._subscriptions.push(cx.subscribe(&self.secondary_switch, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                flip_labeled_switch(&this.secondary_switch, cx);
                cx.notify();
            }
        }));
    }
}

fn labeled_switch(builder: SwitchBuilder, look: Arc<ShadcnLook>, style: ShadcnButtonStyle) -> SwitchBuilder {
    builder
        .track_width_extra(LABEL_TRACK_EXTRA_WIDTH)
        .track_content(labeled_track_content(look.clone(), style))
        .thumb_content(labeled_thumb_content(look, style))
}

fn labeled_track_content(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> impl Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync + 'static {
    move |model, _| {
        let tokens = look.mode_tokens();
        let palette = switch_look(tokens.as_ref(), look.mode(), style, model.data, model.state);
        let on_palette = switch_look(tokens.as_ref(), look.mode(), style, true, InteractionState::default());
        let state_label = if model.data { "ON" } else { "OFF" };
        let state_label_color = if model.data && !model.state.disabled {
            on_palette.thumb_background
        } else {
            palette.label_color
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .px(px(LABEL_TRACK_PADDING_X))
            .when(model.data, |label_row| label_row.justify_start())
            .when(!model.data, |label_row| label_row.justify_end())
            .text_size(px((palette.label_typography.size * 0.85).max(10.0)))
            .line_height(px((palette.label_typography.line_height * 0.85).max(12.0)))
            .font_family(palette.label_font_family.clone())
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(state_label_color)
            .child(state_label)
            .into_any_element()
    }
}

fn labeled_thumb_content(
    look: Arc<ShadcnLook>,
    style: ShadcnButtonStyle,
) -> impl Fn(&ButtonRenderModel<bool>, &mut App) -> AnyElement + Send + Sync + 'static {
    move |model, _| {
        let tokens = look.mode_tokens();
        let palette = switch_look(tokens.as_ref(), look.mode(), style, model.data, model.state);
        let icon = if model.data { LucideIcon::Check } else { LucideIcon::X };
        div().text_color(palette.track_background).child(render_lucide_icon(icon, 11.0)).into_any_element()
    }
}

fn flip_labeled_switch(switch: &Switch, cx: &mut Context<SwitchCustomizationPreview>) {
    switch.update(cx, |button, cx| {
        button.set_data(!*button.data(), cx);
    });
}

impl Render for SwitchCustomizationPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let primary_checked = *self.primary_switch.read(cx).data();
        let secondary_checked = *self.secondary_switch.read(cx).data();

        div().w_full().flex().flex_col().items_center().gap_8().child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_start()
                .justify_center()
                .gap_8()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(demo_block(
                            "Primary labeled switch",
                            chrome.muted_text,
                            self.primary_switch.clone(),
                            state_label(primary_checked, chrome.body_text),
                        ))
                        .child(demo_block(
                            "Secondary labeled switch",
                            chrome.muted_text,
                            self.secondary_switch.clone(),
                            state_label(secondary_checked, chrome.body_text),
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(demo_block(
                            "Disabled (off)",
                            chrome.muted_text,
                            self.disabled_off_switch.clone(),
                            state_label(false, chrome.body_text),
                        ))
                        .child(demo_block(
                            "Disabled (on)",
                            chrome.muted_text,
                            self.disabled_on_switch.clone(),
                            state_label(true, chrome.body_text),
                        )),
                ),
        )
    }
}

fn demo_block(label: &'static str, label_color: gpui::Hsla, control: Switch, value: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(DEMO_STACK_GAP))
        .child(section_label(label, label_color))
        .child(control)
        .child(value)
        .into_any_element()
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(label)
}

fn state_label(checked: bool, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(if checked {
        "State: ON"
    } else {
        "State: OFF"
    })
}

pub(crate) fn render_switch_customization_body(preview: Entity<SwitchCustomizationPreview>) -> AnyElement {
    preview.into_any_element()
}
