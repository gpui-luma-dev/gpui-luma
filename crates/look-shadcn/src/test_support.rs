//! CSS fixtures for look-crate tests. Uses the assets crate files without a Cargo
//! dependency (that would duplicate `luma_look_shadcn` in the test graph).

use crate::ShadcnLook;

pub(crate) fn native_look() -> ShadcnLook {
    ShadcnLook::from_css_str(include_str!("../../look-shadcn-assets/assets/native.css"))
        .expect("native CSS fixture should parse")
}

pub(crate) fn built_in_look(theme_id: &str) -> ShadcnLook {
    let css = match theme_id {
        "astrovista" => include_str!("../../look-shadcn-assets/assets/tweakcn/astrovista.css"),
        "retro-arcade" => include_str!("../../look-shadcn-assets/assets/tweakcn/retro-arcade.css"),
        other => panic!("look-crate tests have no CSS fixture for `{other}`"),
    };
    ShadcnLook::from_css_str(css).unwrap_or_else(|err| panic!("parse `{theme_id}` CSS fixture: {err}"))
}
