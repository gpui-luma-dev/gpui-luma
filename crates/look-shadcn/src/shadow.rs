use std::collections::HashMap;

use anyhow::{Context as _, Result};
use gpui::{BoxShadow, Hsla, point, px};
use crate::catalog::CssTokenMap;

pub const SHADOW_LADDER_TOKENS: [&str; 8] = [
    "shadow-2xs",
    "shadow-xs",
    "shadow-sm",
    "shadow",
    "shadow-md",
    "shadow-lg",
    "shadow-xl",
    "shadow-2xl",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowTokenParts {
    pub color: Hsla,
    pub blur_px: f32,
    pub spread_px: f32,
    pub offset_x_px: f32,
    pub offset_y_px: f32,
}

impl ShadowTokenParts {
    pub fn to_css_value(self) -> String {
        format_shadow_layer(self.offset_x_px, self.offset_y_px, self.blur_px, self.spread_px, self.color)
    }
}

/// Generates the standard Shadcn shadow ladder from one editable shadow part set.
pub fn shadow_ladder_overrides(parts: ShadowTokenParts) -> HashMap<String, String> {
    let levels = [
        ("shadow-2xs", 0.5, None),
        ("shadow-xs", 0.5, None),
        ("shadow-sm", 1.0, Some((1.0, 2.0, -1.0, 1.0))),
        ("shadow", 1.0, Some((1.0, 2.0, -1.0, 1.0))),
        ("shadow-md", 1.0, Some((2.0, 4.0, -1.0, 1.0))),
        ("shadow-lg", 1.0, Some((4.0, 6.0, -1.0, 1.0))),
        ("shadow-xl", 1.0, Some((8.0, 10.0, -1.0, 1.0))),
        ("shadow-2xl", 2.5, None),
    ];

    levels
        .into_iter()
        .map(|(token, primary_alpha, secondary)| {
            let primary = ShadowTokenParts { color: with_alpha(parts.color, parts.color.a * primary_alpha), ..parts };
            let value = if let Some((offset_y_factor, blur_px, spread_px, _)) = secondary {
                let secondary = format_shadow_layer(
                    parts.offset_x_px,
                    parts.offset_y_px * offset_y_factor,
                    blur_px,
                    spread_px,
                    with_alpha(parts.color, parts.color.a),
                );
                format!("{}, {}", primary.to_css_value(), secondary)
            } else {
                primary.to_css_value()
            };
            (token.to_string(), value)
        })
        .collect()
}

fn with_alpha(mut color: Hsla, alpha: f32) -> Hsla {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

fn format_shadow_layer(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Hsla) -> String {
    format!(
        "{}px {}px {}px {}px hsl({} {}% {}% / {})",
        format_shadow_number(offset_x),
        format_shadow_number(offset_y),
        format_shadow_number(blur),
        format_shadow_number(spread),
        (color.h * 360.0).round(),
        (color.s * 100.0).round(),
        (color.l * 100.0).round(),
        format_shadow_number(color.a),
    )
}

fn format_shadow_number(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = format!("{rounded:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(crate) fn parse_shadow_token(catalog: &CssTokenMap, token_key: &str) -> Result<Vec<BoxShadow>> {
    let raw = catalog.get(token_key).with_context(|| format!("missing css token `--{token_key}`"))?;
    parse_box_shadow_value(raw)
}

fn parse_box_shadow_value(raw: &str) -> Result<Vec<BoxShadow>> {
    let layers = split_shadow_layers(raw);
    layers.into_iter().map(parse_shadow_layer).collect()
}

fn split_shadow_layers(raw: &str) -> Vec<&str> {
    let mut layers = Vec::new();
    let mut start = 0;
    let mut depth: usize = 0;

    for (index, ch) in raw.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let layer = raw[start..index].trim();
                if !layer.is_empty() {
                    layers.push(layer);
                }
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = raw[start..].trim();
    if !tail.is_empty() {
        layers.push(tail);
    }

    layers
}

fn parse_shadow_layer(layer: &str) -> Result<BoxShadow> {
    let (lengths, color_raw) = split_lengths_and_color(layer)?;
    let mut parts = lengths.split_whitespace();
    let offset_x = next_length(&mut parts, "shadow offset-x")?;
    let offset_y = next_length(&mut parts, "shadow offset-y")?;
    let blur = next_length(&mut parts, "shadow blur")?;
    let spread = next_length(&mut parts, "shadow spread").unwrap_or(0.0);
    if parts.next().is_some() {
        anyhow::bail!("unexpected extra values in shadow layer `{layer}`");
    }

    let color = catalog_color(color_raw)?;

    Ok(BoxShadow {
        color,
        offset: point(px(offset_x), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    })
}

fn split_lengths_and_color(layer: &str) -> Result<(&str, &str)> {
    if let Some(index) = layer.find("hsl") {
        let (lengths, color) = layer.split_at(index);
        return Ok((lengths.trim(), color.trim()));
    }
    if let Some(index) = layer.find('#') {
        let (lengths, color) = layer.split_at(index);
        return Ok((lengths.trim(), color.trim()));
    }
    anyhow::bail!("unsupported shadow color syntax in `{layer}`")
}

fn next_length<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    let raw = parts.next().with_context(|| format!("missing {label}"))?;
    parse_length_px(raw).with_context(|| format!("parse {label} `{raw}`"))
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

fn catalog_color(raw: &str) -> Result<Hsla> {
    let mut tokens = std::collections::BTreeMap::new();
    tokens.insert("_shadow".to_string(), raw.to_string());
    CssTokenMap::from_map(tokens).color("_shadow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_shadow_layer() {
        let shadows = parse_box_shadow_value("0px 2px 0px 0px hsl(0 0% 20% / 0.07)").unwrap();
        assert_eq!(shadows.len(), 1);
        assert!(shadows[0].color.a > 0.0);
    }

    #[test]
    fn parses_multi_layer_shadow() {
        let raw = "0px 2px 0px 0px hsl(0 0% 20% / 0.15), 0px 1px 2px -1px hsl(0 0% 20% / 0.15)";
        let shadows = parse_box_shadow_value(raw).unwrap();
        assert_eq!(shadows.len(), 2);
    }

    #[test]
    fn shadow_ladder_preserves_parts_and_restores_scale() {
        let parts = ShadowTokenParts {
            color: Hsla { h: 0.5, s: 0.4, l: 0.2, a: 0.2 },
            blur_px: 10.0,
            spread_px: -2.0,
            offset_x_px: 3.0,
            offset_y_px: 4.0,
        };
        let ladder = shadow_ladder_overrides(parts);

        assert_eq!(ladder.len(), SHADOW_LADDER_TOKENS.len());
        assert!(ladder["shadow-2xs"].contains("0.1"));
        assert!(ladder["shadow"].contains(", 3px 4px 2px -1px"));
        assert!(ladder["shadow-xl"].contains("3px 32px 10px -1px"));
        assert!(ladder["shadow-2xl"].contains("0.5"));
    }
}
