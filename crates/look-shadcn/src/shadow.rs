use anyhow::{Context as _, Result};
use gpui::{BoxShadow, Hsla, point, px};
use crate::catalog::CssTokenMap;

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

    Ok(
        BoxShadow {
            color,
            offset: point(px(offset_x), px(offset_y)),
            blur_radius: px(blur),
            spread_radius: px(spread),
        },
    )
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
}
