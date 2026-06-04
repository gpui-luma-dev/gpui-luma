use std::collections::HashMap;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use super::token_color_row::token_color_row;
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::textfield::{TextField, TextFieldAppearance, TextFieldAppearanceOverride, TextFieldEvent};
use gpui_luma::theme::{RadixTheme, RadixThemeControlExt};
use gpui_luma::vstack;

use gpui::Hsla;

use crate::studio::app::ThemeStudioApp;
use crate::studio::export::{catalog_color_for_token, token_css_name};
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::{format_hex_color, parse_hex_color};
use crate::theme::available_theme_names;

const TOKEN_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("BASE", &[("background", "Background"), ("foreground", "Foreground")]),
    ("PRIMARY", &[("primary", "Background"), ("primary-foreground", "Foreground")]),
    ("SECONDARY", &[("secondary", "Background"), ("secondary-foreground", "Foreground")]),
    ("ACCENT", &[("accent", "Background"), ("accent-foreground", "Foreground")]),
    ("CARD", &[("card", "Background"), ("card-foreground", "Foreground")]),
    ("POPOVER", &[("popover", "Background"), ("popover-foreground", "Foreground")]),
    ("MUTED", &[("muted", "Background"), ("muted-foreground", "Foreground")]),
    ("DESTRUCTIVE", &[("destructive", "Background"), ("destructive-foreground", "Foreground")]),
    ("BORDER & INPUT", &[("border", "Border"), ("input", "Input"), ("ring", "Ring")]),
    (
        "CHART",
        &[
            ("chart-1", "Chart 1"),
            ("chart-2", "Chart 2"),
            ("chart-3", "Chart 3"),
            ("chart-4", "Chart 4"),
            ("chart-5", "Chart 5"),
        ],
    ),
    (
        "SIDEBAR",
        &[
            ("sidebar", "Background"),
            ("sidebar-foreground", "Foreground"),
            ("sidebar-primary", "Primary"),
            ("sidebar-primary-foreground", "Primary Foreground"),
            ("sidebar-accent", "Accent"),
            ("sidebar-accent-foreground", "Accent Foreground"),
            ("sidebar-border", "Border"),
            ("sidebar-ring", "Ring"),
        ],
    ),
];

/// Theme token editing state owned by the sidebar (selectors, fields, overrides).
pub struct ThemeSidebarViewModel {
    pub radix_theme: std::sync::Arc<RadixTheme>,
    pub global_overrides: HashMap<String, Hsla>,
    pub token_fields: HashMap<String, TextField>,
}

pub struct ThemeSidebar {
    vm: ThemeSidebarViewModel,
    theme_selector: Entity<Selector>,
    token_accordion: Entity<AccordionControl>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<ThemeStudioApp>,
        radix_theme: std::sync::Arc<RadixTheme>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let theme_items = theme_selector_items();
        let theme_selector = radix_theme
            .selector("theme-studio-theme-selector")
            .label("Theme")
            .items(theme_items)
            .selected_id(active_theme_id.clone())
            .spawn(cx);

        let mut token_fields = HashMap::new();

        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let initial = token_hex_value(&radix_theme, &global_overrides, token);

                let field = radix_theme
                    .textfield(format!("theme-studio-token-{token}"))
                    .value(initial)
                    .full_width(true)
                    .appearance_override(token_field_appearance_override())
                    .spawn(cx);

                token_fields.insert(token.to_string(), field);
            }
        }

        let sidebar_for_accordion = cx.entity();
        let mut accordion_builder = radix_theme
            .accordion("theme-studio-token-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(radix_theme.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let items = *tokens;
            let sidebar = sidebar_for_accordion.clone();
            let mut item = AccordionItem::new(
                category_item_id(category),
                AccordionTrigger::new(*category),
                AccordionContent::custom(move |_window, cx| {
                    category_token_content(sidebar.read(cx), items).into_any_element()
                }),
            );
            if *category == "BASE" {
                item = item.expanded(true);
            }
            accordion_builder = accordion_builder.item(item);
        }

        let token_accordion = accordion_builder.spawn(cx);

        Self {
            vm: ThemeSidebarViewModel { radix_theme, global_overrides, token_fields },
            theme_selector,
            token_accordion,
        }
    }

    /// Wire selector and token field events on the app entity.
    ///
    /// Subscriptions must not be registered on `ThemeSidebar` itself: `cx.subscribe` re-enters
    /// the subscriber entity, and handlers call `theme_sidebar.update`, which panics.
    pub fn wire_subscriptions(
        sidebar: &Entity<Self>,
        cx: &mut Context<ThemeStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let theme_selector = sidebar.read(cx).theme_selector.clone();
        subscriptions.push(cx.subscribe(&theme_selector, |app, _, event: &SelectorEvent, cx| {
            let SelectorEvent::Change { item_id, .. } = event;
            app.change_theme(item_id.as_ref(), cx);
        }));

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

    pub fn apply_theme_snapshot(
        &mut self,
        radix_theme: std::sync::Arc<RadixTheme>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        self.vm.radix_theme = radix_theme;
        self.vm.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.vm.radix_theme.clone();
        self.sync_control_templates(&theme, cx);
        self.sync_token_fields_from(theme.as_ref(), overrides, cx);
        self.notify_token_accordion(cx);
    }

    pub(crate) fn sync_control_templates(&self, theme: &std::sync::Arc<RadixTheme>, cx: &mut Context<Self>) {
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_template(theme.selector_template(), cx);
        });
        self.token_accordion.update(cx, |accordion, cx| {
            accordion.set_template(theme.accordion_template(), cx);
        });
        for field in self.vm.token_fields.values() {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_appearance_override(Some(token_field_appearance_override_arc()), cx);
            });
        }
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.vm.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.vm.radix_theme.clone();
        self.sync_token_fields_from(theme.as_ref(), overrides, cx);
        self.notify_token_accordion(cx);
    }

    pub fn sync_theme_selector(&mut self, active_theme_id: impl Into<gpui::SharedString>, cx: &mut Context<Self>) {
        let active_theme_id = active_theme_id.into();
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_selected_id(active_theme_id, cx);
        });
    }

    pub fn sync_token_fields_from(
        &mut self,
        radix_theme: &RadixTheme,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let Some(field) = self.vm.token_fields.get(*token) else {
                    continue;
                };
                let value = token_hex_value(radix_theme, &overrides.global_color_overrides, token);
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    fn notify_token_accordion(&self, cx: &mut Context<Self>) {
        self.token_accordion.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

/// System monospace face for hex values. Theme CSS `font-mono` families (e.g. Fira Code) are not
/// registered with GPUI unless explicitly loaded, so token fields use a native face per platform.
fn token_field_mono_font() -> gpui::SharedString {
    #[cfg(target_os = "macos")]
    {
        return "Menlo".into();
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

fn apply_token_field_appearance(mut appearance: TextFieldAppearance) -> TextFieldAppearance {
    appearance.font_family = token_field_mono_font();
    appearance.typography.size = TOKEN_FIELD_FONT_SIZE;
    appearance.typography.line_height = TOKEN_FIELD_LINE_HEIGHT;
    appearance
}

fn token_field_appearance_override() -> impl Fn(TextFieldAppearance) -> TextFieldAppearance + Send + Sync + 'static {
    move |appearance| apply_token_field_appearance(appearance)
}

fn token_field_appearance_override_arc() -> TextFieldAppearanceOverride {
    std::sync::Arc::new(apply_token_field_appearance)
}

fn category_item_id(category: &str) -> String {
    format!("token-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

fn category_token_content(sidebar: &ThemeSidebar, tokens: &[(&str, &str)]) -> impl IntoElement {
    let theme = &sidebar.vm.radix_theme;
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

    rows
}

fn token_hex_value(radix_theme: &RadixTheme, global_overrides: &HashMap<String, Hsla>, token: &str) -> String {
    format_hex_color(effective_token_color(radix_theme, global_overrides, token))
}

fn effective_token_color(radix_theme: &RadixTheme, global_overrides: &HashMap<String, Hsla>, token: &str) -> Hsla {
    token_color_with_fallback(radix_theme, global_overrides, token, gpui::hsla(0.0, 0.0, 0.5, 1.0))
}

/// Resolves a theme token when present; otherwise uses `fallback` (e.g. chrome defaults).
fn token_color_with_fallback(
    radix_theme: &RadixTheme,
    global_overrides: &HashMap<String, Hsla>,
    token: &str,
    fallback: Hsla,
) -> Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| catalog_color_for_token(radix_theme, token))
        .unwrap_or(fallback)
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.vm.radix_theme.chrome();
        let has_catalog = self.vm.radix_theme.has_css_catalog();
        let sidebar_bg = token_color_with_fallback(
            &self.vm.radix_theme,
            &self.vm.global_overrides,
            "sidebar",
            chrome.panel_background,
        );
        let mut body = div().flex().flex_col().w_full().gap(px(10.0)).child(self.theme_selector.clone());

        if !has_catalog {
            body = body.child(
                div()
                    .text_size(px(11.0))
                    .text_color(chrome.muted_text)
                    .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
            );
        }

        body = body.child(div().w_full().child(self.token_accordion.clone()));

        div()
            .id("theme-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .p(px(12.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(chrome.title_text)
                    .mb(px(8.0))
                    .child("Theme Tokens"),
            )
            .child(
                div()
                    .id("theme-studio-sidebar-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_y_scroll()
                    .child(body),
            )
    }
}

fn theme_selector_items() -> Vec<SelectorItem> {
    let mut items = vec![SelectorItem::new("default").label("Default")];
    for name in available_theme_names() {
        items.push(SelectorItem::new(name.clone()).label(name));
    }
    items
}
