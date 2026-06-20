use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, Hsla, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*,
    px,
};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use gpui_luma::controls::textfield::{TextField, TextFieldBuilder, TextFieldEvent, TextFieldLook, TextFieldLookOverride};
use gpui_luma::vstack;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::super::model::TOKEN_CATEGORIES;
use super::super::parsing::{effective_token_color, token_hex_value};
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::parse_hex_color;
use crate::studio::token_color_row::token_color_row;

pub struct ColorsPanel {
    look: Arc<ShadcnLook>,
    global_overrides: HashMap<String, Hsla>,
    token_fields: HashMap<String, TextField>,
    token_accordion: Entity<AccordionControl>,
}

impl ColorsPanel {
    pub fn new(look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) -> Self {
        let global_overrides = overrides.global_color_overrides.clone();
        let token_fields = build_token_fields(&look, &global_overrides, cx);
        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self { look, global_overrides, token_fields, token_accordion }
    }

    pub fn apply_theme_snapshot(&mut self, look: Arc<ShadcnLook>, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let expanded = self.expanded_token_category_ids(cx);
        self.look = look;
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_templates(&theme, cx);
        self.sync_values(theme.as_ref(), overrides, cx);
        self.token_accordion = Self::build_token_accordion(cx.entity(), theme, &expanded, cx);
        cx.notify();
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.look.clone();
        self.sync_values(theme.as_ref(), overrides, cx);
        cx.notify();
    }

    pub fn wire_subscriptions(
        panel: &Entity<Self>,
        cx: &mut Context<ThemeStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let field = panel.read(cx).token_fields.get(*token).expect("token field").clone();
                let token_key = token.to_string();
                subscriptions.push(cx.subscribe(&field, move |app, _, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } = event {
                        let Some(color) = parse_hex_color(value) else {
                            return;
                        };
                        app.set_global_color(&token_key, color, cx);
                    }
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
                let value = token_hex_value(look, &overrides.global_color_overrides, token);
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    fn build_token_accordion(
        panel: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("theme-studio-token-accordion")
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

    fn category_token_content(&self, tokens: &[(&str, &str)]) -> AnyElement {
        let theme = &self.look;
        let overrides = &self.global_overrides;
        let chrome = theme.chrome();
        let row_label_typography = theme.mode_tokens().typography.text.label;

        let mut rows = vstack! {
            gap=6;
        };

        for (token, label) in tokens {
            let Some(field) = self.token_fields.get(*token) else {
                continue;
            };
            let color = effective_token_color(theme, overrides, token);
            let label = label.to_string();
            let field = field.clone();

            rows = vstack! {
                gap=6;
                rows,
                token_color_row(label, color, field, &chrome, &row_label_typography),
            };
        }

        rows.into_any_element()
    }
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
            let initial = token_hex_value(look, global_overrides, token);
            let field = apply_token_field_style(look.textfield(format!("theme-studio-token-{token}")))
                .value(initial)
                .full_width(true)
                .spawn(cx);
            token_fields.insert(token.to_string(), field);
        }
    }
    token_fields
}

fn category_item_id(prefix: &str, category: &str) -> String {
    format!("{prefix}-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

fn expanded_category_ids<'a>(
    accordion: &Entity<AccordionControl>,
    categories: impl IntoIterator<Item = &'a str>,
    prefix: &str,
    cx: &App,
) -> HashSet<String> {
    let accordion = accordion.read(cx);
    categories
        .into_iter()
        .filter_map(|category| {
            let id = category_item_id(prefix, category);
            accordion.is_expanded(&id.clone().into()).then_some(id)
        })
        .collect()
}
