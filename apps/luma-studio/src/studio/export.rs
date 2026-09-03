use gpui::Hsla;

pub fn catalog_color_for_token(look: &luma_look_shadcn::ShadcnLook, token: &str) -> Option<Hsla> {
    look.token_color(token).ok()
}

pub fn token_css_name(token: &str) -> String {
    format!("--{}", token.strip_prefix("--").unwrap_or(token))
}
