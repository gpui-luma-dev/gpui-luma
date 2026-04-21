#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeUsage {
    pub component: &'static str,
    pub parts: &'static [ThemePartUsage],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePartUsage {
    pub part: &'static str,
    pub token: &'static str,
    pub states: &'static [&'static str],
    pub appearance_fields: &'static [&'static str],
}
