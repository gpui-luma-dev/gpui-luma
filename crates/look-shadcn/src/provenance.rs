//! Provenance types for inspect-path color resolution. Paint-path code uses [`ResolvedColor::hsla`] only.

use gpui::{Hsla, hsla};

use gpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::catalog::CssTokenMap;
use crate::color::with_alpha;
use crate::state_color::{algorithmic_state_color, catalog_state_color};

#[derive(Clone, Debug)]
pub struct ResolvedColor {
    pub value: Hsla,
    pub source: ColorSource,
}

#[derive(Clone, Debug)]
pub struct ResolvedMetric {
    pub value_px: f32,
    pub source: MetricSource,
}

#[derive(Clone, Debug)]
pub enum MetricSource {
    CssVar { token: String },
    Derived { note: String },
    Scaffold { path: String },
    Constant { label: String },
}

#[derive(Clone, Debug)]
pub enum ColorSource {
    Transparent,
    CssVar { token: String },
    TokenAlpha { token: String, alpha_percent: u8 },
    StateVar { base_token: String, layer: InteractionLayer },
    Algorithmic { base_token: String, layer: InteractionLayer },
    Derived { note: String },
}

#[expect(dead_code)]
#[derive(Clone, Debug)]
pub struct TableRuleMetadata {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub struct LookResolver<'a> {
    catalog: &'a CssTokenMap,
    theme_mode: ThemeMode,
    #[allow(dead_code)]
    label: &'a str,
}

impl<'a> LookResolver<'a> {
    pub fn new(catalog: &'a CssTokenMap, theme_mode: ThemeMode, label: &'a str) -> Self {
        Self { catalog, theme_mode, label }
    }

    pub fn resolve_decl(&self, decl: &str) -> anyhow::Result<ResolvedColor> {
        let decl = decl.trim().strip_prefix("--").unwrap_or(decl.trim());

        if decl.eq_ignore_ascii_case("transparent") {
            return Ok(ResolvedColor::transparent());
        }

        if let Some((base, alpha_percent)) = parse_slash_alpha(decl)? {
            let base_color = self.catalog.color(base)?;
            return Ok(ResolvedColor {
                value: with_alpha(base_color, f32::from(alpha_percent) / 100.0),
                source: ColorSource::TokenAlpha { token: base.to_string(), alpha_percent },
            });
        }

        if let Ok(value) = self.catalog.color(decl) {
            return Ok(ResolvedColor { value, source: ColorSource::CssVar { token: decl.to_string() } });
        }

        if let Some((base, layer)) = parse_state_suffix(decl) {
            let base_color = self.catalog.color(base)?;
            let filled = is_filled_token(base);
            if let Some(value) = catalog_state_color(self.catalog, base, layer) {
                return Ok(ResolvedColor {
                    value,
                    source: ColorSource::StateVar { base_token: base.to_string(), layer },
                });
            }
            return Ok(ResolvedColor {
                value: algorithmic_state_color(base_color, layer, self.theme_mode, filled),
                source: ColorSource::Algorithmic { base_token: base.to_string(), layer },
            });
        }

        anyhow::bail!("unknown color declaration `{decl}`")
    }
}

/// Full CSS custom-property form, e.g. `--accent/50`.
pub fn format_color_source(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => format!("--{token}"),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("--{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => {
            format!("--{}", state_var_token(base_token, *layer))
        }
        ColorSource::Algorithmic { base_token, layer } => {
            format!("--{} · algorithmic · {}", base_token, layer_label(*layer))
        }
        ColorSource::Derived { note } => note.clone(),
    }
}

/// CSS custom property that exists in the loaded theme file (grep target for the inspector).
pub fn format_inspect_css_key(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => format!("--{token}"),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("--{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => {
            format!("--{}", state_var_token(base_token, *layer))
        }
        ColorSource::Algorithmic { base_token, .. } => format!("--{base_token}"),
        ColorSource::Derived { note } => note.trim_start_matches("= ").to_string(),
    }
}

/// Primary source label for metric inspector rows (CSS token, scaffold path, or derivation).
pub fn format_inspect_metric_source(source: &MetricSource) -> String {
    match source {
        MetricSource::CssVar { token } => format!("--{token}"),
        MetricSource::Scaffold { path } => format!("scaffold · {path}"),
        MetricSource::Constant { label } => format!("constant · {label}"),
        MetricSource::Derived { note } => metric_derived_css_key(note).unwrap_or_else(|| note.clone()),
    }
}

/// Secondary provenance line for metrics when the source line alone is ambiguous.
pub fn format_inspect_metric_provenance(source: &MetricSource) -> Option<String> {
    match source {
        MetricSource::Derived { note } => Some(note.clone()),
        MetricSource::CssVar { token } => Some(format!("catalog · --{token}")),
        MetricSource::Scaffold { .. } | MetricSource::Constant { .. } => None,
    }
}

pub fn format_metric_px(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}px", value.round() as i32)
    } else {
        format!("{value}px")
    }
}

fn metric_derived_css_key(note: &str) -> Option<String> {
    note.split_whitespace()
        .find(|word| word.starts_with("--"))
        .map(|token| token.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string())
}

/// How the resolved color relates to the css key (algorithmic state, inheritance, etc.).
pub fn format_inspect_provenance(source: &ColorSource) -> Option<String> {
    match source {
        ColorSource::Algorithmic { layer, .. } => Some(format!("algorithmic · {}", layer_label(*layer))),
        ColorSource::Derived { note } if note.starts_with("= ") => {
            Some(format!("inherits · {}", note.trim_start_matches("= ")))
        }
        ColorSource::Derived { note } => Some(note.clone()),
        ColorSource::StateVar { layer, .. } => Some(format!("catalog · {}", layer_label(*layer))),
        _ => None,
    }
}

/// Shadcn/Tailwind-style token reference for cross-checking theme CSS, e.g. `accent/50` or `primary-hover`.
pub fn format_css_style_ref(source: &ColorSource) -> String {
    match source {
        ColorSource::Transparent => "transparent".to_string(),
        ColorSource::CssVar { token } => token.clone(),
        ColorSource::TokenAlpha { token, alpha_percent } => format!("{token}/{alpha_percent}"),
        ColorSource::StateVar { base_token, layer } => state_var_token(base_token, *layer),
        ColorSource::Algorithmic { base_token, layer } => {
            format!("{} ({})", state_var_token(base_token, *layer), layer_label(*layer))
        }
        ColorSource::Derived { note } => note.clone(),
    }
}

fn state_var_token(base_token: &str, layer: InteractionLayer) -> String {
    match layer {
        InteractionLayer::Default => base_token.to_string(),
        InteractionLayer::Hovered => format!("{base_token}-hover"),
        InteractionLayer::Pressed => format!("{base_token}-pressed"),
        InteractionLayer::Disabled => format!("{base_token}-disabled"),
    }
}

fn layer_label(layer: InteractionLayer) -> &'static str {
    match layer {
        InteractionLayer::Default => "default",
        InteractionLayer::Hovered => "hover",
        InteractionLayer::Pressed => "pressed",
        InteractionLayer::Disabled => "disabled",
    }
}

impl ResolvedColor {
    pub fn transparent() -> Self {
        Self { value: hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
    }

    pub fn fallback_foreground() -> Self {
        Self { value: hsla(0.0, 0.0, 0.0, 1.0), source: ColorSource::CssVar { token: "foreground".to_string() } }
    }

    pub fn hsla(&self) -> Hsla {
        self.value
    }

    pub fn into_hsla(self) -> Hsla {
        self.value
    }
}

fn parse_slash_alpha(decl: &str) -> anyhow::Result<Option<(&str, u8)>> {
    let Some((base, alpha_raw)) = decl.split_once('/') else {
        return Ok(None);
    };
    let alpha_raw = alpha_raw.trim().strip_suffix('%').unwrap_or(alpha_raw.trim());
    let alpha = alpha_raw.parse::<f32>().map_err(|_| anyhow::anyhow!("invalid alpha in `{decl}`"))?;
    let alpha_percent = if decl.contains('%') || alpha > 1.0 {
        alpha.round() as u8
    } else {
        (alpha * 100.0).round() as u8
    };
    Ok(Some((base, alpha_percent)))
}

fn parse_state_suffix(decl: &str) -> Option<(&str, InteractionLayer)> {
    const SUFFIXES: [(&str, InteractionLayer); 4] = [
        ("-hover", InteractionLayer::Hovered),
        ("-pressed", InteractionLayer::Pressed),
        ("-active", InteractionLayer::Pressed),
        ("-disabled", InteractionLayer::Disabled),
    ];

    for (suffix, layer) in SUFFIXES {
        if let Some(base) = decl.strip_suffix(suffix) {
            return Some((base, layer));
        }
    }
    None
}

fn is_filled_token(base: &str) -> bool {
    matches!(base, "primary" | "secondary" | "destructive")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui_luma::theme::ThemeMode;

    use super::*;
    use crate::catalog::CssTokenMap;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330 64% 52%)".into()),
            ("primary-hover".into(), "hsl(320 64% 45%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("accent".into(), "hsl(18 80% 44%)".into()),
            ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
            ("foreground".into(), "hsl(192 81% 14%)".into()),
            ("input".into(), "hsl(186 8% 55%)".into()),
            ("muted".into(), "hsl(180 7% 60%)".into()),
            ("muted-foreground".into(), "hsl(192 81% 14%)".into()),
        ]))
    }

    #[test]
    fn format_inspect_css_key_points_at_catalog_token_for_algorithmic_states() {
        assert_eq!(
            format_inspect_css_key(&ColorSource::Algorithmic {
                base_token: "primary".into(),
                layer: InteractionLayer::Pressed,
            }),
            "--primary"
        );
        assert_eq!(
            format_inspect_provenance(&ColorSource::Algorithmic {
                base_token: "primary".into(),
                layer: InteractionLayer::Pressed,
            }),
            Some("algorithmic · pressed".to_string())
        );
        assert_eq!(
            format_inspect_css_key(&ColorSource::StateVar {
                base_token: "primary".into(),
                layer: InteractionLayer::Hovered,
            }),
            "--primary-hover"
        );
        assert_eq!(
            format_inspect_provenance(&ColorSource::StateVar {
                base_token: "primary".into(),
                layer: InteractionLayer::Hovered,
            }),
            Some("catalog · hover".to_string())
        );
        assert_eq!(format_inspect_css_key(&ColorSource::Derived { note: "= --primary".into() }), "--primary");
        assert_eq!(
            format_inspect_provenance(&ColorSource::Derived { note: "= --primary".into() }),
            Some("inherits · --primary".to_string())
        );
        assert!(format_inspect_provenance(&ColorSource::CssVar { token: "ring".into() }).is_none());
    }

    #[test]
    fn format_inspect_metric_source_uses_catalog_token_for_radius_derivation() {
        let source = MetricSource::Derived { note: "md = --radius − 2px".into() };
        assert_eq!(format_inspect_metric_source(&source), "--radius");
        assert_eq!(format_inspect_metric_provenance(&source), Some("md = --radius − 2px".to_string()));
        assert_eq!(
            format_inspect_metric_source(&MetricSource::Scaffold { path: "MetricTokens.control.md.padding_x".into() }),
            "scaffold · MetricTokens.control.md.padding_x"
        );
    }

    #[test]
    fn format_metric_px_rounds_whole_numbers() {
        assert_eq!(format_metric_px(14.0), "14px");
        assert_eq!(format_metric_px(1.5), "1.5px");
    }

    #[test]
    fn format_css_style_ref_uses_shorthand_notation() {
        assert_eq!(
            format_css_style_ref(&ColorSource::TokenAlpha { token: "accent".into(), alpha_percent: 50 }),
            "accent/50"
        );
        assert_eq!(format_css_style_ref(&ColorSource::CssVar { token: "primary".into() }), "primary");
        assert_eq!(
            format_css_style_ref(&ColorSource::StateVar {
                base_token: "secondary".into(),
                layer: InteractionLayer::Hovered,
            }),
            "secondary-hover"
        );
    }

    #[test]
    fn resolve_transparent() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let color = resolver.resolve_decl("transparent").expect("transparent");
        assert_eq!(color.value, hsla(0.0, 0.0, 0.0, 0.0));
        assert!(matches!(color.source, ColorSource::Transparent));
    }

    #[test]
    fn resolve_slash_alpha() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Dark, "test");
        let input = catalog.color("input").expect("input");
        let color = resolver.resolve_decl("input/50").expect("alpha");
        assert!((color.value.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(color.value.h, input.h);
        assert!(matches!(color.source, ColorSource::TokenAlpha { alpha_percent: 50, .. }));
    }

    #[test]
    fn resolve_state_suffix_uses_catalog_override() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let hover = resolver.resolve_decl("primary-hover").expect("hover");
        let primary = catalog.color("primary").expect("primary");
        assert_ne!(hover.value, primary);
        assert!(matches!(hover.source, ColorSource::CssVar { .. }));
    }

    #[test]
    fn resolve_state_suffix_falls_back_to_algorithmic() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let pressed = resolver.resolve_decl("primary-pressed").expect("pressed");
        let primary = catalog.color("primary").expect("primary");
        assert_ne!(pressed.value, primary);
        assert!(matches!(pressed.source, ColorSource::Algorithmic { .. }));
    }
}
