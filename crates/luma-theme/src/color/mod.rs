use anyhow::{Context as _, Result, anyhow, bail};

use palette::{FromColor, Hsl, Oklch, Srgb, Srgba};

#[derive(Debug, Clone, Copy, PartialEq)]
struct ParsedColor {
    hue: f32,
    saturation: f32,
    lightness: f32,
    alpha: f32,
}

pub fn normalize_color(raw: &str, normalize: bool) -> Result<String> {
    if !normalize {
        return Ok(raw.trim().to_string());
    }

    let parsed = parse_color(raw)?;
    Ok(format_hsl(parsed))
}

pub fn apply_lightness_delta(raw: &str, delta: f32) -> Result<String> {
    let mut parsed = parse_color(raw)?;
    parsed.lightness = (parsed.lightness + delta).clamp(0.0, 100.0);
    Ok(format_hsl(parsed))
}

pub fn apply_alpha(raw: &str, alpha: f32) -> Result<String> {
    let mut parsed = parse_color(raw)?;
    parsed.alpha = alpha.clamp(0.0, 1.0);
    Ok(format_hsl(parsed))
}

fn format_hsl(color: ParsedColor) -> String {
    if (color.alpha - 1.0).abs() <= f32::EPSILON {
        format!("hsl({} {}% {}%)", trim_float(color.hue), trim_float(color.saturation), trim_float(color.lightness))
    } else {
        format!(
            "hsla({} {}% {}% / {})",
            trim_float(color.hue),
            trim_float(color.saturation),
            trim_float(color.lightness),
            trim_alpha(color.alpha)
        )
    }
}

fn trim_float(value: f32) -> String {
    if (value.fract()).abs() <= f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        let rounded = (value * 10.0).round() / 10.0;
        if (rounded.fract()).abs() <= f32::EPSILON {
            format!("{}", rounded.round() as i32)
        } else {
            format!("{rounded:.1}")
        }
    }
}

fn trim_alpha(value: f32) -> String {
    if (value.fract()).abs() <= f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value}")
    }
}

fn parse_color(raw: &str) -> Result<ParsedColor> {
    let value = raw.trim();
    if let Some(inner) = value.strip_prefix("hsla(").and_then(|v| v.strip_suffix(')')) {
        return parse_hsl_channels(inner, true);
    }
    if let Some(inner) = value.strip_prefix("hsl(").and_then(|v| v.strip_suffix(')')) {
        return parse_hsl_channels(inner, false);
    }
    if let Some(inner) = value.strip_prefix("oklch(").and_then(|v| v.strip_suffix(')')) {
        return parse_oklch(inner);
    }
    if let Some(inner) = value.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
        return parse_rgb(inner, false);
    }
    if let Some(inner) = value.strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) {
        return parse_rgb(inner, true);
    }
    if value.starts_with('#') {
        return parse_hex(value);
    }

    bail!("unsupported color syntax `{value}`");
}

fn parse_hsl_channels(inner: &str, requires_alpha: bool) -> Result<ParsedColor> {
    let normalized = inner.replace(',', " ");
    let (channels, alpha) = match normalized.split_once('/') {
        Some((channels, alpha)) => (channels.trim(), Some(alpha.trim())),
        None if requires_alpha => bail!("missing alpha channel in hsla value"),
        None => (normalized.trim(), None),
    };

    let mut parts = channels.split_whitespace();
    let hue = next_number(&mut parts, "hue")?;
    let saturation = next_percent(&mut parts, "saturation")?;
    let lightness = next_percent(&mut parts, "lightness")?;
    if parts.next().is_some() {
        bail!("too many hsl channels");
    }

    let alpha = match alpha {
        Some(alpha) => alpha.parse::<f32>().context("invalid hsl alpha")?,
        None => 1.0,
    };

    Ok(ParsedColor { hue, saturation, lightness, alpha })
}

fn parse_oklch(inner: &str) -> Result<ParsedColor> {
    let normalized = inner.replace(',', " ");
    let (channels, alpha) = match normalized.split_once('/') {
        Some((channels, alpha)) => (channels.trim(), Some(alpha.trim())),
        None => (normalized.trim(), None),
    };

    let mut parts = channels.split_whitespace();
    let l = next_number(&mut parts, "oklch lightness")?;
    let c = next_number(&mut parts, "oklch chroma")?;
    let h = next_number(&mut parts, "oklch hue")?;
    if parts.next().is_some() {
        bail!("too many oklch channels");
    }

    let alpha = match alpha {
        Some(alpha) => alpha.parse::<f32>().context("invalid oklch alpha")?,
        None => 1.0,
    };

    let oklch = Oklch::new(l, c, h);
    let hsl: Hsl = Hsl::from_color(Srgb::from_color(oklch));
    Ok(ParsedColor {
        hue: hsl.hue.into_positive_degrees(),
        saturation: hsl.saturation * 100.0,
        lightness: hsl.lightness * 100.0,
        alpha,
    })
}

fn parse_rgb(inner: &str, has_alpha: bool) -> Result<ParsedColor> {
    let normalized = inner.replace(',', " ");
    let mut parts = normalized.split_whitespace();
    let r = next_number(&mut parts, "red")? / 255.0;
    let g = next_number(&mut parts, "green")? / 255.0;
    let b = next_number(&mut parts, "blue")? / 255.0;
    let alpha = if has_alpha {
        next_number(&mut parts, "alpha")?
    } else {
        1.0
    };
    if parts.next().is_some() {
        bail!("too many rgb channels");
    }

    let srgb = Srgba::new(r, g, b, alpha);
    let hsl: Hsl = Hsl::from_color(srgb);
    Ok(ParsedColor {
        hue: hsl.hue.into_positive_degrees(),
        saturation: hsl.saturation * 100.0,
        lightness: hsl.lightness * 100.0,
        alpha,
    })
}

fn parse_hex(value: &str) -> Result<ParsedColor> {
    let hex = value.trim_start_matches('#');
    let (r, g, b, alpha) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)?;
            (r, g, b, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            (r, g, b, 1.0)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            let a = u8::from_str_radix(&hex[6..8], 16)? as f32 / 255.0;
            (r, g, b, a)
        }
        _ => bail!("unsupported hex color `{value}`"),
    };

    let srgb = Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha);
    let hsl: Hsl = Hsl::from_color(srgb);
    Ok(ParsedColor {
        hue: hsl.hue.into_positive_degrees(),
        saturation: hsl.saturation * 100.0,
        lightness: hsl.lightness * 100.0,
        alpha,
    })
}

fn next_number<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    parts
        .next()
        .ok_or_else(|| anyhow!("missing {label} channel"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {label} channel"))
}

fn next_percent<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    let raw = parts.next().ok_or_else(|| anyhow!("missing {label} channel"))?;
    raw.strip_suffix('%')
        .ok_or_else(|| anyhow!("missing percent sign for {label}"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {label} channel"))
}

pub fn first_font(raw: &str) -> String {
    raw.split(',').next().unwrap_or(raw).trim().trim_matches('"').trim_matches('\'').to_string()
}

pub fn rem_to_px(raw: &str, base_px: f32) -> Result<f64> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        let amount = rem.trim().parse::<f64>().context("invalid rem value")?;
        return Ok(amount * base_px as f64);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f64>().context("invalid px value");
    }
    value.parse::<f64>().context("invalid numeric metric value")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_comma_hsl() {
        let out = normalize_color("hsl(204, 12.2%, 91.96%)", true).unwrap();
        assert_eq!(out, "hsl(204 12.2% 92%)");
    }

    #[test]
    fn converts_oklch() {
        let out = normalize_color("oklch(1 0 0)", true).unwrap();
        assert!(out.starts_with("hsl("));
    }

    #[test]
    fn first_font_strips_quotes() {
        assert_eq!(first_font("'Rajdhani', ui-sans-serif"), "Rajdhani");
    }
}
