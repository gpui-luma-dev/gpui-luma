use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, IntoElement, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::textfield::{TextField, TextFieldBuilder, TextFieldLook, TextFieldLookOverride, TextFieldEvent};
use gpui_luma::vstack;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::super::ThemeSidebar;
use super::super::model::TOKEN_CATEGORIES;
use super::super::parsing::{effective_token_color, token_hex_value};
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::parse_hex_color;
use crate::studio::token_color_row::token_color_row;

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

pub(in crate::studio::theme_sidebar) fn build_token_fields(
    look: &Arc<ShadcnLook>,
    global_overrides: &HashMap<String, Hsla>,
    cx: &mut Context<ThemeSidebar>,
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

pub(in crate::studio::theme_sidebar) fn render_colors_panel(sidebar: &ThemeSidebar) -> AnyElement {
    let chrome = sidebar.vm.look.chrome();
    let has_catalog = sidebar.vm.look.has_css_catalog();

    let mut body = div().flex().flex_col().w_full().gap(px(10.0));

    if !has_catalog {
        body = body.child(
            div()
                .text_xs()
                .text_color(chrome.muted_text)
                .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
        );
    }

    body.child(div().w_full().child(sidebar.token_accordion.clone())).into_any_element()
}

pub(in crate::studio::theme_sidebar) fn category_token_content(
    sidebar: &ThemeSidebar,
    tokens: &[(&str, &str)],
) -> AnyElement {
    let theme = &sidebar.vm.look;
    let overrides = &sidebar.vm.global_overrides;
    let chrome = theme.chrome();
    let row_label_typography = theme.mode_tokens().typography.text.label;

    let mut rows = vstack! {
        gap=6;
    };

    for (token, label) in tokens {
        let Some(field) = sidebar.vm.token_fields.get(*token) else {
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

impl ThemeSidebar {
    pub(in crate::studio::theme_sidebar) fn sync_color_panel_templates(
        &self,
        theme: &Arc<ShadcnLook>,
        cx: &mut Context<Self>,
    ) {
        for field in self.vm.token_fields.values() {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_look_override(Some(token_field_look_override_arc()), cx);
            });
        }
    }

    pub(in crate::studio::theme_sidebar) fn sync_color_panel_values(
        &mut self,
        look: &ShadcnLook,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let Some(field) = self.vm.token_fields.get(*token) else {
                    continue;
                };
                let value = token_hex_value(look, &overrides.global_color_overrides, token);
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }
}

pub(in crate::studio::theme_sidebar) fn wire_color_subscriptions(
    sidebar: &Entity<ThemeSidebar>,
    cx: &mut Context<ThemeStudioApp>,
    subscriptions: &mut Vec<Subscription>,
) {
    for (_, tokens) in TOKEN_CATEGORIES {
        for (token, _) in *tokens {
            let field = sidebar.read(cx).vm.token_fields.get(*token).expect("token field").clone();
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
