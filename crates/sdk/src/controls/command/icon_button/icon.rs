use gpui::SharedString;
use lucide_icons::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum IconButtonIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl IconButtonIcon {
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

impl From<LucideIcon> for IconButtonIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for IconButtonIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for IconButtonIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for IconButtonIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}
