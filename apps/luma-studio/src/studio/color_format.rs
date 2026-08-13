use gpui::Hsla;

/// Formats an ordinary theme color for Luma Studio's compact readouts.
pub(crate) fn format_compact_hsla(color: Hsla) -> String {
    format!(
        "hsla({} {}% {}% / {})",
        rounded_channel(color.h.rem_euclid(1.0) * 360.0),
        rounded_channel(color.s.clamp(0.0, 1.0) * 100.0),
        rounded_channel(color.l.clamp(0.0, 1.0) * 100.0),
        compact_alpha(color.a),
    )
}

/// Parses the compact HSLA notation used by the Luma Studio theme color fields.
pub(crate) fn parse_compact_hsla(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    let inner = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("hsl(") && trimmed.ends_with(')') {
        &trimmed[4..trimmed.len() - 1]
    } else if trimmed.len() >= 6 && trimmed[..5].eq_ignore_ascii_case("hsla(") && trimmed.ends_with(')') {
        &trimmed[5..trimmed.len() - 1]
    } else {
        return None;
    };

    let normalized = inner.replace(',', " ");
    let (channels, alpha) = normalized
        .split_once('/')
        .map_or((normalized.as_str(), None), |(channels, alpha)| (channels, Some(alpha.trim())));
    let mut parts = channels.split_whitespace();
    let hue = parse_finite(parts.next()?)? / 360.0;
    let saturation = parse_percent(parts.next()?)?;
    let lightness = parse_percent(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    let alpha = alpha.map(parse_finite).unwrap_or(Some(1.0))?;

    Some(Hsla {
        h: hue.rem_euclid(1.0),
        s: saturation.clamp(0.0, 1.0),
        l: lightness.clamp(0.0, 1.0),
        a: alpha.clamp(0.0, 1.0),
    })
}

fn parse_finite(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().filter(|value| value.is_finite())
}

fn parse_percent(value: &str) -> Option<f32> {
    value.strip_suffix('%').and_then(parse_finite).map(|value| value / 100.0)
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(alpha: f32) -> String {
    let rounded = (alpha.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() <= f32::EPSILON {
        format!("{}", rounded.round() as i32)
    } else {
        format!("{rounded:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Hsla, hsla};

    use super::{format_compact_hsla, parse_compact_hsla};

    #[test]
    fn formats_ordinary_color() {
        assert_eq!(format_compact_hsla(hsla(210.0 / 360.0, 0.5, 0.4, 0.75)), "hsla(210 50% 40% / 0.75)");
    }

    #[test]
    fn normalizes_hue_and_clamps_channels() {
        assert_eq!(format_compact_hsla(Hsla { h: -30.0 / 360.0, s: 1.2, l: -0.1, a: 1.2 }), "hsla(330 100% 0% / 1)");
    }

    #[test]
    fn formats_grayscale_and_transparent_colors() {
        assert_eq!(format_compact_hsla(hsla(0.0, 0.0, 0.5, 0.0)), "hsla(0 0% 50% / 0)");
    }

    #[test]
    fn parses_compact_hsla() {
        let color = parse_compact_hsla("hsla(210 50% 40% / 0.75)").expect("valid HSLA");
        assert_eq!(format_compact_hsla(color), "hsla(210 50% 40% / 0.75)");
    }

    #[test]
    fn parses_clamped_and_wrapped_hsla() {
        let color = parse_compact_hsla("hsl(-30 120% -10%)").expect("valid HSL");
        assert_eq!(format_compact_hsla(color), "hsla(330 100% 0% / 1)");
    }

    #[test]
    fn rounds_boundary_values_compactly() {
        assert_eq!(format_compact_hsla(hsla(0.999, 0.1234, 0.9876, 0.126)), "hsla(360 12% 99% / 0.13)");
    }
}
