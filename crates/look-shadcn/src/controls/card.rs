//! Card surface resolved from shadcn `card` tokens.

use gpui_luma::controls::card::CardAppearance;
use gpui_luma::theme::ControlSize;

use crate::appearance_context::AppearanceContext;
use crate::look::ShadcnLook;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::stylesheet::{embedded_stylesheet, find_card_color_rule, resolve_card_color_rule};
use crate::tokens::{ShadcnFont, ShadcnRadius, ShadcnShadow, ShadcnTextRole, ShadcnTextSize};

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
    let ctx = AppearanceContext::new(tokens.as_ref(), theme.mode(), Default::default());
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

pub fn card_appearance(theme: &ShadcnLook, size: ControlSize) -> CardAppearance {
    let tokens = theme.mode_tokens();
    let colors = resolve_card_colors(theme).unwrap_or_else(|_| CardColorTable::fallback());
    let metrics = &tokens.metrics;
    let typography = &tokens.typography;

    CardAppearance {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        title_color: colors.foreground.hsla(),
        description_color: colors.muted_foreground.hsla(),
        body_color: colors.foreground.hsla(),
        shadow: theme.shadow(ShadcnShadow::Default),
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

#[cfg(test)]
mod tests {
    use gpui_luma::theme::ControlSize;

    use crate::look::ShadcnLook;

    use super::card_appearance;

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
            }",
        )
        .expect("look")
    }

    #[test]
    fn card_uses_card_tokens() {
        let look = sample_look();
        let appearance = card_appearance(&look, ControlSize::Md);

        assert_eq!(appearance.background, look.color(crate::tokens::ShadcnToken::Card));
        assert_eq!(appearance.title_color, look.color(crate::tokens::ShadcnToken::CardForeground));
    }
}
