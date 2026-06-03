use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::textfield::{TextField, TextFieldEvent};
use gpui_luma::theme::{RadixTheme, RadixThemeControlExt};
use gpui_luma::{hstack, vstack};

use gpui::Hsla;

use crate::studio::app::ThemeStudioApp;
use crate::studio::export::{catalog_color_for_token, token_css_name};
use crate::studio::overrides::StudioOverrides;
use crate::studio::panels::{format_hex_color, parse_hex_color};
use crate::theme::available_theme_names;

const TOKEN_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("BASE", &[("background", "Background"), ("foreground", "Foreground")]),
    ("PRIMARY", &[("primary", "Primary"), ("primary-foreground", "Primary Foreground")]),
    ("SECONDARY", &[("secondary", "Secondary"), ("secondary-foreground", "Secondary Foreground")]),
    ("ACCENT", &[("accent", "Accent"), ("accent-foreground", "Accent Foreground")]),
    ("CARD", &[("card", "Card"), ("card-foreground", "Card Foreground")]),
    ("POPOVER", &[("popover", "Popover"), ("popover-foreground", "Popover Foreground")]),
    ("MUTED", &[("muted", "Muted"), ("muted-foreground", "Muted Foreground")]),
    (
        "DESTRUCTIVE",
        &[("destructive", "Destructive"), ("destructive-foreground", "Destructive Foreground")],
    ),
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
            ("sidebar", "Sidebar"),
            ("sidebar-foreground", "Sidebar Foreground"),
            ("sidebar-primary", "Sidebar Primary"),
            ("sidebar-primary-foreground", "Sidebar Primary Foreground"),
            ("sidebar-accent", "Sidebar Accent"),
            ("sidebar-accent-foreground", "Sidebar Accent Foreground"),
            ("sidebar-border", "Sidebar Border"),
            ("sidebar-ring", "Sidebar Ring"),
        ],
    ),
];

/// Shared state for accordion content closures (no `Context` in `AccordionContent::custom`).
struct TokenAccordionState {
    theme: Arc<RwLock<Arc<RadixTheme>>>,
    overrides: Arc<RwLock<HashMap<String, Hsla>>>,
    fields: HashMap<String, TextField>,
}

pub struct ThemeSidebar {
    pub(crate) radix_theme: Arc<RadixTheme>,
    global_overrides: HashMap<String, Hsla>,
    theme_selector: Entity<Selector>,
    token_fields: HashMap<String, TextField>,
    token_accordion: Entity<AccordionControl>,
    accordion_state: Arc<TokenAccordionState>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<ThemeStudioApp>,
        radix_theme: Arc<RadixTheme>,
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

                let field = radix_theme.textfield(format!("theme-studio-token-{token}")).value(initial).spawn(cx);

                token_fields.insert(token.to_string(), field);
            }
        }

        let accordion_state = Arc::new(TokenAccordionState {
            theme: Arc::new(RwLock::new(radix_theme.clone())),
            overrides: Arc::new(RwLock::new(global_overrides.clone())),
            fields: token_fields.clone(),
        });

        let mut accordion_builder = radix_theme
            .accordion("theme-studio-token-accordion")
            .multiple()
            .template(radix_theme.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let state = accordion_state.clone();
            let items = *tokens;
            accordion_builder = accordion_builder.item(AccordionItem::new(
                category_item_id(category),
                AccordionTrigger::new(*category),
                AccordionContent::custom(move || category_token_content(state.clone(), items).into_any_element()),
            ));
        }

        let token_accordion = accordion_builder.spawn(cx);

        Self {
            radix_theme,
            global_overrides,
            theme_selector,
            token_fields,
            token_accordion,
            accordion_state,
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
                let field = sidebar.read(cx).token_fields.get(*token).expect("token field").clone();
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
        radix_theme: Arc<RadixTheme>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        self.radix_theme = radix_theme;
        self.global_overrides = overrides.global_color_overrides.clone();
        self.write_accordion_state();
        let theme = self.radix_theme.clone();
        self.sync_token_fields_from(&theme, overrides, cx);
        self.notify_token_accordion(cx);
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        self.global_overrides = overrides.global_color_overrides.clone();
        self.write_accordion_state();
        let theme = self.radix_theme.clone();
        self.sync_token_fields_from(&theme, overrides, cx);
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
                let Some(field) = self.token_fields.get(*token) else {
                    continue;
                };
                let value = token_hex_value(radix_theme, &overrides.global_color_overrides, token);
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    fn write_accordion_state(&self) {
        *self.accordion_state.theme.write().expect("theme lock") = self.radix_theme.clone();
        *self.accordion_state.overrides.write().expect("overrides lock") = self.global_overrides.clone();
    }

    fn notify_token_accordion(&self, cx: &mut Context<Self>) {
        self.token_accordion.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

fn category_item_id(category: &str) -> String {
    format!("token-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

fn category_token_content(state: Arc<TokenAccordionState>, tokens: &[(&str, &str)]) -> impl IntoElement {
    let theme = state.theme.read().expect("theme lock").clone();
    let overrides = state.overrides.read().expect("overrides lock").clone();
    let chrome = theme.chrome();

    let mut rows = vstack! {
        gap=8 align=start;
    };

    for (token, label) in tokens {
        let Some(field) = state.fields.get(*token) else {
            continue;
        };
        let color = effective_token_color(&theme, &overrides, token);
        let label = label.to_string();
        let field = field.clone();

        rows = vstack! {
            gap=8 align=start;
            rows,
            vstack! {
                gap=4 align=start;
                div().text_size(px(11.0)).text_color(chrome.body_text).child(label),
                hstack! {
                    gap=8 align=center;
                    div()
                        .size(px(16.0))
                        .flex_shrink_0()
                        .rounded(px(4.0))
                        .bg(color)
                        .border_1()
                        .border_color(chrome.border),
                    div().flex_1().min_w(px(0.0)).child(field),
                },
            },
        };
    }

    rows
}

fn token_hex_value(radix_theme: &RadixTheme, global_overrides: &HashMap<String, Hsla>, token: &str) -> String {
    format_hex_color(effective_token_color(radix_theme, global_overrides, token))
}

fn effective_token_color(radix_theme: &RadixTheme, global_overrides: &HashMap<String, Hsla>, token: &str) -> Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| catalog_color_for_token(radix_theme, token))
        .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.5, 1.0))
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let has_catalog = self.radix_theme.has_css_catalog();

        let mut body = div().flex().flex_col().gap(px(10.0)).child(self.theme_selector.clone());

        if !has_catalog {
            body = body.child(
                div()
                    .text_size(px(11.0))
                    .text_color(chrome.muted_text)
                    .child("Load a named tweakcn theme to edit color tokens."),
            );
        } else {
            body = body.child(self.token_accordion.clone());
        }

        div()
            .id("theme-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(chrome.panel_background)
            .border_r_1()
            .border_color(chrome.border)
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
