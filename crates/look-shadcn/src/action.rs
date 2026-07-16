use super::ShadcnButtonStyle;

/// Maps a Shadcn button style to tweakcn token names for filled roles.
pub fn style_token_pair(style: ShadcnButtonStyle) -> (&'static str, &'static str) {
    match style {
        ShadcnButtonStyle::Primary => ("primary", "primary-foreground"),
        ShadcnButtonStyle::Secondary => ("secondary", "secondary-foreground"),
        ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost | ShadcnButtonStyle::ContentOnly => {
            ("foreground", "foreground")
        }
    }
}
