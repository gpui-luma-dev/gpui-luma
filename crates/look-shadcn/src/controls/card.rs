//! Shadcn card surface: a look-layer styled container, not an SDK control.

use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, BoxShadow, Context, Div, Entity, Hsla, IntoElement, Render, SharedString, Stateful,
    Window, div, prelude::*, px,
};
use gpui_luma::theme::{ControlSize, LumaTextStyle, observe_theme_revision};

use crate::look::ShadcnLook;
use crate::look_context::LookContext;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, embedded_stylesheet, find_card_color_rule, find_card_elevation_rule, resolve_card_color_rule,
    resolve_stylesheet_shadow_token,
};
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnTextRole, ShadcnTextSize};

#[derive(Clone, Debug)]
pub struct CardColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub muted_foreground: ResolvedColor,
    pub border: ResolvedColor,
}

impl CardColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
            foreground: ResolvedColor::fallback_foreground(),
            muted_foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_card_colors(theme: &ShadcnLook) -> anyhow::Result<CardColorTable> {
    resolve_card_colors_with_stylesheet(theme, embedded_stylesheet())
}

pub fn resolve_card_colors_with_stylesheet(
    theme: &ShadcnLook,
    stylesheet: &crate::stylesheet::StylesheetConfig,
) -> anyhow::Result<CardColorTable> {
    let tokens = theme.mode_tokens();
    let ctx = LookContext::new(tokens.as_ref(), theme.mode(), Default::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "card");
    let rule = find_card_color_rule(stylesheet).ok_or_else(|| anyhow::anyhow!("no card color rule configured"))?;
    let colors = resolve_card_color_rule(&resolver, rule)?;
    Ok(CardColorTable {
        background: colors.background,
        foreground: colors.foreground,
        muted_foreground: colors.muted_foreground,
        border: colors.border,
    })
}

#[derive(Clone, Debug)]
pub struct CardLook {
    pub background: Hsla,
    pub border: Hsla,
    pub title_color: Hsla,
    pub description_color: Hsla,
    pub body_color: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub section_gap: f32,
    pub header_gap: f32,
    pub body_gap: f32,
    pub title: LumaTextStyle,
    pub description: LumaTextStyle,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

pub fn card_look(theme: &ShadcnLook, size: ControlSize) -> CardLook {
    let tokens = theme.mode_tokens();
    let colors = resolve_card_colors(theme).unwrap_or_else(|_| CardColorTable::fallback());
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;
    let shadow = card_elevation_shadow(&tokens.catalog, embedded_stylesheet());

    CardLook {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        title_color: colors.foreground.hsla(),
        description_color: colors.muted_foreground.hsla(),
        body_color: colors.foreground.hsla(),
        shadow,
        radius: theme.radius(ShadcnRadius::Lg),
        padding: metrics.padding_x(size),
        section_gap: metrics.gap(size),
        header_gap: (metrics.gap(size) * 0.5).max(2.0),
        body_gap: metrics.gap(size),
        title: match size {
            ControlSize::Sm => typography.text.label,
            ControlSize::Md => theme.typography_role(ShadcnTextRole::H4),
            ControlSize::Lg => theme.typography_scale(ShadcnTextSize::Xl),
        },
        description: typography.text.caption,
        body: typography.text.body,
        font_family: theme.font(ShadcnFont::Sans),
    }
}

fn card_elevation_shadow(catalog: &crate::catalog::CssTokenMap, stylesheet: &StylesheetConfig) -> Vec<BoxShadow> {
    let Some(rule) = find_card_elevation_rule(stylesheet) else {
        return Vec::new();
    };
    let Some(token) = resolve_stylesheet_shadow_token(&rule.shadow) else {
        return Vec::new();
    };
    parse_shadow_token(catalog, &token).unwrap_or_default()
}

pub type CardElementRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone)]
struct ShadcnCardConfig {
    id: SharedString,
    size: ControlSize,
    title: Option<SharedString>,
    description: Option<SharedString>,
    header: Option<CardElementRenderer>,
    body: Vec<CardElementRenderer>,
    footer: Option<CardElementRenderer>,
    elevated: bool,
    full_height: bool,
    body_fill: bool,
}

pub struct ShadcnCardBuilder {
    look: Arc<ShadcnLook>,
    config: ShadcnCardConfig,
}

impl ShadcnCardBuilder {
    pub fn new(look: Arc<ShadcnLook>, id: impl Into<SharedString>) -> Self {
        Self {
            look,
            config: ShadcnCardConfig {
                id: id.into(),
                size: ControlSize::Md,
                title: None,
                description: None,
                header: None,
                body: Vec::new(),
                footer: None,
                elevated: true,
                full_height: false,
                body_fill: false,
            },
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.config.size = size;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    pub fn header(mut self, header: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.header = Some(Arc::new(header));
        self
    }

    pub fn header_element(mut self, header: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.header = Some(renderer_from_element(header));
        self
    }

    pub fn child(mut self, child: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.body.push(renderer_from_element(child));
        self
    }

    pub fn child_render(mut self, child: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.body.push(Arc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CardElementRenderer>) -> Self {
        self.config.body.extend(children);
        self
    }

    pub fn footer(mut self, footer: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.config.footer = Some(Arc::new(footer));
        self
    }

    pub fn footer_element(mut self, footer: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.config.footer = Some(renderer_from_element(footer));
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.config.elevated = elevated;
        self
    }

    pub fn full_height(mut self, full_height: bool) -> Self {
        self.config.full_height = full_height;
        self
    }

    pub fn body_fill(mut self, body_fill: bool) -> Self {
        self.config.body_fill = body_fill;
        self
    }

    pub fn render(self, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        render_card(&self.look, &self.config, window, cx)
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ShadcnCard> {
        cx.new(|cx| ShadcnCard::from_config(self.look, self.config, cx))
    }
}

pub struct ShadcnCard {
    look: Arc<ShadcnLook>,
    config: ShadcnCardConfig,
}

impl ShadcnCard {
    fn from_config(look: Arc<ShadcnLook>, config: ShadcnCardConfig, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { look, config }
    }
}

impl Render for ShadcnCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_card(&self.look, &self.config, window, cx)
    }
}

fn render_card(look: &ShadcnLook, config: &ShadcnCardConfig, window: &mut Window, cx: &mut App) -> Stateful<Div> {
    let card_look = card_look(look, config.size);

    let mut root = div()
        .id(config.id.clone())
        .relative()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(card_look.section_gap))
        .rounded(px(card_look.radius))
        .border_1()
        .border_color(card_look.border)
        .overflow_hidden()
        .bg(card_look.background)
        .p(px(card_look.padding))
        .text_color(card_look.body_color)
        .font_family(card_look.font_family.clone())
        .text_size(px(card_look.body.size))
        .line_height(px(card_look.body.line_height))
        .font_weight(card_look.body.weight);

    if config.elevated {
        root = root.shadow(card_look.shadow.clone());
    }

    if config.full_height {
        root = root.h_full().min_h(px(0.0));
    }

    if let Some(header) = &config.header {
        root = root.child(header(window, cx));
    } else if config.title.is_some() || config.description.is_some() {
        let mut header = div().w_full().flex().flex_col().gap(px(card_look.header_gap));

        if let Some(title) = &config.title {
            header = header.child(
                div()
                    .text_size(px(card_look.title.size))
                    .line_height(px(card_look.title.line_height))
                    .font_weight(card_look.title.weight)
                    .text_color(card_look.title_color)
                    .child(title.clone()),
            );
        }

        if let Some(description) = &config.description {
            header = header.child(
                div()
                    .text_size(px(card_look.description.size))
                    .line_height(px(card_look.description.line_height))
                    .font_weight(card_look.description.weight)
                    .text_color(card_look.description_color)
                    .child(description.clone()),
            );
        }

        root = root.child(header);
    }

    if !config.body.is_empty() {
        let mut body = div().w_full().flex().flex_col().gap(px(card_look.body_gap));

        if config.body_fill {
            body = body.flex_1().min_h(px(0.0));
        }

        for child in &config.body {
            body = body.child(child(window, cx));
        }

        root = root.child(body);
    }

    if let Some(footer) = &config.footer {
        root = root.child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_end()
                .text_color(card_look.body_color)
                .child(footer(window, cx)),
        );
    }

    root
}

fn renderer_from_element(element: impl IntoElement + Clone + Send + Sync + 'static) -> CardElementRenderer {
    Arc::new(move |_, _| element.clone().into_any_element())
}

#[cfg(test)]
mod tests {
    use gpui_luma::theme::ControlSize;

    use crate::look::ShadcnLook;
    use crate::tokens::ShadcnToken;

    use super::card_look;

    fn sample_look() -> ShadcnLook {
        ShadcnLook::from_css_str(
            ":root { \
                --background: oklch(0.9735 0.0261 90.0953); \
                --foreground: oklch(0.3092 0.0518 219.6516); \
                --card: oklch(0.9306 0.0260 92.4020); \
                --card-foreground: oklch(0.3092 0.0518 219.6516); \
                --muted: oklch(0.90 0.01 220); \
                --muted-foreground: oklch(0.45 0.02 220); \
                --primary: oklch(0.5924 0.2025 355.8943); \
                --primary-foreground: oklch(1 0 0); \
                --secondary: oklch(0.6437 0.1019 187.3840); \
                --secondary-foreground: oklch(1 0 0); \
                --accent: oklch(0.90 0.05 250); \
                --accent-foreground: oklch(0.20 0.05 250); \
                --border: oklch(0.6537 0.0197 205.2618); \
                --input: oklch(0.6537 0.0197 205.2618); \
                --ring: oklch(0.5924 0.2025 355.8943); \
                --shadow-xs: 0 1px 3px 0px hsl(0 0% 0% / 0.05); \
                --shadow: 0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10); \
            } \
            .dark { \
                --background: oklch(0.205 0 0); \
                --foreground: oklch(0.985 0 0); \
                --card: oklch(0.205 0 0); \
                --card-foreground: oklch(0.985 0 0); \
                --muted: oklch(0.30 0 0); \
                --muted-foreground: oklch(0.75 0 0); \
                --primary: oklch(0.5924 0.2025 355.8943); \
                --primary-foreground: oklch(1 0 0); \
                --secondary: oklch(0.6437 0.1019 187.3840); \
                --secondary-foreground: oklch(1 0 0); \
                --accent: oklch(0.90 0.05 250); \
                --accent-foreground: oklch(0.20 0.05 250); \
                --border: oklch(0.40 0 0); \
                --input: oklch(0.40 0 0); \
                --ring: oklch(0.5924 0.2025 355.8943); \
                --shadow-xs: 0 1px 3px 0px hsl(0 0% 0% / 0.05); \
                --shadow: 0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10); \
            }",
        )
        .expect("look")
    }

    #[test]
    fn card_resolves_stylesheet_shadow() {
        let look = card_look(&sample_look(), ControlSize::Md);
        assert!(!look.shadow.is_empty());
    }

    #[test]
    fn card_uses_card_tokens() {
        let shadcn = sample_look();
        let look = card_look(&shadcn, ControlSize::Md);

        assert_eq!(look.background, shadcn.color(ShadcnToken::Card));
        assert_eq!(look.title_color, shadcn.color(ShadcnToken::CardForeground));
    }
}
