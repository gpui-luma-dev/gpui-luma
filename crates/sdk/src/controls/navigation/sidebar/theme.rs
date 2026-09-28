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
        Self {
            width_expanded: px(256.0),
            width_icon_rail: px(48.0),
            width_mobile: px(288.0),
            item_height: px(32.0),
            icon_size: px(16.0),
            rail_hit_width: px(6.0),
            popover_offset: px(8.0),
        }
    }
}
