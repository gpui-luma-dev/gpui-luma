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
        }
    }
}
