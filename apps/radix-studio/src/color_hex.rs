//! Shared six-digit RGB hex formatting/parsing for the app's color readouts and editors.
//! Alpha is intentionally omitted: the palette editor uses opaque seed colors.

use gpui::Hsla;

/// Uppercase RGB without a leading `#`, using GPUI's HSL-to-RGB conversion.
pub fn format_hex(color: Hsla) -> String {
    let rgb = color.to_rgb();
    format!(
        "{:02X}{:02X}{:02X}",
        (rgb.r * 255.0).round() as u8,
        (rgb.g * 255.0).round() as u8,
        (rgb.b * 255.0).round() as u8
    )
}

/// Accept exactly six hex digits, with optional `#` and surrounding whitespace.
pub fn parse_hex(value: &str) -> Option<Hsla> {
    let value = value.trim();
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let rgb = u32::from_str_radix(value, 16).ok()?;
    Some(gpui::rgb(rgb).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_hex_round_trips_and_rejects_incomplete_input() {
        for hex in ["000000", "FFFFFF", "111113", "3E63DD", "FF0000", "00FF00", "0000FF"] {
            assert_eq!(format_hex(parse_hex(hex).expect("valid hex")), hex);
        }
        assert_eq!(format_hex(parse_hex("  #3e63dd  ").expect("prefixed hex")), "3E63DD");
        for invalid in ["", "123", "12345", "1234567", "GG0000", "+12345", "#1234", "１２３"] {
            assert!(parse_hex(invalid).is_none(), "accepted {invalid:?}");
        }
    }
}
