use crate::theme::{LumaTypography, MetricTokens};

use super::CssTokenMap;

const GENERIC_FAMILIES: &[&str] = &[
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
];

pub(crate) fn metrics_from_catalog(catalog: &CssTokenMap, mut scaffold: MetricTokens) -> MetricTokens {
    let Some(radius_raw) = catalog.get("radius") else {
        return scaffold;
    };
    let Some(base_px) = parse_length_px(radius_raw) else {
        return scaffold;
    };

    let scale = base_px / scaffold.radius.md;
    scaffold.radius.sm *= scale;
    scaffold.radius.md = base_px;
    scaffold.radius.lg *= scale;
    scaffold.radius.xl *= scale;

    scale_control_metrics(&mut scaffold.sm, scale, base_px);
    scale_control_metrics(&mut scaffold.md, scale, base_px);
    scale_control_metrics(&mut scaffold.lg, scale, base_px);
    scale_control_metrics(&mut scaffold.control.sm, scale, base_px);
    scale_control_metrics(&mut scaffold.control.md, scale, base_px);
    scale_control_metrics(&mut scaffold.control.lg, scale, base_px);

    scaffold
}

fn scale_control_metrics(metrics: &mut crate::theme::ControlMetricTokens, scale: f32, md_radius: f32) {
    metrics.radius = md_radius;
    let _ = scale;
}

pub(crate) fn typography_from_catalog(catalog: &CssTokenMap, mut scaffold: LumaTypography) -> LumaTypography {
    if let Some(raw) = catalog.get("font-sans") {
        scaffold.font.sans.family = first_font_family(raw);
    }
    if let Some(raw) = catalog.get("font-mono") {
        scaffold.font.mono.family = first_font_family(raw);
    }
    if let Some(raw) = catalog.get("font-serif") {
        scaffold.font.serif.family = first_font_family(raw);
    }
    scaffold
}

fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

fn normalize_font_family(family: &str) -> String {
    // Fontshare variable TTF typographic family (name ID 16); tweakcn exports "Rajdhani".
    if family.eq_ignore_ascii_case("Rajdhani") {
        return "Rajdhani Variable".to_string();
    }
    family.to_string()
}

fn first_font_family(raw: &str) -> String {
    for part in raw.split(',') {
        let candidate = part.trim().trim_matches(['\'', '"']);
        if candidate.is_empty() {
            continue;
        }
        if GENERIC_FAMILIES.iter().any(|generic| candidate.eq_ignore_ascii_case(generic)) {
            continue;
        }
        return normalize_font_family(candidate);
    }

    normalize_font_family(raw.split(',').next().unwrap_or(raw).trim().trim_matches(['\'', '"']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rem_radius_to_pixels() {
        assert_eq!(parse_length_px("0.625rem"), Some(10.0));
    }

    #[test]
    fn picks_first_non_generic_font() {
        assert_eq!(first_font_family("ui-sans-serif, system-ui, 'Rajdhani', sans-serif"), "Rajdhani Variable");
    }

    #[test]
    fn maps_rajdhani_css_export_to_variable_family() {
        assert_eq!(first_font_family("Rajdhani, sans-serif"), "Rajdhani Variable");
    }
}
