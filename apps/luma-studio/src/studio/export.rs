pub fn token_css_name(token: &str) -> String {
    format!("--{}", token.strip_prefix("--").unwrap_or(token))
}
