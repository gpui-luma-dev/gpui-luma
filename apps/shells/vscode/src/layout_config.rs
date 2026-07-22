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

    pub fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
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

    pub fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Center => 2,
            Self::Justify => 3,
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
        match self {
            Self::PrimarySideBar => Some(["⌘", "B"]),
            Self::Panel => Some(["⌘", "J"]),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::ActivityBar => 0,
            Self::SecondaryActivityBar => 1,
            Self::PrimarySideBar => 2,
            Self::SecondarySideBar => 3,
            Self::Panel => 4,
            Self::StatusBar => 5,
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

    pub fn toggle_region(&mut self, region: LayoutRegion) {
        let visible = self.region_visible(region);
        self.set_region_visible(region, !visible);
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
