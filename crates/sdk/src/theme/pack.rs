use gpui::Hsla;

/// Shared chrome colors for app shells and gallery surfaces.
#[derive(Clone, Copy)]
pub struct LumaChrome {
    pub app_background: Hsla,
    pub content_background: Hsla,
    pub title_text: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub border: Hsla,
    pub panel_background: Hsla,
}
