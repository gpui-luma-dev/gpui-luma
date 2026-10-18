use gpui::{Pixels, px};

/// Navigation presentation; width and visibility belong to the host.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SidebarPresentation {
    #[default]
    Expanded,
    Icons,
}

/// Typed metric scale for sidebar layout. Resolved by look crates (e.g. Shadcn).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarMetricScale {
    pub width_expanded: Pixels,
    pub width_icon_rail: Pixels,
    pub width_mobile: Pixels,
    pub item_height: Pixels,
    pub icon_size: Pixels,
    pub rail_hit_width: Pixels,
    pub popover_offset: Pixels,
}

impl Default for SidebarMetricScale {
    fn default() -> Self {
        Self::from_stylesheet(crate::theme::stylesheet::CommonStylesheet::sdk_default())
    }
}

impl SidebarMetricScale {
    /// Capture resolved sidebar dimensions; explicit host metrics can be edited afterward.
    pub fn from_stylesheet(stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let base = crate::theme::stylesheet::CommonStylesheet::sdk_default().sidebar.authored_geometry("md");
        let geometry = stylesheet.sidebar.authored_geometry("md").with_fallback(&base);
        Self {
            width_expanded: px(geometry.width_expanded.unwrap_or_default()),
            width_icon_rail: px(geometry.width_icon_rail.unwrap_or_default()),
            width_mobile: px(geometry.width_mobile.unwrap_or_default()),
            item_height: px(geometry.item_height.unwrap_or_default()),
            icon_size: px(geometry.item_icon_size.unwrap_or_default()),
            rail_hit_width: px(geometry.rail_hit_width.unwrap_or_default()),
            popover_offset: px(geometry.popover_offset.unwrap_or_default()),
        }
    }
}
