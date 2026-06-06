use anyhow::{Context as _, Result, anyhow, bail};
use gpui::{Hsla, hsla};
use palette::{FromColor, Hsl, Oklch, Srgb, Srgba};

pub(crate) fn parse_css_color(raw: &str) -> Result<Hsla> {
    let parsed = parse_color(raw.trim())?;
    Ok(hsla(parsed.hue / 360.0, parsed.saturation / 100.0, parsed.lightness / 100.0, parsed.alpha))
}

pub(crate) fn darken(color: Hsla, delta: f32) -> Hsla {
    adjust_lightness(color, -delta)
}

/// Adjusts perceived lightness in Oklch space, preserving hue and chroma.
pub(crate) fn adjust_lightness(color: Hsla, delta: f32) -> Hsla {
    let hsl = Hsl::new(color.h * 360.0, color.s, color.l);
    let mut oklch = Oklch::from_color(Srgb::from_color(hsl));
    oklch.l = (oklch.l + delta).clamp(0.0, 1.0);
    let adjusted: Hsl = Hsl::from_color(Srgb::from_color(oklch));
    hsla(adjusted.hue.into_positive_degrees() / 360.0, adjusted.saturation, adjusted.lightness, color.a)
}

pub(crate) fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

#[derive(Debug, Clone, Copy)]
struct ParsedColor {
    hue: f32,
    saturation: f32,
    lightness: f32,
    alpha: f32,
}

fn parse_color(value: &str) -> Result<ParsedColor> {
    if let Some(inner) = value.strip_prefix("hsla(").and_then(|v| v.strip_suffix(')')) {
        return parse_hsl_channels(inner, true);
    }
    if let Some(inner) = value.strip_prefix("hsl(").and_then(|v| v.strip_suffix(')')) {
        return parse_hsl_channels(inner, false);
    }
    if let Some(inner) = value.strip_prefix("oklch(").and_then(|v| v.strip_suffix(')')) {
        return parse_oklch(inner);
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
        Some(alpha) => parse_alpha(alpha)?,
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
        Some(alpha) => parse_alpha(alpha)?,
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

fn parse_hex(value: &str) -> Result<ParsedColor> {
    let hex = value.trim_start_matches('#');
    let rgba = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&format!("{}{}", &hex[0..1], &hex[0..1]), 16)?;
            let g = u8::from_str_radix(&format!("{}{}", &hex[1..2], &hex[1..2]), 16)?;
            let b = u8::from_str_radix(&format!("{}{}", &hex[2..3], &hex[2..3]), 16)?;
            Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
        }
        _ => bail!("unsupported hex color `{value}`"),
    };
    let hsl: Hsl = Hsl::from_color(rgba);
    Ok(ParsedColor {
        hue: hsl.hue.into_positive_degrees(),
        saturation: hsl.saturation * 100.0,
        lightness: hsl.lightness * 100.0,
        alpha: rgba.alpha,
    })
}

fn next_number<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    parts
        .next()
        .ok_or_else(|| anyhow!("missing {label}"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {label}"))
}

fn next_percent<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    let value = parts.next().ok_or_else(|| anyhow!("missing {label}"))?;
    value
        .strip_suffix('%')
        .ok_or_else(|| anyhow!("missing percent for {label}"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {label}"))
}

fn parse_alpha(alpha: &str) -> Result<f32> {
    let alpha = alpha.trim();
    if let Some(percent) = alpha.strip_suffix('%') {
        return percent.parse::<f32>().context("invalid alpha percentage").map(|v| v / 100.0);
    }
    alpha.parse::<f32>().context("invalid alpha")
}

#[cfg(test)]
mod tests {
    use super::parse_css_color;

    #[test]
    fn oklch_alpha_accepts_percentage() {
        let color = parse_css_color("oklch(0.633 0.000 263.283 / 73%)").expect("parse");
        assert!((color.a - 0.73).abs() < f32::EPSILON);
    }

    #[test]
    fn oklch_alpha_accepts_decimal() {
        let color = parse_css_color("oklch(1 0 0 / 0.15)").expect("parse");
        assert!((color.a - 0.15).abs() < f32::EPSILON);
    }
}
