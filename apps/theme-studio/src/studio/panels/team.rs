use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorItem, SelectorPalette, SelectorTheme, ThemedSelectorTemplate};
use gpui_luma::controls::selector_panel::default_selector_items_template;
use gpui_luma::theme::radix::prelude::*;
use gpui_luma::theme::{InteractionState, MetricTokens, RadixTheme};

use super::common::{avatar_circle, card, card_header};

/// Compact role dropdowns (~half of default 180px menu/trigger min width).
const ROLE_SELECTOR_MIN_WIDTH: f32 = 90.0;
const ROLE_SELECTOR_WIDTH: f32 = ROLE_SELECTOR_MIN_WIDTH;
const TEAM_CARD_WIDTH: f32 = 380.0;

struct CompactSelectorTheme {
    inner: Arc<dyn SelectorTheme>,
    panel_min_width: f32,
}

impl SelectorTheme for CompactSelectorTheme {
    fn resolve(&self, state: InteractionState) -> SelectorPalette {
        let mut palette = self.inner.resolve(state);
        palette.items_panel.min_width = self.panel_min_width;
        palette
    }

    fn metrics(&self) -> &MetricTokens {
        self.inner.metrics()
    }
}

pub struct TeamPanel {
    radix_theme: Arc<RadixTheme>,
    member_selectors: [Entity<Selector>; 3],
}

impl TeamPanel {
    pub fn new(cx: &mut Context<Self>, radix_theme: Arc<RadixTheme>) -> Self {
        let roles = role_items();
        Self {
            member_selectors: [
                member_selector(cx, &radix_theme, "team-sofia", "Owner", &roles),
                member_selector(cx, &radix_theme, "team-jackson", "Developer", &roles),
                member_selector(cx, &radix_theme, "team-isabella", "Billing", &roles),
            ],
            radix_theme,
        }
    }
}

impl Render for TeamPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let avatar_bg = gpui::hsla(0.55, 0.12, 0.35, 1.0);

        card(
            TEAM_CARD_WIDTH,
            chrome.border,
            chrome.panel_background,
            div()
                .w_full()
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(card_header(
                    "Team Members",
                    "Invite your team members to collaborate.",
                    chrome.title_text,
                    chrome.muted_text,
                ))
                .child(member_row("SD", "Sofia Davis", "m@example.com", &self.member_selectors[0], chrome, avatar_bg))
                .child(member_row("JL", "Jackson Lee", "m@example.com", &self.member_selectors[1], chrome, avatar_bg))
                .child(member_row(
                    "IN",
                    "Isabella Nguyen",
                    "m@example.com",
                    &self.member_selectors[2],
                    chrome,
                    avatar_bg,
                )),
        )
    }
}

fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
    selector: &Entity<Selector>,
    chrome: gpui_luma::theme::LumaChrome,
    avatar_bg: gpui::Hsla,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .py(px(6.0))
        .overflow_hidden()
        .child(avatar_circle(initials, 32.0, avatar_bg, chrome.title_text))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .overflow_hidden()
                        .child(name),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(14.0))
                        .text_color(chrome.muted_text)
                        .overflow_hidden()
                        .child(email),
                ),
        )
        .child(div().flex_none().w(px(ROLE_SELECTOR_WIDTH)).child(selector.clone()))
}

fn role_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("owner").label("Owner"),
        SelectorItem::new("developer").label("Developer"),
        SelectorItem::new("billing").label("Billing"),
    ]
}

fn member_selector(
    cx: &mut Context<TeamPanel>,
    theme: &Arc<RadixTheme>,
    id: &'static str,
    default_role: &'static str,
    items: &[SelectorItem],
) -> Entity<Selector> {
    let template = Arc::new(ThemedSelectorTemplate::new(
        Arc::new(CompactSelectorTheme { inner: theme.selector_theme(), panel_min_width: ROLE_SELECTOR_MIN_WIDTH }),
        default_selector_items_template(),
    ));
    theme.selector(id).label(default_role).items(items.to_vec()).template(template).spawn(cx)
}
