use super::super::inspectable::InspectableId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContentTab {
    #[default]
    Cards,
    Dashboard,
    Typography,
    Palette,
    ThemeUsage,
    Controls,
    Components,
}

impl ContentTab {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "cards" => Some(Self::Cards),
            "dashboard" => Some(Self::Dashboard),
            "typography" => Some(Self::Typography),
            "palette" => Some(Self::Palette),
            "theme-usage" => Some(Self::ThemeUsage),
            "controls" => Some(Self::Controls),
            "components" => Some(Self::Components),
            _ => None,
        }
    }

    pub fn panels(self) -> Option<&'static [InspectableId]> {
        match self {
            Self::Cards => Some(&InspectableId::CARDS),
            Self::Dashboard => Some(&InspectableId::DASHBOARD),
            Self::Typography => None,
            Self::Palette => None,
            Self::ThemeUsage => None,
            Self::Controls => None,
            Self::Components => None,
        }
    }
}
