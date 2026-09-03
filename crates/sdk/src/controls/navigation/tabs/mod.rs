mod control;
mod indicator;
mod model;
mod template;
mod theme;

pub use control::{Tabs, TabsEvent};
pub use indicator::{TabsIndicatorMotion, TabsIndicatorPaint, TabsIndicatorRect, indicator_rect_from_bounds};
pub use model::{
    TabsBuilder, TabsItem, TabsItemAccessory, TabsModel, TabsRenderItem, TabsRenderModel, TabsTriggerKind,
    TabsWidthMode,
};
pub use template::{
    TabsBoundsHandler, TabsClickHandler, TabsHoverHandler, TabsItemButtonStyle, TabsItemOverlay, TabsMouseDownHandler,
    TabsMouseUpHandler, TabsOverlayPlacement, TabsOverlayState, TabsTemplate, TabsTemplateHandlers, ThemedTabsTemplate,
    default_tabs_template, render_tab_button, render_tab_button_with_style, render_tabs_overlay_host,
    resolve_tabs_uniform_item_width,
};

pub use theme::{DefaultTabsTheme, TabsItemLook, TabsListLook, TabsTheme, default_tabs_theme};

pub use crate::infra::state::{CompositeItemState as TabsItemState, ControlFocusState};
