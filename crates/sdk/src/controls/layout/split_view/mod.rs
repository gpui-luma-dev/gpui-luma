mod control;
mod model;
mod template;
mod theme;

pub use control::{SplitView, SplitViewEvent};
pub use model::{
    PaneRender, SplitViewBuilder, SplitViewModel, SplitViewRenderModel, SplitViewSeparatorVisibility, render_pane,
};
pub use template::{SplitViewTemplate, SplitViewTemplateModifier, ThemedSplitViewTemplate, default_split_view_template};
pub use theme::{DefaultSplitViewTheme, SplitViewLook, SplitViewTheme, default_split_view_theme};

pub use template::{
    SEPARATOR_HITBOX_WIDTH, SEPARATOR_CUE_WIDTH, SEPARATOR_CUE_HOVERED_WIDTH, SEPARATOR_CUE_RADIUS,
    SEPARATOR_CUE_INSET_Y,
};
