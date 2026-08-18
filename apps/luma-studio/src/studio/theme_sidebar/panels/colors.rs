use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, EventEmitter, Hsla, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::anchored_panel::{AnchoredPanel, AnchoredPanelDismissPolicy, AnchoredPanelPlacement};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
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
use gpui_luma::controls::textfield::{TextField, TextFieldBuilder, TextFieldEvent, TextFieldLook, TextFieldLookOverride};
use gpui_luma::vstack;
use gpui_luma_look_shadcn::ShadcnLook;

use super::super::model::TOKEN_CATEGORIES;
use super::super::parsing::effective_token_color;
use super::{category_item_id, expanded_category_ids, spawn_compact_textfield};
use crate::studio::app::LumaStudioApp;
use crate::studio::color_format::{format_compact_hsla, parse_compact_hsla};
use crate::studio::overrides::StudioOverrides;
use crate::studio::token_color_row::token_color_row;

pub struct ColorsPanel {
    look: Arc<ShadcnLook>,
    global_overrides: HashMap<String, Hsla>,
    token_fields: HashMap<String, TextField>,
    token_pickers: HashMap<String, Entity<ColorPickerPopover>>,
    token_accordion: Entity<AccordionControl>,
}

impl ColorsPanel {
    pub fn new(look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) -> Self {
        let global_overrides = overrides.global_color_overrides.clone();
        let token_fields = build_token_fields(&look, &global_overrides, cx);
        let token_pickers = build_token_pickers(&look, &global_overrides, cx);
        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self { look, global_overrides, token_fields, token_pickers, token_accordion }
    }

    pub fn apply_theme_snapshot(&mut self, look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let expanded = self.expanded_token_category_ids(cx);
        self.look = look;
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_templates(&theme, cx);
        self.sync_values(theme.as_ref(), overrides, cx);
        self.sync_pickers(theme.as_ref(), overrides, cx);
        self.token_accordion = Self::build_token_accordion(cx.entity(), theme, &expanded, cx);
        cx.notify();
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_values(theme.as_ref(), overrides, cx);
        self.sync_pickers(theme.as_ref(), overrides, cx);
        cx.notify();
    }

    pub fn wire_subscriptions(
        panel: &Entity<Self>,
        cx: &mut Context<LumaStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let field = panel.read(cx).token_fields.get(*token).expect("token field").clone();
                let token_key = token.to_string();
                subscriptions.push(cx.subscribe(&field, move |app, _, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } = event {
                        let Some(color) = parse_compact_hsla(value) else {
                            return;
                        };
                        app.set_global_color(&token_key, color, cx);
                    }
                }));
                let picker = panel.read(cx).token_pickers.get(*token).expect("token picker").clone();
                let token_key = token.to_string();
                subscriptions.push(cx.subscribe(&picker, move |app, _, event: &ColorPickerEvent, cx| {
                    let ColorPickerEvent::Change(color) = event;
                    app.set_global_color(&token_key, *color, cx);
                }));
            }
        }
    }

    fn sync_templates(&self, theme: &Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in self.token_fields.values() {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_look_override(Some(token_field_look_override_arc()), cx);
            });
        }
    }

    fn sync_values(&mut self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let Some(field) = self.token_fields.get(*token) else {
                    continue;
                };
                let value = format_compact_hsla(effective_token_color(look, &overrides.global_color_overrides, token));
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    fn sync_pickers(&self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        for (token, picker) in &self.token_pickers {
            let color = effective_token_color(look, &overrides.global_color_overrides, token);
            picker.update(cx, |picker, cx| picker.set_color(color, cx));
        }
    }

    fn build_token_accordion(
        panel: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("luma-studio-token-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let items = *tokens;
            let panel = panel.clone();
            let id = category_item_id("token", category);
            let expanded = if expanded_categories.is_empty() {
                *category == "BASE"
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| panel.read(cx).category_token_content(items)),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    fn expanded_token_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(
            &self.token_accordion,
            TOKEN_CATEGORIES.iter().map(|(category, _)| *category),
            "token",
            cx,
        )
    }

    pub(crate) fn set_all_categories_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        for (index, (category, _)) in TOKEN_CATEGORIES.iter().enumerate() {
            let id = category_item_id("token", category);
            if self.token_accordion.read(cx).is_expanded(&id.clone().into()) != expanded {
                self.token_accordion.update(cx, |accordion, cx| accordion.toggle_item(index, cx));
            }
        }
    }

    fn category_token_content(&self, tokens: &[(&str, &str)]) -> AnyElement {
        let theme = &self.look;
        let chrome = theme.chrome();
        let row_label_typography = theme.mode_tokens().typography.text.label;

        let mut rows = vstack! {
            gap=6;
        };

        for (token, label) in tokens {
            let Some(field) = self.token_fields.get(*token) else {
                continue;
            };
            let label = label.to_string();
            let field = field.clone();
            let Some(picker) = self.token_pickers.get(*token) else {
                continue;
            };

            rows = vstack! {
                gap=6;
                rows,
                token_color_row(
                    label,
                    picker.clone(),
                    field,
                    &chrome,
                    &row_label_typography,
                ),
            };
        }

        rows.into_any_element()
    }
}

#[derive(Clone, Debug)]
pub enum ColorPickerEvent {
    Change(Hsla),
}

struct ColorPickerPopover {
    look: Arc<ShadcnLook>,
    color: Hsla,
    hsv: Hsv,
    sync: ColorCompositionSync,
    field: Entity<ColorFieldState>,
    hue_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    panel: Entity<AnchoredPanel>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ColorPickerEvent> for ColorPickerPopover {}

impl ColorPickerPopover {
    fn new(look: Arc<ShadcnLook>, color: Hsla, cx: &mut Context<Self>) -> Self {
        let hsv = Hsv::from_hsla_ext(color);
        let field = cx.new(|_| {
            ColorFieldState::saturation_value("colors-picker-field", hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .rounded(px(4.0))
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
                // The SV field owns H/S/V only. Keep the alpha selected by the
                // alpha slider; field events must never reset it.
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
            _subscriptions: subscriptions,
        }
    }

    fn set_color(&mut self, color: Hsla, cx: &mut Context<Self>) {
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
        let mut swatch = div()
            .id("colors-picker-swatch")
            .size(px(30.0))
            .rounded(px(4.0))
            .on_click(move |_, window, cx| {
                panel.update(cx, |panel, cx| panel.toggle_guarded_from(None, cx));
                window.prevent_default();
            })
            .child(ColorSwatch::new(color).height(px(30.0)).rounded(px(4.0)).checkerboard(true));
        let panel_for_bounds = self.panel.clone();
        swatch = swatch.on_prepaint(move |bounds, _, cx| {
            panel_for_bounds.update(cx, |panel, cx| panel.set_anchor_bounds(bounds, cx));
        });
        div().size(px(30.0)).relative().child(swatch).child(self.panel.clone())
    }
}

fn build_token_pickers(
    look: &Arc<ShadcnLook>,
    overrides: &HashMap<String, Hsla>,
    cx: &mut Context<ColorsPanel>,
) -> HashMap<String, Entity<ColorPickerPopover>> {
    let mut pickers = HashMap::new();
    for (_, tokens) in TOKEN_CATEGORIES {
        for (token, _) in *tokens {
            let color = effective_token_color(look, overrides, token);
            let picker = cx.new(|cx| ColorPickerPopover::new(look.clone(), color, cx));
            pickers.insert(token.to_string(), picker);
        }
    }
    pickers
}

impl Render for ColorsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let has_catalog = self.look.has_css_catalog();

        let mut body = div().flex().flex_col().w_full().gap(px(10.0));

        if !has_catalog {
            body = body.child(
                div()
                    .text_xs()
                    .text_color(chrome.muted_text)
                    .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
            );
        }

        body.child(div().w_full().child(self.token_accordion.clone()))
    }
}

/// System monospace face for hex values. Theme CSS `font-mono` families (e.g. Fira Code) are not
/// registered with GPUI unless explicitly loaded, so token fields use a native face per platform.
fn token_field_mono_font() -> SharedString {
    #[cfg(target_os = "macos")]
    {
        "Menlo".into()
    }
    #[cfg(target_os = "windows")]
    {
        return "Consolas".into();
    }
    #[cfg(target_os = "linux")]
    {
        return "DejaVu Sans Mono".into();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "monospace".into()
    }
}

const TOKEN_FIELD_FONT_SIZE: f32 = 12.0;
const TOKEN_FIELD_LINE_HEIGHT: f32 = 16.0;

fn apply_token_field_look(mut look: TextFieldLook) -> TextFieldLook {
    look.font_family = token_field_mono_font();
    look.typography.size = TOKEN_FIELD_FONT_SIZE;
    look.typography.line_height = TOKEN_FIELD_LINE_HEIGHT;
    look
}

pub(super) fn token_field_look_override_arc() -> TextFieldLookOverride {
    Arc::new(apply_token_field_look)
}

pub(super) fn apply_token_field_style(builder: TextFieldBuilder) -> TextFieldBuilder {
    builder.compact().look_override(apply_token_field_look)
}

fn build_token_fields(
    look: &Arc<ShadcnLook>,
    global_overrides: &HashMap<String, Hsla>,
    cx: &mut Context<ColorsPanel>,
) -> HashMap<String, TextField> {
    let mut token_fields = HashMap::new();
    for (_, tokens) in TOKEN_CATEGORIES {
        for (token, _) in *tokens {
            let initial = format_compact_hsla(effective_token_color(look, global_overrides, token));
            let field = spawn_compact_textfield(look, &format!("token-{token}"), initial, cx);
            token_fields.insert(token.to_string(), field);
        }
    }
    token_fields
}
