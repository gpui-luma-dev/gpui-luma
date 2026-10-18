//! Shared default icon choices, with explicit SVG-path overrides.
use serde::Deserialize;
use crate::infra::icon::IconSource;

#[derive(Clone, Debug, PartialEq)]
pub struct StylesheetIcon(pub IconSource);
impl<'de> Deserialize<'de> for StylesheetIcon {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        if let Some(path) = name.strip_prefix("svg:") {
            if path.is_empty() {
                return Err(serde::de::Error::custom("SVG icon path must not be empty"));
            }
            return Ok(Self(IconSource::from(path.to_owned())));
        }
        lucide_svg_static::Icon::all()
            .iter()
            .find(|icon| icon.name() == name)
            .map(|icon| Self(IconSource::Lucide(*icon)))
            .ok_or_else(|| serde::de::Error::custom(format!("unknown Lucide icon: {name}")))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IconRole {
    Clear,
    Search,
    DisclosureExpanded,
    DisclosureCollapsed,
    MenuExpanded,
    MenuCollapsed,
    Selected,
    Completed,
    SelectorTrigger,
    PagerFirst,
    PagerPrevious,
    PagerNext,
    PagerLast,
    PagerGap,
    TextareaResize,
    FolderOpen,
    Folder,
    File,
    DragHandle,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct IconsStylesheet {
    pub clear: Option<StylesheetIcon>,
    pub search: Option<StylesheetIcon>,
    pub disclosure_expanded: Option<StylesheetIcon>,
    pub disclosure_collapsed: Option<StylesheetIcon>,
    pub menu_expanded: Option<StylesheetIcon>,
    pub menu_collapsed: Option<StylesheetIcon>,
    pub selected: Option<StylesheetIcon>,
    pub completed: Option<StylesheetIcon>,
    pub selector_trigger: Option<StylesheetIcon>,
    pub pager_first: Option<StylesheetIcon>,
    pub pager_previous: Option<StylesheetIcon>,
    pub pager_next: Option<StylesheetIcon>,
    pub pager_last: Option<StylesheetIcon>,
    pub pager_gap: Option<StylesheetIcon>,
    pub textarea_resize: Option<StylesheetIcon>,
    pub folder_open: Option<StylesheetIcon>,
    pub folder: Option<StylesheetIcon>,
    pub file: Option<StylesheetIcon>,
    pub drag_handle: Option<StylesheetIcon>,
}
impl IconsStylesheet {
    pub(super) fn with_baseline(mut self, baseline: &Self) -> Self {
        self.clear = self.clear.or_else(|| baseline.clear.clone());
        self.search = self.search.or_else(|| baseline.search.clone());
        self.disclosure_expanded = self.disclosure_expanded.or_else(|| baseline.disclosure_expanded.clone());
        self.disclosure_collapsed = self.disclosure_collapsed.or_else(|| baseline.disclosure_collapsed.clone());
        self.menu_expanded = self.menu_expanded.or_else(|| baseline.menu_expanded.clone());
        self.menu_collapsed = self.menu_collapsed.or_else(|| baseline.menu_collapsed.clone());
        self.selected = self.selected.or_else(|| baseline.selected.clone());
        self.completed = self.completed.or_else(|| baseline.completed.clone());
        self.selector_trigger = self.selector_trigger.or_else(|| baseline.selector_trigger.clone());
        self.pager_first = self.pager_first.or_else(|| baseline.pager_first.clone());
        self.pager_previous = self.pager_previous.or_else(|| baseline.pager_previous.clone());
        self.pager_next = self.pager_next.or_else(|| baseline.pager_next.clone());
        self.pager_last = self.pager_last.or_else(|| baseline.pager_last.clone());
        self.pager_gap = self.pager_gap.or_else(|| baseline.pager_gap.clone());
        self.textarea_resize = self.textarea_resize.or_else(|| baseline.textarea_resize.clone());
        self.folder_open = self.folder_open.or_else(|| baseline.folder_open.clone());
        self.folder = self.folder.or_else(|| baseline.folder.clone());
        self.file = self.file.or_else(|| baseline.file.clone());
        self.drag_handle = self.drag_handle.or_else(|| baseline.drag_handle.clone());
        self
    }
    /// Resolve a role from this layer, then the embedded SDK baseline.
    pub fn resolve(&self, role: IconRole) -> IconSource {
        let baseline = &super::CommonStylesheet::sdk_default().icons;
        let icon = match role {
            IconRole::Clear => self.clear.as_ref().or(baseline.clear.as_ref()),
            IconRole::Search => self.search.as_ref().or(baseline.search.as_ref()),
            IconRole::DisclosureExpanded => self.disclosure_expanded.as_ref().or(baseline.disclosure_expanded.as_ref()),
            IconRole::DisclosureCollapsed => {
                self.disclosure_collapsed.as_ref().or(baseline.disclosure_collapsed.as_ref())
            }
            IconRole::MenuExpanded => self.menu_expanded.as_ref().or(baseline.menu_expanded.as_ref()),
            IconRole::MenuCollapsed => self.menu_collapsed.as_ref().or(baseline.menu_collapsed.as_ref()),
            IconRole::Selected => self.selected.as_ref().or(baseline.selected.as_ref()),
            IconRole::Completed => self.completed.as_ref().or(baseline.completed.as_ref()),
            IconRole::SelectorTrigger => self.selector_trigger.as_ref().or(baseline.selector_trigger.as_ref()),
            IconRole::PagerFirst => self.pager_first.as_ref().or(baseline.pager_first.as_ref()),
            IconRole::PagerPrevious => self.pager_previous.as_ref().or(baseline.pager_previous.as_ref()),
            IconRole::PagerNext => self.pager_next.as_ref().or(baseline.pager_next.as_ref()),
            IconRole::PagerLast => self.pager_last.as_ref().or(baseline.pager_last.as_ref()),
            IconRole::PagerGap => self.pager_gap.as_ref().or(baseline.pager_gap.as_ref()),
            IconRole::TextareaResize => self.textarea_resize.as_ref().or(baseline.textarea_resize.as_ref()),
            IconRole::FolderOpen => self.folder_open.as_ref().or(baseline.folder_open.as_ref()),
            IconRole::Folder => self.folder.as_ref().or(baseline.folder.as_ref()),
            IconRole::File => self.file.as_ref().or(baseline.file.as_ref()),
            IconRole::DragHandle => self.drag_handle.as_ref().or(baseline.drag_handle.as_ref()),
        };
        icon.map(|icon| icon.0.clone()).unwrap_or_else(|| IconSource::from(String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_icon_catalog_and_preserves_svg_sources() {
        assert!(super::super::CommonStylesheet::parse("[common.icons]\nclear='missing-icon'").is_err());
        let sheet = super::super::CommonStylesheet::parse("[common.icons]\nclear='svg:custom/clear.svg'").unwrap();
        assert_eq!(sheet.icons.resolve(IconRole::Clear).svg_path().unwrap().as_ref(), "custom/clear.svg");
        assert_eq!(sheet.icons.resolve(IconRole::Selected).lucide(), Some(lucide_svg_static::Icon::Check));
    }
}

#[cfg(test)]
mod binding_tests {
    use crate::infra::icon::DisclosureIcons;
    use crate::controls::{tabs::TabsBuilder, popup_menu::PopupMenuBuilder};

    #[test]
    fn menu_controls_capture_shared_icons_and_preserve_explicit_overrides() {
        let sheet = super::super::CommonStylesheet::parse("[common.icons]\nmenu_collapsed='svg:menu.svg'").unwrap();
        let explicit = DisclosureIcons::default();
        let tabs = TabsBuilder::new("tabs").stylesheet(&sheet);
        assert_eq!(tabs.model.disclosure_icons.collapsed.svg_path().unwrap().as_ref(), "menu.svg");
        let tabs = tabs.disclosure_icons(explicit.clone()).stylesheet(&sheet);
        assert_eq!(tabs.model.disclosure_icons.collapsed, explicit.collapsed);
        let popup = PopupMenuBuilder::new("menu").stylesheet(&sheet);
        assert_eq!(popup.model.disclosure_icons.collapsed.svg_path().unwrap().as_ref(), "menu.svg");
        let popup = popup.disclosure_icons(explicit.clone()).stylesheet(&sheet);
        assert_eq!(popup.model.disclosure_icons.collapsed, explicit.collapsed);
    }
}
