use super::RadixButtonStyle;

/// Maps a Radix button style to tweakcn token names for filled roles.
pub fn style_token_pair(style: RadixButtonStyle) -> (&'static str, &'static str) {
    match style {
        RadixButtonStyle::Primary => ("primary", "primary-foreground"),
        RadixButtonStyle::Secondary => ("secondary", "secondary-foreground"),
        RadixButtonStyle::Outline | RadixButtonStyle::Ghost => ("foreground", "foreground"),
    }
}
