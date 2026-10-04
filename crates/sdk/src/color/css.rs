//! Concrete color syntax for SDK/theme values, not a complete CSS parser.
use anyhow::{Context as _, Result, anyhow, bail, ensure};
use palette::{Hsla, Srgba, convert::FromColorUnclamped};

use super::ColorValue;

impl ColorValue {
    /// Parse hex, HSL, Oklch, transparent, and `color(srgb|srgb-linear|display-p3 ...)`.
    /// Retains extended channels and source hue. Rejects missing/relative components,
    /// expressions, and invalid alpha rather than silently normalizing source values.
    pub fn parse_css(raw: &str) -> Result<Self> {
        let value = raw.trim().to_ascii_lowercase();
        let color = if value == "transparent" {
            Self::srgb(0.0, 0.0, 0.0, 0.0)
        } else if let Some(hex) = value.strip_prefix('#') {
            parse_hex(hex)?
        } else if let Some((function, inner)) = value.split_once('(') {
            let inner = inner.strip_suffix(')').ok_or_else(|| anyhow!("missing color closing parenthesis"))?;
            match function {
                "hsl" | "hsla" => parse_hsl(inner)?,
                "oklch" => {
                    let (channels, alpha) = channels(inner)?;
                    ensure!(!channels.contains(','), "Oklch requires space-separated channels");
                    let parts = three_channels(channels)?;
                    Self::oklch(number(parts[0], 1.0)?, number(parts[1], 0.4)?, hue(parts[2])?, alpha)
                }
                "color" => {
                    let (channels, alpha) = channels(inner)?;
                    let mut parts = channels.split_whitespace();
                    let space = parts.next().ok_or_else(|| anyhow!("missing color space"))?;
                    let red = number(parts.next().ok_or_else(|| anyhow!("missing red"))?, 1.0)?;
                    let green = number(parts.next().ok_or_else(|| anyhow!("missing green"))?, 1.0)?;
                    let blue = number(parts.next().ok_or_else(|| anyhow!("missing blue"))?, 1.0)?;
                    ensure!(parts.next().is_none(), "too many color channels");
                    match space {
                        "srgb" => Self::srgb(red, green, blue, alpha),
                        "srgb-linear" => Self::linear_srgb(red, green, blue, alpha),
                        "display-p3" => Self::display_p3(red, green, blue, alpha),
                        _ => bail!("unsupported color space `{space}`"),
                    }
                }
                _ => bail!("unsupported color function `{function}`"),
            }
        } else {
            bail!("unsupported color syntax `{raw}`");
        };
        color.validate()?;
        Ok(color)
    }

    /// Format canonical concrete CSS syntax without converting spaces or mapping gamut.
    /// This is source storage syntax, not CSSOM serialization.
    pub fn to_css(self) -> Result<String> {
        self.validate()?;
        let (space, [x, y, z, alpha]) = self.components();
        let function = match space {
            super::SourceSpace::Srgb => "color(srgb",
            super::SourceSpace::LinearSrgb => "color(srgb-linear",
            super::SourceSpace::DisplayP3 => "color(display-p3",
            super::SourceSpace::Oklch => "oklch(",
        };
        let separator = if matches!(self, Self::Oklch(_)) { "" } else { " " };
        Ok(format!("{function}{separator}{x} {y} {z} / {alpha})"))
    }
}

fn parse_hsl(inner: &str) -> Result<ColorValue> {
    let (parts, alpha) = if inner.contains(',') {
        ensure!(!inner.contains('/'), "cannot mix comma and slash HSL syntax");
        let parts: Vec<_> = inner.split(',').map(str::trim).collect();
        ensure!(parts.len() == 3 || parts.len() == 4, "expected three HSL channels and optional alpha");
        let alpha = if parts.len() == 4 { number(parts[3], 1.0)? } else { 1.0 };
        ([parts[0], parts[1], parts[2]], alpha)
    } else {
        let (channels, alpha) = channels(inner)?;
        (three_channels(channels)?, alpha)
    };
    let saturation = percent(parts[1])?;
    let lightness = percent(parts[2])?;
    Ok(ColorValue::Srgb(Srgba::from_color_unclamped(Hsla::new(
        hue(parts[0])?,
        saturation,
        lightness,
        alpha,
    ))))
}

fn channels(inner: &str) -> Result<(&str, f32)> {
    match inner.split_once('/') {
        Some((channels, alpha)) => Ok((channels.trim(), number(alpha.trim(), 1.0)?)),
        None => Ok((inner.trim(), 1.0)),
    }
}

fn three_channels(channels: &str) -> Result<[&str; 3]> {
    let parts: Vec<_> = channels.split_whitespace().collect();
    ensure!(parts.len() == 3, "expected three color channels");
    Ok([parts[0], parts[1], parts[2]])
}

fn number(value: &str, percent_scale: f32) -> Result<f32> {
    let result = match value.strip_suffix('%') {
        Some(percent) => percent.parse::<f32>().context("invalid color percentage")? * percent_scale / 100.0,
        None => value.parse::<f32>().context("invalid color number")?,
    };
    ensure!(result.is_finite(), "color number must be finite");
    Ok(result)
}

fn percent(value: &str) -> Result<f32> {
    ensure!(value.ends_with('%'), "HSL requires percentages");
    number(value, 1.0)
}

fn hue(value: &str) -> Result<f32> {
    for (unit, scale) in [("deg", 1.0), ("grad", 0.9), ("rad", 180.0 / std::f32::consts::PI), ("turn", 360.0)] {
        if let Some(value) = value.strip_suffix(unit) {
            ensure!(!value.ends_with('%'), "hue cannot be a percentage");
            return number(value, 1.0).map(|value| value * scale);
        }
    }
    ensure!(!value.ends_with('%'), "hue cannot be a percentage");
    number(value, 1.0)
}

fn parse_hex(hex: &str) -> Result<ColorValue> {
    ensure!(hex.is_ascii() && hex.bytes().all(|b| b.is_ascii_hexdigit()), "invalid hex color");
    ensure!(matches!(hex.len(), 3 | 4 | 6 | 8), "unsupported hex color length");
    let expanded = if hex.len() <= 4 {
        hex.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        hex.to_string()
    };
    let channel =
        |start: usize| -> Result<f32> { Ok(u8::from_str_radix(&expanded[start..start + 2], 16)? as f32 / 255.0) };
    Ok(ColorValue::srgb(
        channel(0)?,
        channel(2)?,
        channel(4)?,
        if expanded.len() == 8 { channel(6)? } else { 1.0 },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_gamut_and_extended_source_round_trip() {
        let sources = [
            ("color(display-p3 1.1 -0.2 0.3 / 37%)", ColorValue::display_p3(1.1, -0.2, 0.3, 0.37)),
            ("oklch(72% 100% 725deg / .6)", ColorValue::oklch(0.72, 0.4, 725.0, 0.6)),
            ("color(srgb-linear -.25 2 .5)", ColorValue::linear_srgb(-0.25, 2.0, 0.5, 1.0)),
            ("oklch(1.2 0 725)", ColorValue::oklch(1.2, 0.0, 725.0, 1.0)),
        ];
        for (input, expected) in sources {
            let color = ColorValue::parse_css(input).unwrap();
            assert_eq!(color.components(), expected.components());
            assert_eq!(ColorValue::parse_css(&color.to_css().unwrap()).unwrap().components(), color.components());
        }
    }

    #[test]
    fn hsl_units_and_hex_alpha() {
        for input in ["hsl(180 100% 50% / 50%)", "hsla(0.5turn, 100%, 50%, .5)", "hsl(200grad 100% 50% / .5)"] {
            let c = ColorValue::parse_css(input).unwrap().to_srgba_unclamped().unwrap();
            assert!(c.red.abs() < 0.00001 && (c.green - 1.0).abs() < 0.00001 && (c.blue - 1.0).abs() < 0.00001);
            assert_eq!(c.alpha, 0.5);
        }
        assert_eq!(ColorValue::parse_css("#f008").unwrap(), ColorValue::srgb(1.0, 0.0, 0.0, 136.0 / 255.0));
        assert_eq!(ColorValue::parse_css("#ff000080").unwrap().alpha(), 128.0 / 255.0);
    }

    #[test]
    fn extended_hsl_exports_retain_rgb_beyond_srgb() {
        let color = ColorValue::parse_css("hsl(34.9187 147.5678% 35.6981%)").unwrap();
        let rgb = color.to_srgba_unclamped().unwrap();
        assert!(rgb.blue < 0.0);
        assert!(!color.is_in_gamut(super::super::Gamut::Srgb).unwrap());
        assert_eq!(ColorValue::parse_css(&color.to_css().unwrap()).unwrap(), color);
    }

    #[test]
    fn invalid_inputs_fail_without_panicking_or_reinterpreting_alpha() {
        for input in [
            "#éa",
            "#12345",
            "oklch(NaN .2 0)",
            "oklch(.5 -.2 0)",
            "hsl(0 50% 50% / 50)",
            "color(display-p3 1 0 0 / -1)",
            "oklch(.5 .2 none)",
            "oklch(.5,.2,0)",
            "color(srgb 1 0 0 1)",
            "hsl(0,50%,50% / .5)",
        ] {
            assert!(ColorValue::parse_css(input).is_err(), "{input}");
        }
    }

    #[test]
    fn ui_lightness_derivation_retains_unmapped_chroma_hue_and_alpha() {
        let source = ColorValue::oklch(0.7, 0.4, 725.0, 0.37);
        let derived = source.adjust_ui_lightness(-0.03).unwrap();
        assert_eq!(derived.components(), ColorValue::oklch(0.67, 0.4, 725.0, 0.37).components());
        assert_eq!(source.components().1, [0.7, 0.4, 725.0, 0.37]);
        let p3 = ColorValue::display_p3(1.0, 0.0, 0.0, 0.5);
        let before = p3.to_oklcha_unclamped().unwrap();
        let after = p3.adjust_ui_lightness(-0.03).unwrap().to_oklcha_unclamped().unwrap();
        assert_eq!(after.chroma, before.chroma);
        assert_eq!(after.hue, before.hue);
        assert_eq!(after.alpha, before.alpha);
    }
}
