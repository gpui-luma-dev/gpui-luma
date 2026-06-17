mod control;
mod model;
mod template;
mod theme;

pub use control::{PagerControl, PagerEvent};
pub use model::{
    PagerBuilder, PagerInfoSlot, PagerModel, PagerPageIndicatorFormatter, PagerPageItem, PagerRenderModel, PagerStyle,
    PagerTemplateParameters, new,
};
pub use template::{PagerTemplate, PagerTemplateHandlers, ThemedPagerTemplate, default_pager_template, numeric_page_items};
pub(crate) use template::{render_info_slot, render_nav_group, render_page_indicator, render_page_size_select};
pub use theme::{DefaultPagerTheme, PagerAppearance, PagerTheme, default_pager_theme};

use gpui::Entity;

pub type Pager = Entity<PagerControl>;
