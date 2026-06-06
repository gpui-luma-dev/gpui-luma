use gpui_luma::theme::{ThemePartUsage, ThemeUsage};
use gpui_luma_look_shadcn::all_shadcn_theme_usages;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InspectableId {
    UpgradeSubscription,
    CreateAccount,
    TeamMembers,
    Chat,
    CookieSettings,
    ReportIssue,
    Payments,
    NavigationSidebar,
}

impl InspectableId {
    pub const ALL: [Self; 8] = [
        Self::UpgradeSubscription,
        Self::CreateAccount,
        Self::TeamMembers,
        Self::Chat,
        Self::CookieSettings,
        Self::ReportIssue,
        Self::Payments,
        Self::NavigationSidebar,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub fn config_key(self) -> &'static str {
        match self {
            Self::UpgradeSubscription => "upgrade_subscription",
            Self::CreateAccount => "create_account",
            Self::TeamMembers => "team_members",
            Self::Chat => "chat",
            Self::CookieSettings => "cookie_settings",
            Self::ReportIssue => "report_issue",
            Self::Payments => "payments",
            Self::NavigationSidebar => "navigation_sidebar",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::UpgradeSubscription => "Form (composite)",
            Self::CreateAccount => "Button",
            Self::TeamMembers => "Selector",
            Self::Chat => "TextField",
            Self::CookieSettings => "Switch",
            Self::ReportIssue => "TextField",
            Self::Payments => "ListView",
            Self::NavigationSidebar => "Navigation Sidebar",
        }
    }

    pub fn settings_title(self) -> String {
        match self {
            Self::UpgradeSubscription => "Upgrade Subscription".to_string(),
            Self::CreateAccount => "Create Account".to_string(),
            Self::TeamMembers => "Team Members".to_string(),
            Self::Chat => "Chat".to_string(),
            Self::CookieSettings => "Cookie Settings".to_string(),
            Self::ReportIssue => "Report an Issue".to_string(),
            Self::Payments => "Payments".to_string(),
            Self::NavigationSidebar => "Navigation Sidebar".to_string(),
        }
    }

    pub fn theme_usage(self) -> Option<&'static ThemeUsage> {
        let label = self.label();
        all_shadcn_theme_usages().iter().copied().find(|usage| usage.label == label)
    }

    pub fn scale_fields(self) -> &'static [ScaleField] {
        match self {
            Self::CookieSettings => SWITCH_SCALE_FIELDS,
            _ => &[],
        }
    }
}

const SWITCH_SCALE_FIELDS: &[ScaleField] = &[
    ScaleField::new("track_width", "Width"),
    ScaleField::new("track_height", "Height"),
    ScaleField::new("thumb_size", "Thumb"),
];

#[derive(Clone, Copy, Debug)]
pub struct ScaleField {
    pub key: &'static str,
    pub label: &'static str,
}

impl ScaleField {
    const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

pub fn parts_for(id: InspectableId) -> &'static [ThemePartUsage] {
    id.theme_usage().map(|usage| usage.parts).unwrap_or(&[])
}
