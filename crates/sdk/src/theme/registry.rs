#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeUsage {
    pub label: &'static str,
    pub parts: &'static [ThemePartUsage],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePartUsage {
    pub part: &'static str,
    pub token: &'static str,
    pub states: &'static [&'static str],
    pub look_fields: &'static [&'static str],
}
