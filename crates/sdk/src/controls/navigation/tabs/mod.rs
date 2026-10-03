//! Tab navigation with optional retained panels and incoming-content fades.
//!
//! Navigation-only `.items(...)` remains supported. Content-bearing tabs render
//! list and body together by default, or expose slots for a custom layout.
//!
//! ```no_run
//! use std::time::Duration;
//! use gpui::{Context, div, prelude::*};
//! use gpui_luma::controls::tabs::Tabs;
//!
//! # fn build<M: 'static>(cx: &mut Context<M>) {
//! let tabs = Tabs::new("settings")
//!     .tab_with("account", "Account", |_, _| div().child("Account settings"))
//!     .tab_with("appearance", "Appearance", |_, _| div().child("Appearance settings"))
//!     .fade_in(Duration::from_millis(300))
//!     .active("account")
//!     .spawn(cx);
//!
//! // Or place these slots separately in your header and content viewport.
//! let list = tabs.read(cx).tab_list();
//! let body = tabs.read(cx).body();
//! # }
//! ```
//! Content callbacks should retain existing entities; recreating entities inside
//! the callback would reset their state. For a view entity, use `.tab(id, label, view)`.

mod content;
mod control;
mod indicator;
mod model;
mod template;
mod theme;

pub use control::{Tabs, TabsEvent};
pub use content::TabsContent;
use content::{TabsBody, TabsList};
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

pub use theme::{DefaultTabsTheme, TabsBaseline, TabsItemLook, TabsListLook, TabsTheme, default_tabs_theme};

pub use crate::infra::state::{CompositeItemState as TabsItemState, ControlFocusState};
