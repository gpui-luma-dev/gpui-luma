use gpui::{AnyElement, FontWeight, Hsla, SharedString, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum IconSource {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl IconSource {
    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::SvgPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) => None,
            Self::SvgPath(path) => Some(path),
        }
    }
}

impl From<LucideIcon> for IconSource {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for IconSource {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for IconSource {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for IconSource {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

pub const LUCIDE_FONT_FAMILY: &str = "lucide";

pub fn lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family(LUCIDE_FONT_FAMILY)
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
