pub const DEFAULT_PANEL_HEIGHT_PX: f32 = 180.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PrimarySideBarPosition {
    #[default]
    Left,
    Right,
}

impl PrimarySideBarPosition {
    pub fn id(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PanelAlignment {
    Left,
    Right,
    #[default]
    Center,
    Justify,
}

impl PanelAlignment {
    pub fn id(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::Justify => "justify",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Center => "Center",
            Self::Justify => "Justify",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum LayoutRegion {
    ActivityBar,
    SecondaryActivityBar,
    PrimarySideBar,
    SecondarySideBar,
    Panel,
    StatusBar,
}

impl LayoutRegion {
    pub fn label(self) -> &'static str {
        match self {
            Self::ActivityBar => "Activity Bar",
            Self::SecondaryActivityBar => "Secondary Activity Bar",
            Self::PrimarySideBar => "Primary Side Bar",
            Self::SecondarySideBar => "Secondary Side Bar",
            Self::Panel => "Panel",
            Self::StatusBar => "Status Bar",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::ActivityBar => "activity-bar",
            Self::SecondaryActivityBar => "secondary-activity-bar",
            Self::PrimarySideBar => "primary-side-bar",
            Self::SecondarySideBar => "secondary-side-bar",
            Self::Panel => "panel",
            Self::StatusBar => "status-bar",
        }
    }

    pub fn shortcut_keys(self) -> Option<[&'static str; 2]> {
        let (_, key) = self.shortcut_key()?;
        let modifier = if cfg!(target_os = "macos") { "⌘" } else { "Ctrl" };
        Some([modifier, key])
    }

    pub fn shortcut_keystroke(self) -> Option<&'static str> {
        let (_, key) = self.shortcut_key()?;
        Some(if cfg!(target_os = "macos") {
            match key {
                "B" => "cmd-b",
                "J" => "cmd-j",
                _ => return None,
            }
        } else {
            match key {
                "B" => "ctrl-b",
                "J" => "ctrl-j",
                _ => return None,
            }
        })
    }

    fn shortcut_key(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::PrimarySideBar => Some(("primary-side-bar", "B")),
            Self::Panel => Some(("panel", "J")),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutConfig {
    pub activity_bar_visible: bool,
    pub secondary_activity_bar_visible: bool,
    pub primary_side_bar_visible: bool,
    pub secondary_side_bar_visible: bool,
    pub panel_visible: bool,
    pub status_bar_visible: bool,
    pub primary_side_bar_position: PrimarySideBarPosition,
    pub panel_alignment: PanelAlignment,
    pub panel_height_px: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            activity_bar_visible: true,
            secondary_activity_bar_visible: false,
            primary_side_bar_visible: true,
            secondary_side_bar_visible: false,
            panel_visible: true,
            status_bar_visible: true,
            primary_side_bar_position: PrimarySideBarPosition::Left,
            panel_alignment: PanelAlignment::Center,
            panel_height_px: DEFAULT_PANEL_HEIGHT_PX,
        }
    }
}

impl LayoutConfig {
    pub fn region_visible(&self, region: LayoutRegion) -> bool {
        match region {
            LayoutRegion::ActivityBar => self.activity_bar_visible,
            LayoutRegion::SecondaryActivityBar => self.secondary_activity_bar_visible,
            LayoutRegion::PrimarySideBar => self.primary_side_bar_visible,
            LayoutRegion::SecondarySideBar => self.secondary_side_bar_visible,
            LayoutRegion::Panel => self.panel_visible,
            LayoutRegion::StatusBar => self.status_bar_visible,
        }
    }

    pub fn set_region_visible(&mut self, region: LayoutRegion, visible: bool) {
        match region {
            LayoutRegion::ActivityBar => self.activity_bar_visible = visible,
            LayoutRegion::SecondaryActivityBar => self.secondary_activity_bar_visible = visible,
            LayoutRegion::PrimarySideBar => self.primary_side_bar_visible = visible,
            LayoutRegion::SecondarySideBar => self.secondary_side_bar_visible = visible,
            LayoutRegion::Panel => self.panel_visible = visible,
            LayoutRegion::StatusBar => self.status_bar_visible = visible,
        }
    }

    pub fn visible_region_ids(&self) -> Vec<gpui::SharedString> {
        [
            LayoutRegion::ActivityBar,
            LayoutRegion::SecondaryActivityBar,
            LayoutRegion::PrimarySideBar,
            LayoutRegion::SecondarySideBar,
            LayoutRegion::Panel,
            LayoutRegion::StatusBar,
        ]
        .into_iter()
        .filter(|region| self.region_visible(*region))
        .map(|region| region.id().into())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_matches_startup_state() {
        let config = LayoutConfig::default();

        assert!(config.activity_bar_visible);
        assert!(config.primary_side_bar_visible);
        assert!(config.panel_visible);
        assert_eq!(config.primary_side_bar_position, PrimarySideBarPosition::Left);
        assert_eq!(config.panel_alignment, PanelAlignment::Center);
        assert_eq!(config.panel_height_px, DEFAULT_PANEL_HEIGHT_PX);
    }

    #[test]
    fn region_toggle_updates_visibility_and_ids() {
        let mut config = LayoutConfig::default();
        config.set_region_visible(LayoutRegion::Panel, false);

        assert!(!config.panel_visible);
        assert!(!config.visible_region_ids().iter().any(|id| id == "panel"));
    }
}
