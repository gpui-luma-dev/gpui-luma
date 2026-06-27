#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InspectableId {
    UpgradeSubscription,
    CreateAccount,
    TeamMembers,
    Chat,
    CookieSettings,
    ReportIssue,
    Payments,
    ShareDocument,
    DatePickerRange,
    TreeView,
    Accordion,
    SystemPreferences,
    NavigationSidebar,
}

impl InspectableId {
    pub const ALL: [Self; 13] = [
        Self::UpgradeSubscription,
        Self::CreateAccount,
        Self::TeamMembers,
        Self::Chat,
        Self::CookieSettings,
        Self::ReportIssue,
        Self::Payments,
        Self::ShareDocument,
        Self::DatePickerRange,
        Self::TreeView,
        Self::Accordion,
        Self::SystemPreferences,
        Self::NavigationSidebar,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub const CARDS: [Self; 12] = [
        Self::UpgradeSubscription,
        Self::CreateAccount,
        Self::TeamMembers,
        Self::Chat,
        Self::CookieSettings,
        Self::ReportIssue,
        Self::Payments,
        Self::ShareDocument,
        Self::DatePickerRange,
        Self::TreeView,
        Self::Accordion,
        Self::SystemPreferences,
    ];

    pub const DASHBOARD: [Self; 0] = [];

    pub fn config_key(self) -> &'static str {
        match self {
            Self::UpgradeSubscription => "upgrade_subscription",
            Self::CreateAccount => "create_account",
            Self::TeamMembers => "team_members",
            Self::Chat => "chat",
            Self::CookieSettings => "cookie_settings",
            Self::ReportIssue => "report_issue",
            Self::Payments => "payments",
            Self::ShareDocument => "share_document",
            Self::DatePickerRange => "date_picker_range",
            Self::TreeView => "tree_view",
            Self::Accordion => "accordion",
            Self::SystemPreferences => "system_preferences",
            Self::NavigationSidebar => "navigation_sidebar",
        }
    }
}
