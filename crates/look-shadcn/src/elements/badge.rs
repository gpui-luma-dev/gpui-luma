use std::sync::Arc;

use gpui::{Div, FontWeight, IntoElement, SharedString, div, px, prelude::*};
use gpui_luma::controls::icon::{IconSource, lucide_icon};
use gpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::look::ShadcnLook;
use crate::provenance::{ColorSource, LookResolver, ResolvedColor};
use crate::stylesheet::{embedded_stylesheet, find_badge_color_rule, resolve_badge_color_rule};
use crate::tokens::{ShadcnFont, ShadcnToken};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Ghost,
}

impl BadgeVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BadgeIconPlacement {
    #[default]
    Start,
    End,
}

#[derive(Clone)]
pub struct Badge {
    look: Arc<ShadcnLook>,
    label: SharedString,
    variant: BadgeVariant,
    size: ControlSize,
    icon: Option<IconSource>,
    icon_placement: BadgeIconPlacement,
}

#[derive(Clone, Debug)]
pub struct BadgeColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct BadgeLook {
    pub background: gpui::Hsla,
    pub foreground: gpui::Hsla,
    pub border: Option<gpui::Hsla>,
    pub radius: f32,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

impl BadgeColorTable {
    pub fn fallback(look: &ShadcnLook, variant: BadgeVariant) -> Self {
        let (background, foreground, border) = match variant {
            BadgeVariant::Default => (
                ResolvedColor {
                    value: look.color(ShadcnToken::Primary),
                    source: ColorSource::Derived { note: "badge fallback primary".into() },
                },
                ResolvedColor {
                    value: look.color(ShadcnToken::PrimaryForeground),
                    source: ColorSource::Derived { note: "badge fallback primary-foreground".into() },
                },
                None,
            ),
            BadgeVariant::Secondary => (
                ResolvedColor {
                    value: look.color(ShadcnToken::Secondary),
                    source: ColorSource::Derived { note: "badge fallback secondary".into() },
                },
                ResolvedColor {
                    value: look.color(ShadcnToken::SecondaryForeground),
                    source: ColorSource::Derived { note: "badge fallback secondary-foreground".into() },
                },
                None,
            ),
            BadgeVariant::Outline => (
                ResolvedColor::transparent(),
                ResolvedColor {
                    value: look.color(ShadcnToken::Foreground),
                    source: ColorSource::Derived { note: "badge fallback foreground".into() },
                },
                Some(ResolvedColor {
                    value: look.color(ShadcnToken::Border),
                    source: ColorSource::Derived { note: "badge fallback border".into() },
                }),
            ),
            BadgeVariant::Ghost => (
                ResolvedColor::transparent(),
                ResolvedColor {
                    value: look.color(ShadcnToken::Foreground),
                    source: ColorSource::Derived { note: "badge fallback foreground".into() },
                },
                None,
            ),
        };

        Self { background, foreground, border }
    }
}

impl Badge {
    pub fn new(look: Arc<ShadcnLook>, label: impl Into<SharedString>) -> Self {
        Self {
            look,
            label: label.into(),
            variant: BadgeVariant::Default,
            size: ControlSize::Md,
            icon: None,
            icon_placement: BadgeIconPlacement::Start,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: impl Into<IconSource>, placement: BadgeIconPlacement) -> Self {
        self.icon = Some(icon.into());
        self.icon_placement = placement;
        self
    }

    pub fn start_icon(self, icon: impl Into<IconSource>) -> Self {
        self.icon(icon, BadgeIconPlacement::Start)
    }

    pub fn end_icon(self, icon: impl Into<IconSource>) -> Self {
        self.icon(icon, BadgeIconPlacement::End)
    }
}

pub fn resolve_badge_colors(theme: &ShadcnLook, variant: BadgeVariant) -> anyhow::Result<BadgeColorTable> {
    resolve_badge_colors_with_stylesheet(theme, embedded_stylesheet(), variant)
}

pub fn resolve_badge_colors_with_stylesheet(
    theme: &ShadcnLook,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    variant: BadgeVariant,
) -> anyhow::Result<BadgeColorTable> {
    let tokens = theme.mode_tokens();
    let resolver = LookResolver::new(&tokens.catalog, theme.mode(), "badge");
    let rule = find_badge_color_rule(stylesheet, variant, theme.mode())
        .ok_or_else(|| anyhow::anyhow!("no matching badge color rule"))?;
    let colors = resolve_badge_color_rule(&resolver, rule)?;

    Ok(BadgeColorTable { background: colors.background, foreground: colors.foreground, border: colors.border })
}

pub fn badge_look(theme: &ShadcnLook, variant: BadgeVariant, size: ControlSize) -> BadgeLook {
    let tokens = theme.mode_tokens();
    let metrics = &tokens.metrics;
    let typography = badge_typography(tokens.as_ref(), size);
    let min_height = typography.line_height + metrics.padding_y(size);
    let colors = resolve_badge_colors(theme, variant).unwrap_or_else(|_| BadgeColorTable::fallback(theme, variant));

    BadgeLook {
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.map(|border| border.hsla()),
        radius: tokens.metrics.radius.pill,
        min_height,
        padding_x: metrics.padding_x(size),
        padding_y: ((min_height - typography.line_height) * 0.5).max(0.0),
        gap: (metrics.gap(size) * 0.75).max(4.0),
        icon_size: typography.size,
        typography,
        font_family: theme.font(ShadcnFont::Sans),
    }
}

impl IntoElement for Badge {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let look = badge_look(&self.look, self.variant, self.size);
        let mut root = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(look.gap))
            .min_h(px(look.min_height))
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .rounded(px(look.radius))
            .bg(look.background)
            .text_color(look.foreground)
            .font_family(look.font_family)
            .font_weight(look.typography.weight)
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .when_some(look.border, |root, border| root.border_1().border_color(border));

        if let Some(icon) = self.icon.as_ref() {
            let rendered = render_badge_icon(icon, look.foreground, look.icon_size);
            if self.icon_placement == BadgeIconPlacement::Start {
                root = root.child(rendered).child(self.label);
            } else {
                root = root.child(self.label).child(rendered);
            }
        } else {
            root = root.child(self.label);
        }

        root
    }
}

fn badge_typography(tokens: &crate::mode::ShadcnModeTokens, size: ControlSize) -> LumaTextStyle {
    match size {
        ControlSize::Sm => tokens.typography.text.caption,
        ControlSize::Md => tokens.typography.text.label,
        ControlSize::Lg => LumaTextStyle {
            size: tokens.typography.text.body.size,
            line_height: tokens.typography.text.label.line_height,
            weight: FontWeight::MEDIUM,
        },
    }
}

fn render_badge_icon(icon: &IconSource, color: gpui::Hsla, size: f32) -> gpui::AnyElement {
    if let Some(icon) = icon.lucide() {
        return lucide_icon(icon, color, size);
    }

    if let Some(path) = icon.svg_path() {
        return gpui::svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element();
    }

    div().size(px(size)).into_any_element()
}

impl ShadcnLook {
    pub fn badge(&self, label: impl Into<SharedString>) -> Badge {
        Badge::new(Arc::new(self.clone()), label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_look() -> ShadcnLook {
        ShadcnLook::native()
    }

    #[test]
    fn default_badge_uses_primary_tokens() {
        let shadcn = sample_look();
        let look = badge_look(&shadcn, BadgeVariant::Default, ControlSize::Md);

        assert_eq!(look.background, shadcn.color(ShadcnToken::Primary));
        assert_eq!(look.foreground, shadcn.color(ShadcnToken::PrimaryForeground));
    }

    #[test]
    fn outline_badge_uses_border_in_dark_mode() {
        let shadcn = sample_look();
        shadcn.set_mode(gpui_luma::theme::ThemeMode::Dark);
        let look = badge_look(&shadcn, BadgeVariant::Outline, ControlSize::Md);

        assert_eq!(look.foreground, shadcn.color(ShadcnToken::Foreground));
        assert_eq!(look.border, Some(shadcn.token_color("input").expect("input")));
    }

    #[test]
    fn ghost_badge_is_transparent() {
        let shadcn = sample_look();
        let look = badge_look(&shadcn, BadgeVariant::Ghost, ControlSize::Md);

        assert_eq!(look.background.a, 0.0);
        assert!(look.border.is_none());
    }
}
