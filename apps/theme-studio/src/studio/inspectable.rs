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
}
