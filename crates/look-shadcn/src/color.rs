use gpui::Hsla;

pub(crate) fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}
