use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};
use lucide_icons::Icon as LucideIcon;

#[derive(Clone, Debug)]
pub enum SelectorItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl SelectorItemIcon {
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

impl From<LucideIcon> for SelectorItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for SelectorItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for SelectorItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for SelectorItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

pub trait SelectorItemLike {
    fn id(&self) -> &SharedString;

    fn is_enabled(&self) -> bool {
        true
    }

    fn label_text(&self) -> &SharedString {
        self.id()
    }

    fn icon(&self) -> Option<&SelectorItemIcon> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct SelectorItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<SelectorItemIcon>,
    pub(crate) enabled: bool,
}

impl SelectorItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<SelectorItemIcon>) -> Self {
        self.icon = Some(icon.into());
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

    pub fn icon_ref(&self) -> Option<&SelectorItemIcon> {
        self.icon.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl SelectorItemLike for SelectorItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn label_text(&self) -> &SharedString {
        &self.label
    }

    fn icon(&self) -> Option<&SelectorItemIcon> {
        self.icon.as_ref()
    }
}

pub fn normalize_selector_items<T: SelectorItemLike>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    items.into_iter().filter(SelectorItemLike::is_enabled).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPath {
    Item(usize),
}

impl SelectorPath {
    pub fn is_item(self, index: usize) -> bool {
        matches!(self, Self::Item(active) if active == index)
    }
}

pub struct SelectorItemRenderModel<'a, T> {
    pub selector_id: &'a SharedString,
    pub item: &'a T,
    pub index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

pub type SelectorItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selector_item_template<T, F, E>(template: F) -> SelectorItemTemplate<T>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}
