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
}

impl InspectableId {
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
}
