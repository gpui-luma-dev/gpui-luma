pub(super) const TOKEN_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("BASE", &[("background", "Background"), ("foreground", "Foreground")]),
    ("PRIMARY", &[("primary", "Background"), ("primary-foreground", "Foreground")]),
    ("SECONDARY", &[("secondary", "Background"), ("secondary-foreground", "Foreground")]),
    ("ACCENT", &[("accent", "Background"), ("accent-foreground", "Foreground")]),
    ("CARD", &[("card", "Background"), ("card-foreground", "Foreground")]),
    ("POPOVER", &[("popover", "Background"), ("popover-foreground", "Foreground")]),
    ("MUTED", &[("muted", "Background"), ("muted-foreground", "Foreground")]),
    ("DESTRUCTIVE", &[("destructive", "Background"), ("destructive-foreground", "Foreground")]),
    ("BORDER & INPUT", &[("border", "Border"), ("input", "Input"), ("ring", "Ring")]),
    (
        "CHART",
        &[
            ("chart-1", "Chart 1"),
            ("chart-2", "Chart 2"),
            ("chart-3", "Chart 3"),
            ("chart-4", "Chart 4"),
            ("chart-5", "Chart 5"),
        ],
    ),
    (
        "SIDEBAR",
        &[
            ("sidebar", "Background"),
            ("sidebar-foreground", "Foreground"),
            ("sidebar-primary", "Primary"),
            ("sidebar-primary-foreground", "Primary FG"),
            ("sidebar-accent", "Accent"),
            ("sidebar-accent-foreground", "Accent FG"),
            ("sidebar-border", "Border"),
            ("sidebar-ring", "Ring"),
        ],
    ),
];

pub(super) const OTHER_CATEGORIES: &[&str] = &["HSL ADJUSTMENTS", "HS MIXER", "RADIUS", "SPACING", "SHADOW"];
pub(super) const TYPOGRAPHY_CATEGORIES: &[&str] = &["FONT FAMILY"];
pub(super) const METRIC_FIELD_WIDTH: f32 = 70.0;
pub(super) const SHADOW_COLOR_SWATCH_SIZE: f32 = 28.0;
pub(super) const SHADOW_COLOR_FIELD_WIDTH: f32 = 250.0;
pub(super) const SHADOW_SECTION_GAP: f32 = 4.0;
pub(super) const DEFAULT_RADIUS_REM: f32 = 0.5;
pub(super) const DEFAULT_SPACING_REM: f32 = 0.25;
pub(super) const REM_IN_PX: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum SidebarTab {
    #[default]
    Colors,
    Typography,
    Other,
}

impl SidebarTab {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "colors" => Some(Self::Colors),
            "typography" => Some(Self::Typography),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}
