//! CSS fixtures for look-crate tests. Kept local so the look crate does not depend
//! on app demo assets.

use crate::ShadcnLook;

pub(crate) fn native_look() -> ShadcnLook {
    ShadcnLook::from_css_str(include_str!("../tests/fixtures/native.css")).expect("native CSS fixture should parse")
}

pub(crate) fn built_in_look(theme_id: &str) -> ShadcnLook {
    let css = match theme_id {
        "astrovista" => include_str!("../tests/fixtures/tweakcn/astrovista.css"),
        "retro-arcade" => include_str!("../tests/fixtures/tweakcn/retro-arcade.css"),
        other => panic!("look-crate tests have no CSS fixture for `{other}`"),
    };
    ShadcnLook::from_css_str(css).unwrap_or_else(|err| panic!("parse `{theme_id}` CSS fixture: {err}"))
}
