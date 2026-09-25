//! Test looks from the canonical fallback asset and local tweakcn fixtures.

use crate::ShadcnLook;

pub(crate) fn fallback_look() -> ShadcnLook {
    ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled fallback CSS should parse")
}

pub(crate) fn built_in_look(theme_id: &str) -> ShadcnLook {
    let css = match theme_id {
        "astrovista" => include_str!("../tests/fixtures/tweakcn/astrovista.css"),
        "retro-arcade" => include_str!("../tests/fixtures/tweakcn/retro-arcade.css"),
        other => panic!("look-crate tests have no CSS fixture for `{other}`"),
    };
    ShadcnLook::from_css_str(css).unwrap_or_else(|err| panic!("parse `{theme_id}` CSS fixture: {err}"))
}
