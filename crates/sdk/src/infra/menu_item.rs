use gpui::SharedString;
use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum MenuItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
    /// SVG supplied by the application's registered asset source.
    AssetPath(SharedString),
}

impl MenuItemIcon {
    /// Use an SVG from the application's registered asset source.
    pub fn asset(path: impl Into<SharedString>) -> Self {
        Self::AssetPath(path.into())
    }

    pub fn asset_path(&self) -> Option<&SharedString> {
        match self {
            Self::AssetPath(path) => Some(path),
            _ => None,
        }
    }

    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::SvgPath(_) | Self::AssetPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) | Self::AssetPath(_) => None,
            Self::SvgPath(path) => Some(path),
        }
    }
}

impl From<LucideIcon> for MenuItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for MenuItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for MenuItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for MenuItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<MenuItemIcon>,
    pub(crate) submenu_items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    separator: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, submenu_items: Vec::new(), enabled: true, separator: false }
    }

    /// A noninteractive divider styled by the active floating-menu look.
    pub fn separator(id: impl Into<SharedString>) -> Self {
        Self { separator: true, ..Self::new(id) }
    }

    /// Whether this entry is a divider rather than an action or submenu.
    pub fn is_separator(&self) -> bool {
        self.separator
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<MenuItemIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.submenu_items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn icon_ref(&self) -> Option<&MenuItemIcon> {
        self.icon.as_ref()
    }

    pub fn submenu_items(&self) -> &[MenuItem] {
        &self.submenu_items
    }

    /// Whether the entry can be selected or navigated to. Separators always return false.
    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.separator
    }
}
