use gpui::{AnyElement, Hsla, SharedString, px, prelude::*, svg};
use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum IconSource {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

/// The two icons used by an expanding or opening disclosure affordance.
///
/// The defaults preserve the SDK's existing Lucide appearance. Consumers can
/// replace either side independently with a Lucide glyph or an SVG path.
#[derive(Clone, Debug)]
pub struct DisclosureIcons {
    pub expanded: IconSource,
    pub collapsed: IconSource,
}

impl DisclosureIcons {
    pub fn new(expanded: impl Into<IconSource>, collapsed: impl Into<IconSource>) -> Self {
        Self { expanded: expanded.into(), collapsed: collapsed.into() }
    }
}

impl Default for DisclosureIcons {
    fn default() -> Self {
        Self::new(LucideIcon::ChevronDown, LucideIcon::ChevronRight)
    }
}

/// Selection and completion affordances shared by choice and progress controls.
#[derive(Clone, Debug)]
pub struct SelectionStatusIcons {
    pub selected: IconSource,
    pub completed: IconSource,
}

impl Default for SelectionStatusIcons {
    fn default() -> Self {
        Self { selected: LucideIcon::Check.into(), completed: LucideIcon::Check.into() }
    }
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

pub fn lucide_icon(icon: LucideIcon, color: Hsla, size: f32) -> AnyElement {
    svg().size(px(size)).path(icon.asset_path()).text_color(color).into_any_element()
}

pub fn render_icon_source(icon: &IconSource, color: Hsla, size: f32) -> AnyElement {
    match icon {
        IconSource::Lucide(icon) => lucide_icon(*icon, color, size),
        IconSource::SvgPath(path) => {
            svg().size(px(size)).text_color(color).external_path(path.clone()).into_any_element()
        }
    }
}

/// Renders an icon while inheriting the surrounding text color.
pub fn render_icon_source_inherit(icon: &IconSource, size: f32) -> AnyElement {
    match icon {
        IconSource::Lucide(icon) => svg().size(px(size)).path(icon.asset_path()).into_any_element(),
        IconSource::SvgPath(path) => svg().size(px(size)).external_path(path.clone()).into_any_element(),
    }
}
